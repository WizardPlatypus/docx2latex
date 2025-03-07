use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
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

fn start_element<W: Write>(
    buf_writer: &mut BufWriter<W>,
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
        }
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
        }
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

fn end_element<W: Write>(
    buf_writer: &mut BufWriter<W>,
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

fn xml_event<W: Write>(
    buf_writer: &mut BufWriter<W>,
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
        }
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
                State::End => break,
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
    use std::{
        collections::HashMap,
        io::{Read, Write},
    };

    use xml::namespace::NS_NO_PREFIX;

    use crate::{peekaboo::Boo, tag};

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

    #[test]
    fn start_element_works_with_simple_tags() {
        use super::Tag::*;
        let input = vec![
            // MoMathPara,
            MDelim, MRad, MDeg, MSub, MSup,
            // MNaryPr,
            // MChr { value: "X".to_string() },
            MFraction, MNum, MDen,
        ];
        let output = ["(", "\\sqrt", "[", "_{", "^{", "\\frac", "{", "{"];

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            let mut buf_writer = super::BufWriter::new(Vec::new());
            let mut math_mode = false;
            let mut nary_has_chr = None;

            let (name, attributes) = input[i].to_owned().unwrap();
            let state = super::start_element(
                &mut buf_writer,
                &name,
                &attributes,
                &mut math_mode,
                &mut nary_has_chr,
            );
            assert!(state.is_ok());
            let state = state.unwrap();
            assert!(matches!(state, super::State::OpenedTag(_)));
            if let super::State::OpenedTag(tag) = state {
                assert_eq!(tag, input[i]);
            }

            let mut written = String::new();
            buf_writer.buffer().read_to_string(&mut written).unwrap();
            assert_eq!(written, output[i]);
        }
    }

    #[test]
    fn start_element_recognizes_missing_attributes() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let mut math_mode = false;
        let mut nary_has_chr = None;

        let name = super::tag::owned_name("a", "blip");
        let state = super::start_element(
            &mut buf_writer,
            &name,
            &vec![],
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(matches!(state, super::State::AttributesMissing));
    }

    #[test]
    fn start_element_recognizes_momathpara() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let mut math_mode = false;
        let mut nary_has_chr = None;

        let name = super::tag::owned_name("m", "oMathPara");

        let state = super::start_element(
            &mut buf_writer,
            &name,
            &vec![],
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(matches!(state, super::State::OpenedTag(_)));
        if let super::State::OpenedTag(tag) = state {
            assert_eq!(tag, super::Tag::MoMathPara);
        }

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "$$");

        buf_writer = super::BufWriter::new(Vec::new());
        math_mode = true;
        let state = super::start_element(
            &mut buf_writer,
            &name,
            &vec![],
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(matches!(state, super::State::OpenedTag(_)));
        if let super::State::OpenedTag(tag) = state {
            assert_eq!(tag, super::Tag::MoMathPara);
        }

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "");
    }

    #[test]
    fn start_element_recognizes_mnarypr() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let mut math_mode = false;
        let mut nary_has_chr = None;

        let name = super::tag::owned_name("m", "naryPr");

        let state = super::start_element(
            &mut buf_writer,
            &name,
            &vec![],
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(matches!(state, super::State::OpenedTag(_)));
        if let super::State::OpenedTag(tag) = state {
            assert_eq!(tag, super::Tag::MNaryPr);
        }

        assert_eq!(nary_has_chr, Some(false));

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "");

        let mut buf_writer = super::BufWriter::new(Vec::new());
        let state = super::start_element(
            &mut buf_writer,
            &name,
            &vec![],
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(matches!(state, super::State::OpenedTag(_)));
        if let super::State::OpenedTag(tag) = state {
            assert_eq!(tag, super::Tag::MNaryPr);
        }

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "");
    }

    #[test]
    fn start_element_recognizes_mchr() {
        let mut math_mode = false;

        let name = super::tag::owned_name("m", "chr");

        let input = ["⋀", "⋁", "⋂", "⋃", "∐", "∏", "∑", "∮"];
        let output = [
            "\\bigwedge",
            "\\bigvee",
            "\\bigcap",
            "\\bigcup",
            "\\coprod",
            "\\prod",
            "\\sum",
            "\\oint",
        ];

        let mut nary_has_chr;

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            let mut buf_writer = super::BufWriter::new(Vec::new());
            let attr = super::tag::owned_attr("m", "val", input[i]);

            nary_has_chr = Some(false);

            let state = super::start_element(
                &mut buf_writer,
                &name,
                &vec![attr],
                &mut math_mode,
                &mut nary_has_chr,
            );

            assert!(state.is_ok());
            let state = state.unwrap();
            assert!(matches!(
                state,
                super::State::OpenedTag(super::Tag::MChr { value: _ })
            ));
            if let super::State::OpenedTag(tag) = state {
                assert_eq!(
                    tag,
                    super::Tag::MChr {
                        value: input[i].to_string()
                    }
                );
            }
            assert_eq!(nary_has_chr, Some(true));

            let mut written = String::new();
            buf_writer.buffer().read_to_string(&mut written).unwrap();
            assert_eq!(written, output[i]);

            let mut buf_writer = super::BufWriter::new(Vec::new());
            let attr = super::tag::owned_attr("m", "val", input[i]);
            let state = super::start_element(
                &mut buf_writer,
                &name,
                &vec![attr],
                &mut math_mode,
                &mut nary_has_chr,
            );

            assert!(state.is_ok());
            let state = state.unwrap();
            assert!(matches!(
                state,
                super::State::OpenedTag(super::Tag::MChr { value: _ })
            ));
            if let super::State::OpenedTag(tag) = state {
                assert_eq!(
                    tag,
                    super::Tag::MChr {
                        value: input[i].to_string()
                    }
                );
            }
            assert_eq!(nary_has_chr, Some(true));

            let mut written = String::new();
            buf_writer.buffer().read_to_string(&mut written).unwrap();
            assert_eq!(written, output[i]);
        }
    }

    fn drain<W: std::io::Write>(buf_writer: &mut std::io::BufWriter<W>) -> std::io::Result<String> {
        let mut s = String::new();
        buf_writer.buffer().read_to_string(&mut s)?;
        buf_writer.flush()?;
        Ok(s)
    }

    #[test]
    fn end_element_recognizes_lonely_tags() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let rels = HashMap::<String, String>::default();
        let mut stack = Boo::default();
        let mut math_mode = false;
        let mut nary_has_chr = None;

        //// w:p

        stack.push(super::Tag::WParagraph);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "\n\n");
        stack.pop();

        //// w:bookmarkStart

        stack.push(super::Tag::WBookmarkStart {
            anchor: "Anchor".to_string(),
        });
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "\\hypertarget{Anchor}{");
        stack.pop();

        //// m:d

        stack.push(super::Tag::MDelim);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), ")");
        stack.pop();

        //// m:oMathPara math_mode = false

        math_mode = false;
        stack.push(super::Tag::MoMathPara);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);
        assert!(!math_mode);

        assert_eq!(drain(&mut buf_writer).unwrap(), "$$\n");

        //// m:oMathPara math_mode = true

        math_mode = true;
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);
        assert!(!math_mode);

        assert_eq!(drain(&mut buf_writer).unwrap(), "$$\n");
        stack.pop();

        //// m:deg

        stack.push(super::Tag::MDeg);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "]{");
        stack.pop();

        //// m:sub

        stack.push(super::Tag::MSub);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// m:sup

        stack.push(super::Tag::MSup);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// m:num

        stack.push(super::Tag::MNum);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// m:den

        stack.push(super::Tag::MDen);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// m:rad

        stack.push(super::Tag::MRad);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// w:bookmarkEnd

        stack.push(super::Tag::WBookmarkEnd);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        assert_eq!(drain(&mut buf_writer).unwrap(), "}");
        stack.pop();

        //// m:naryPr nary_has_chr = Some(false)

        nary_has_chr = Some(false);
        stack.push(super::Tag::MNaryPr);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);
        assert!(nary_has_chr.is_none());

        assert_eq!(drain(&mut buf_writer).unwrap(), "\\int");
        stack.pop();

        //// m:naryPr nary_has_chr = Some(true)

        nary_has_chr = Some(true);
        stack.push(super::Tag::MNaryPr);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);
        assert!(nary_has_chr.is_none());

        assert_eq!(drain(&mut buf_writer).unwrap(), "");
        stack.pop();

        //// m:naryPr nary_has_chr = None

        nary_has_chr = None;
        stack.push(super::Tag::MNaryPr);
        let state = super::end_element(
            &mut buf_writer,
            &stack,
            &rels,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);
        assert!(nary_has_chr.is_none());

        assert_eq!(drain(&mut buf_writer).unwrap(), "");
        stack.pop();
    }

    #[test]
    fn xml_event_produces_correct_state() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let rels = HashMap::<String, String>::default();
        let stack = Boo::default();
        let mut math_mode = false;
        let mut nary_has_chr = None;

        let namespace = xml::namespace::Namespace::empty();

        //// StartElement

        let event = xml::reader::XmlEvent::StartElement {
            name: tag::owned_name("docx2latex", "test"),
            attributes: vec![],
            namespace,
        };
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(
            state.unwrap(),
            super::State::OpenedTag(super::Tag::Unknown {
                id: "docx2latex:test".to_string()
            })
        );

        //// EndElement

        let event = xml::reader::XmlEvent::EndElement {
            name: tag::owned_name("docx2latex", "test"),
        };
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::ClosedTag);

        //// Characters

        let event = xml::reader::XmlEvent::Characters("Characters".to_string());
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(
            state.unwrap(),
            super::State::FoundContent("Characters".to_string())
        );

        //// StartDocument

        let event = xml::reader::XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: "ignored".to_string(),
            standalone: None,
        };
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::Happy);

        //// EndDocument

        let event = xml::reader::XmlEvent::EndDocument;
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::End);

        //// Whitespace

        let event = xml::reader::XmlEvent::Whitespace(" ".to_string());
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::FoundContent(" ".to_string()));

        //// other

        let event = xml::reader::XmlEvent::Comment("Ignored".to_string());
        let state = super::xml_event(
            &mut buf_writer,
            &stack,
            &rels,
            &event,
            &mut math_mode,
            &mut nary_has_chr,
        );
        assert!(state.is_ok());
        assert_eq!(state.unwrap(), super::State::Happy);
    }
}
