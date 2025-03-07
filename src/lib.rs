use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, BufWriter, Write},
};

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

mod latex;
mod ooxml;
mod peekaboo;
mod tag;

use peekaboo::Boo;
use tag::{normalize, InputError, Link, Tag};

fn blink(value: bool) -> Option<()> {
    if value {
        Some(())
    } else {
        None
    }
}

pub fn relationships<R: Read>(
    parser: &mut EventReader<BufReader<R>>,
) -> Result<HashMap<String, String>, xml::reader::Error> {
    let mut count = 0;
    let mut rels = HashMap::<String, String>::default();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.local_name.as_str() {
                    "Relationships" => continue,
                    "Relationship" => {
                        count += 1;
                        let id = attributes.iter().find(|&a| normalize(&a.name) == "Id");
                        let target = attributes.iter().find(|&a| normalize(&a.name) == "Target");
                        if id.is_none() && target.is_none() {
                            log::error!(
                                "Relationship #{count} is missing attributes 'Id' and 'Target'"
                            );
                        } else if id.is_none() {
                            log::error!("Relationship #{count} is missing attribute 'Id'");
                        } else if target.is_none() {
                            log::error!("Relationship #{count} is missing attribute 'Target'");
                        } else {
                            // if id.is_some() && target.is_some()
                            let id = id.expect("Id was previously confirmed to be Some");
                            let target =
                                target.expect("Target was previously confirmed to be Some");
                            rels.insert(id.value.clone(), target.value.clone());
                        }
                    }
                    x => log::warn!("Unknown entry in Relationships: {x:?}"),
                }
            }
            Ok(XmlEvent::EndDocument { .. }) => break,
            Ok(_) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(rels)
}

#[derive(Debug, PartialEq)]
enum State {
    OpenedTag(Tag),
    ClosedTag,
    FoundContent(String),
    AttributesMissing,
    RelationshipMissing,
    Happy,
    End,
}

fn start_element(
    buf_writer: &mut BufWriter<File>,
    name: &OwnedName,
    attributes: &Vec<OwnedAttribute>,
    math_mode: &mut bool,
    nary_has_chr: &mut Option<bool>,
) -> std::io::Result<State> {
    let tag = Tag::try_from((name, attributes));

    if let Err(InputError::MissingAttributes { id, missing }) = &tag {
        log::error!("Tag '{id}' is missing attributes: {missing:?}");
        return Ok(State::AttributesMissing);
    }

    let tag = tag.expect("Error case was handled");

    match &tag {
        Tag::MoMathPara => {
            if *math_mode {
                log::error!("Entering Math Mode multiple times");
            } else {
                *math_mode = true;
                write!(buf_writer, "$$")?;
            }
        },
        Tag::MDelim => write!(buf_writer, "(")?,
        Tag::MRad => write!(buf_writer, "\\sqrt")?,
        Tag::MDeg => write!(buf_writer, "[")?,
        Tag::MSub => write!(buf_writer, "_{{")?,
        Tag::MSup => write!(buf_writer, "^{{")?,
        Tag::MNaryPr => {
            if nary_has_chr.is_none() {
                *nary_has_chr = Some(false);
            } else {
                log::error!("Nested <m:naryPr> detected");
            }
        },
        Tag::MChr { value } => {
            if let Some(false) = nary_has_chr {
                *nary_has_chr = Some(true);
            } else if let Some(true) = nary_has_chr {
                log::error!("<m:naryPr> has multiple <m:chr> specified");
            }
            write!(
                buf_writer,
                "\\{}",
                match value.as_str() {
                    "⋀" => "bigwedge",
                    "⋁" => "bigvee",
                    "⋂" => "bigcap",
                    "⋃" => "bigcup",
                    "∐" => "coprod",
                    "∏" => "prod",
                    "∑" => "sum",
                    "∮" => "oint",
                    _ => "",
                }
            )?;
        }
        Tag::MFraction => write!(buf_writer, "\\frac")?,
        Tag::MNum => write!(buf_writer, "{{")?,
        Tag::MDen => write!(buf_writer, "{{")?,
        Tag::Unknown { id } => {
            log::warn!("Ignoring tag '{id}'")
        }
        _ => {}
    };

    Ok(State::OpenedTag(tag))
}

