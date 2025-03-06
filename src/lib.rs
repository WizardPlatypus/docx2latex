use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
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

pub struct Prism {
    stack: Boo<Tag>,
    rels: HashMap<String, String>,
    fa: State,
    context: Context,
}

enum State {
    None,
    NaryPr,
    Chr,
}

enum Context {
    None,
    Math,
}

fn blink(value: bool) -> Option<()> {
    if value {
        Some(())
    } else {
        None
    }
}

pub fn relationships(
    parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
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

impl Prism {
    pub fn new(rels: HashMap<String, String>) -> Prism {
        Prism {
            stack: vec![].into(),
            rels,
            fa: State::None,
            context: Context::None,
        }
    }

    fn start_element(
        &mut self,
        buf_writer: &mut BufWriter<File>,
        name: &OwnedName,
        attributes: &Vec<OwnedAttribute>,
    ) -> std::io::Result<()> {
        let tag = Tag::try_from((name, attributes));

        if let Err(InputError::MissingAttributes { id, missing }) = &tag {
            if id == "m:chr" {
                if let State::NaryPr = &self.fa {
                    self.fa = State::Chr;
                }
            }
            log::error!("Tag '{id}' is missing attributes: {missing:?}");
            return Ok(());
        }

        let tag = tag.expect("Error case was handled");

        match &tag {
            Tag::MoMathPara => write!(buf_writer, "$$")?,
            Tag::MDelim => write!(buf_writer, "(")?,
            Tag::MRad => write!(buf_writer, "\\sqrt")?,
            Tag::MDeg => write!(buf_writer, "[")?,
            Tag::MSub => write!(buf_writer, "_{{")?,
            Tag::MSup => write!(buf_writer, "^{{")?,
            Tag::MNaryPr => self.fa = State::NaryPr,
            Tag::MChr { value } => {
                if let State::NaryPr = &self.fa {
                    self.fa = State::Chr;
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

        self.stack.push(tag);

        Ok(())
    }

    fn end_element(&mut self, buf_writer: &mut BufWriter<File>) -> std::io::Result<()> {
        log::debug!("Stack: {:?}", &self.stack);

        if let Some(rel) = ooxml::drawing(&self.stack) {
            // ["w:drawing", ("wp:inline"/"wp:anchor"), "a:graphic", "a:graphicData", "pic:pic", "pic:blipFill", "a:blip"]
            latex::drawing(buf_writer, &self.rels, rel)?;
        } else if let Some(hyperlink) = ooxml::hyperlink(&self.stack) {
            // ["w:hyperlink", "w:r", "w:t", "text"] -> hyperlink(text)
            latex::hyperlink(buf_writer, &self.rels, hyperlink)?;
        } else if let Some(content) = ooxml::word_text(&self.stack) {
            // ["w:r", "w:t", "text"] -> text
            write!(buf_writer, "{}", content)?;
        } else if let Some(content) = ooxml::math_text(&self.stack) {
            // ["m:r", "m:t", "text"] -> text
            write!(buf_writer, "{}", content)?;
        } else if let Some(tag) = self.stack.last() {
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
                    self.context = Context::None;
                }
                Tag::MDeg => {
                    write!(buf_writer, "]{{")?;
                }
                Tag::MSub | Tag::MSup | Tag::MNum | Tag::MDen | Tag::MRad | Tag::WBookmarkEnd => {
                    write!(buf_writer, "}}")?;
                }
                Tag::MNaryPr => {
                    if let State::NaryPr = &self.fa {
                        // m:naryPr with no m:chr within are treated as integrals
                        write!(buf_writer, "\\int")?;
                    }
                    self.fa = State::None;
                }
                _ => {}
            }
        }

        self.stack.pop();

        Ok(())
    }

    fn xml_event(
        &mut self,
        buf_writer: &mut BufWriter<File>,
        event: &XmlEvent,
    ) -> std::io::Result<()> {
        match event {
            XmlEvent::StartElement {
                name, attributes, ..
            } => self.start_element(buf_writer, name, attributes),
            XmlEvent::EndElement { .. } => self.end_element(buf_writer),
            XmlEvent::Characters(content) => {
                log::debug!("Characters [Raw] {:?}", &content);
                let content = escape(&self.context, content.as_str());
                log::debug!("Characters [Escaped] {:?}", &content);
                self.stack.push(Tag::Content(content));
                self.end_element(buf_writer)
            }
            XmlEvent::StartDocument { version, .. } => {
                log::debug!("StartDocument {version}");
                Ok(())
            }
            XmlEvent::EndDocument => {
                log::debug!("EndDocument");
                Ok(())
            }
            XmlEvent::Whitespace(_) => Ok(()),
            event => {
                log::warn!("Unmatched Event: {event:?}");
                Ok(())
            }
        }
    }

    pub fn document(
        &mut self,
        parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
        buf_writer: &mut std::io::BufWriter<std::fs::File>,
    ) -> std::io::Result<()> {
        loop {
            match parser.next() {
                Ok(event) => self.xml_event(buf_writer, &event)?,
                Err(error) => {
                    log::error!("Error: {error}");
                    break;
                }
            }
        }
        Ok(())
    }
}

// TODO: unit test candidate
fn escape(cxt: &Context, raw: &str) -> String {
    let mut buf = String::new();
    for c in raw.chars() {
        match c {
            '∞' => buf.push_str("\\infty "),
            'π' => buf.push_str("\\pi "),
            '&' => buf.push_str("\\& "),
            '<' => {
                if let Context::Math = cxt {
                    buf.push('<');
                } else {
                    buf.push_str("\\textless");
                }
            }
            '>' => {
                if let Context::Math = cxt {
                    buf.push('>');
                } else {
                    buf.push_str("\\textgreater");
                }
            }
            '%' => buf.push_str("\\% "),
            '$' => buf.push_str("\\$ "),
            '{' => buf.push_str("\\{ "),
            '#' => buf.push_str("\\# "),
            '}' => buf.push_str("\\} "),
            '~' => buf.push_str("\\~{} "),
            '_' => buf.push_str("\\_"),
            '±' => buf.push_str("\\pm"),
            '∓' => buf.push_str("\\mp"),
            c => buf.push(c),
        }
    }
    buf
}