fn end_element(
    buf_writer: &mut BufWriter<File>,
    stack: &Boo<Tag>,
    rels: &HashMap<String, String>,
    math_mode: &mut bool,
    nary_has_chr: &mut Option<bool>,
) -> std::io::Result<State> {
    log::debug!("Stack: {:?}", &stack);

    if let Some(rel) = ooxml::drawing(stack) {
        // ["w:drawing", ("wp:inline"/"wp:anchor"), "a:graphic", "a:graphicData", "pic:pic", "pic:blipFill", "a:blip"]
        latex::drawing(buf_writer, rels, rel)?;
    } else if let Some(hyperlink) = ooxml::hyperlink(stack) {
        // ["w:hyperlink", "w:r", "w:t", "text"] -> hyperlink(text)
        latex::hyperlink(buf_writer, rels, hyperlink)?;
    } else if let Some(content) = ooxml::word_text(stack) {
        // ["w:r", "w:t", "text"] -> text
        write!(buf_writer, "{}", content)?;
    } else if let Some(content) = ooxml::math_text(stack) {
        // ["m:r", "m:t", "text"] -> text
        write!(buf_writer, "{}", content)?;
    } else if let Some(tag) = stack.last() {
        // ["w:p"] -> newline
        // ["w:bookmarkStart"] -> \hypertarget{anchor}{
        // ["m:d"] -> )
        // ["m:oMathPara"] -> $$
        // ["m:deg"] -> ]{
        // [("m:sub"/"m:sup"/"m:num"/"m:den"/"m:rad"/"m:bookmarkEnd")] -> }
        match tag {
            Tag::WParagraph => {
                writeln!(buf_writer)?;
                writeln!(buf_writer)?;
            }
            Tag::WBookmarkStart { anchor } => {
                write!(buf_writer, "\\hypertarget{{{anchor}}}{{")?;
            }
            Tag::MDelim => {
                write!(buf_writer, ")")?;
            }
            Tag::MoMathPara => {
                writeln!(buf_writer, "$$")?;
                if !*math_mode {
                    log::error!("Exiting Math Mode without entering Math Mode");
                }
                *math_mode = false;
            }
            Tag::MDeg => {
                write!(buf_writer, "]{{")?;
            }
            Tag::MSub | Tag::MSup | Tag::MNum | Tag::MDen | Tag::MRad | Tag::WBookmarkEnd => {
                write!(buf_writer, "}}")?;
            }
            Tag::MNaryPr => {
                if let Some(false) = nary_has_chr {
                    // m:naryPr with no m:chr within are treated as integrals
                    write!(buf_writer, "\\int")?;
                }
                *nary_has_chr = None;
            }
            _ => {}
        }
    }

    Ok(State::ClosedTag)
}

fn xml_event(
    buf_writer: &mut BufWriter<File>,
    stack: &Boo<Tag>,
    rels: &HashMap<String, String>,
    event: &XmlEvent,
    math_mode: &mut bool,
    nary_has_chr: &mut Option<bool>,
) -> std::io::Result<State> {
    match event {
        XmlEvent::StartElement {
            name, attributes, ..
        } => start_element(buf_writer, name, attributes, math_mode, nary_has_chr),
        XmlEvent::EndElement { .. } => {
            end_element(buf_writer, stack, rels, math_mode, nary_has_chr)
        }
        XmlEvent::Characters(content) => {
            log::debug!("Characters [Raw] {:?}", content);
            let content = escape(content, math_mode);
            log::debug!("Characters [Escaped] {:?}", &content);
            Ok(State::FoundContent(content))
        }
        XmlEvent::StartDocument { version, .. } => {
            log::debug!("StartDocument {version}");
            Ok(State::Happy)
        }
        XmlEvent::EndDocument => {
            log::debug!("EndDocument");
            Ok(State::End)
        }
        XmlEvent::Whitespace(content) => {
            log::debug!("Whitespace [{content}]");
            Ok(State::FoundContent(content.clone()))
        },
        event => {
            log::warn!("Unmatched Event: {event:?}");
            Ok(State::Happy)
        }
    }
}

pub fn document(
    parser: &mut EventReader<BufReader<File>>,
    buf_writer: &mut BufWriter<File>,
    rels: &HashMap<String, String>,
) -> std::io::Result<()> {
    let mut stack = Boo::default();
    let mut math_mode = false;
    let mut nary_has_chr = None;
    loop {
        match parser.next() {
            Ok(event) => match xml_event(
                buf_writer,
                &stack,
                rels,
                &event,
                &mut math_mode,
                &mut nary_has_chr,
            )? {
                State::OpenedTag(tag) => {
                    stack.push(tag);
                }
                State::ClosedTag => {
                    stack.pop();
                }
                State::FoundContent(content) => {
                    stack.push(Tag::Content(content));
                    let _ =
                        end_element(buf_writer, &stack, rels, &mut math_mode, &mut nary_has_chr)?;
                    stack.pop();
                }
                State::AttributesMissing | State::RelationshipMissing | State::Happy => {}
                State::End => break
            },
            Err(error) => {
                log::error!("Error: {error}");
                break;
            }
        }
    }
    Ok(())
}

fn escape(raw: &str, math_mode: &bool) -> String {
    let mut buf = String::new();
    for c in raw.chars() {
        match c {
            '∞' => buf.push_str("\\infty "),
            'π' => buf.push_str("\\pi "),
            '&' => buf.push_str("\\& "),
            '<' => {
                if *math_mode {
                    buf.push('<');
                } else {
                    buf.push_str("\\textless ");
                }
            }
            '>' => {
                if *math_mode {
                    buf.push('>');
                } else {
                    buf.push_str("\\textgreater ");
                }
            }
            '%' => buf.push_str("\\% "),
            '$' => buf.push_str("\\$ "),
            '{' => buf.push_str("\\{ "),
            '#' => buf.push_str("\\# "),
            '}' => buf.push_str("\\} "),
            '~' => buf.push_str("\\~{} "),
            '_' => buf.push_str("\\_ "),
            '±' => buf.push_str("\\pm "),
            '∓' => buf.push_str("\\mp "),
            c => buf.push(c),
        }
    }
    buf
}

#[cfg(test)]
mod test {
    #[test]
    fn blink_true_is_some() {
        let actual = super::blink(true);
        assert!(actual.is_some());
    }

    #[test]
    fn blink_false_is_none() {
        let actual = super::blink(false);
        assert!(actual.is_none());
    }

    #[test]
    fn unconditional_escape_works() {
        let input = "∞π&%${#}~_±∓ abrakadabra";
        let actual = super::escape(input, &false);
        let expected = "\\infty \\pi \\& \\% \\$ \\{ \\# \\} \\~{} \\_ \\pm \\mp  abrakadabra";
        assert_eq!(actual, expected);
    }

    #[test]
    fn escape_recognizes_math_mode() {
        let input = "<>";
        let on = "<>";
        let off = "\\textless \\textgreater ";
        assert_eq!(super::escape(input, &true), on);
        assert_eq!(super::escape(input, &false), off);
    }

    #[test]
    fn relationships_recognizes_missing_attributes() {
        let raw = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships>
    <Relationship Id="rId1" Target="styles.xml"/>
    <Relationship Id="rId2" Target="https://www.lipsum.com/"/>
    <Relationship           Target="settings.xml"/>
    <Relationship Id="rId3"/>
    <Relationship/>
    <Junk/>
</Relationships>
"#;
        let mut parser = xml::EventReader::new(std::io::BufReader::new(raw.as_bytes()));
        let rels = super::relationships(&mut parser);
        assert!(rels.is_ok());
        let rels = rels.unwrap();
        assert_eq!(rels.len(), 2);

        assert!(rels.contains_key("rId1"));
        assert_eq!(rels.get("rId1").unwrap(), "styles.xml");

        assert!(rels.contains_key("rId2"));
        assert_eq!(rels.get("rId2").unwrap(), "https://www.lipsum.com/");
    }

    #[test]
    fn relationships_recognizes_xml_error() {
        let raw = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
Relationships>
    <Relationship Id="rId1" Target="styles.xml"/>
    <Relationship Id="rId2" Target="https://www.lipsum.com/"/>
    <Relationship           Target="settings.xml"/>
    <Relationship Id="rId3"/>
    <Relationship/>
    <Junk/>
</Relationships>
"#;
        let mut parser = xml::EventReader::new(std::io::BufReader::new(raw.as_bytes()));
        let rels = super::relationships(&mut parser);
        assert!(rels.is_err());
    }
}