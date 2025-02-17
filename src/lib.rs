use std::{collections::HashMap, io::Write};

use xml::{
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

pub struct Prysm {
    stack: Vec<Tag>,
    rels: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Link {
    Anchor(String),
    Relationship(String),
}

#[derive(Debug)]
pub enum Tag {
    AGraphic,
    AGraphicData,
    ABlip { rel: String },
    PicPic,
    PicBlipFill,
    WPInline,
    WPAnchor,
    WBookmarkStart { anchor: String },
    WBookmarkEnd,
    WDocument,
    WDrawing,
    WParagraph,
    WRun,
    WText,
    WHyperlink(Link),
    Content(String),
    Unknown { id: String },
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

impl Prysm {
    pub fn new(rels: HashMap<String, String>) -> Prysm {
        Prysm {
            stack: vec![],
            rels,
        }
    }

    pub fn document(
        &mut self,
        parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
        buf_writer: &mut std::io::BufWriter<std::fs::File>,
    ) -> std::io::Result<()> {
        let mut bookmark_start = 0;
        let mut hyperlink = 0;
        let mut blip = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let tag = normalize(&name);
                    log::debug!("StartElement '{tag}'",);
                    let element = match tag.as_str() {
                        "a:graphic" => Tag::AGraphic,
                        "a:graphicData" => Tag::AGraphicData,
                        "a:blip" => {
                            blip += 1;
                            if let Some(rel_id) =
                                attributes.iter().find(|&a| normalize(&a.name) == "r:embed")
                            {
                                Tag::ABlip { rel: rel_id.value.clone() }
                            } else {
                                log::error!("\"a:blip\" #{blip} is missing atrribute \"r:embed\"");
                                break;
                            }
                        },
                        "pic:pic" => Tag::PicPic,
                        "pic:blipFill" => Tag::PicBlipFill,
                        "wp:inline" => Tag::WPInline,
                        "wp:anchor" => Tag::WPAnchor,
                        "w:p" => Tag::WParagraph,
                        "w:r" => Tag::WRun,
                        "w:t" => Tag::WText,
                        "w:hyperlink" => {
                            hyperlink += 1;
                            if let Some(rel_id) =
                                attributes.iter().find(|&a| normalize(&a.name) == "r:id")
                            {
                                Tag::WHyperlink(Link::Relationship(rel_id.value.clone()))
                            } else if let Some(anchor) = attributes
                                .iter()
                                .find(|&a| normalize(&a.name) == "w:anchor")
                            {
                                Tag::WHyperlink(Link::Anchor(anchor.value.clone()))
                            } else {
                                log::error!("\"w:hyperlink\" #{hyperlink} is missing both \"r:Id\" and \"w:anchor\"");
                                break;
                            }
                        }
                        "w:bookmarkStart" => {
                            bookmark_start += 1;
                            if let Some(anchor) = attributes
                                .iter()
                                .find(|&a| normalize(&a.name) == "w:anchor")
                            {
                                Tag::WBookmarkStart {
                                    anchor: anchor.value.clone(),
                                }
                            } else {
                                log::warn!("Tag \"w:bookmarkStart\" #{bookmark_start} is missing attribute \"w:anchor\"");
                                Tag::WBookmarkStart {
                                    anchor: "".to_string(),
                                }
                            }
                        }
                        "w:bookmarkEnd" => Tag::WBookmarkEnd,
                        "w:document" => Tag::WDocument,
                        "w:drawing" => Tag::WDrawing,
                        _ => {
                            log::warn!("Ignored: {tag:?}");
                            Tag::Unknown { id: tag }
                        }
                    };
                    self.stack.push(element);
                }
                Ok(XmlEvent::EndElement { name }) => {
                    let id = normalize(&name);
                    log::debug!("EndElement '{id}'",);
                    self.process(buf_writer)?;
                    self.stack.pop();
                }
                Ok(XmlEvent::Characters(content)) => {
                    log::debug!("Characters {:?}", &content);
                    self.stack.push(Tag::Content(content));
                    self.process(buf_writer)?;
                    self.stack.pop();
                }
                Ok(XmlEvent::StartDocument { version, .. }) => {
                    log::debug!("StartDocument {version}");
                    self.stack.push(Tag::WDocument);
                }
                Ok(XmlEvent::EndDocument) => {
                    log::debug!("EndDocument");
                    self.stack.pop();
                    break;
                }
                Ok(XmlEvent::Whitespace(_)) => continue,
                Ok(event) => {
                    log::warn!("Unmatched Event: {event:?}");
                }
                Err(e) => {
                    log::error!("Error: {e}");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn process(
        &mut self,
        buf_writer: &mut std::io::BufWriter<std::fs::File>,
    ) -> std::io::Result<()> {
        // ["w:drawing", ("wp:inline"/"wp:anchor"), "a:graphic", "a:graphicData", "pic:pic", "pic:blipFill", "a:blip"]
        // ["w:hyperlink", "w:r", "w:t", "text"] -> hyperlink(text)
        // ["w:r", "w:t", "text"] -> text
        // ["w:p"] -> newline
        // ["w:bookmarkStart"] -> \hypertarget{anchor}{
        // ["w:bookmarkEnd"] -> }
        log::debug!("Stack: {:?}", &self.stack);
        let n = self.stack.len();
        if n > 6 {
            if let Tag::ABlip { rel } = &self.stack[n - 1] {
                if let Tag::PicBlipFill = &self.stack[n - 2] {
                    if let Tag::PicPic = &self.stack[n - 3] {
                        if let Tag::AGraphicData = &self.stack[n - 4] {
                            if let Tag::AGraphic = &self.stack[n - 5] {
                                let switch;
                                if let Tag::WPInline = &self.stack[n - 6] {
                                    switch = true;
                                } else if let Tag::WPAnchor = &self.stack[n - 6] {
                                    switch = true;
                                } else {
                                    switch = false;
                                }
                                if switch {
                                    if let Tag::WDrawing = &self.stack[n - 7] {
                                        if let Some(path) = self.rels.get(rel) {
                                            let path = std::path::PathBuf::from(path);
                                            write!(
                                                buf_writer,
                                                "\\includegraphics {{ {:?} }}",
                                                path.file_stem()
                                                    .expect("Rels did not point to an image file")
                                            )?;
                                        } else {
                                            log::error!("\"a:blip\" relies on a relationship that does not exist: {:?}", rel);
                                        }
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if n > 3 {
            if let Tag::Content(content) = &self.stack[n - 1] {
                if let Tag::WText = &self.stack[n - 2] {
                    if let Tag::WRun = &self.stack[n - 3] {
                        if let Tag::WHyperlink(link) = &self.stack[n - 4] {
                            match link {
                                Link::Anchor(anchor) => {
                                    write!(
                                        buf_writer,
                                        "\\hyperlink {{ {anchor} }} {{ {content} }}"
                                    )?;
                                }
                                Link::Relationship(rel_id) => {
                                    if let Some(url) = self.rels.get(rel_id) {
                                        write!(buf_writer, "\\href {{ {url} }} {{ {content} }}")?;
                                    } else {
                                        log::error!(
                                            "Hyperlink relies on a missing relationship {rel_id:?}"
                                        );
                                        write!(buf_writer, "{content}")?;
                                    }
                                }
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
        if n > 2 {
            if let Tag::Content(content) = &self.stack[n - 1] {
                if let Tag::WText = &self.stack[n - 2] {
                    if let Tag::WRun = &self.stack[n - 3] {
                        write!(buf_writer, "{}", content)?;
                        return Ok(());
                    }
                }
            }
        }
        if n > 0 {
            if let Tag::WParagraph = &self.stack[n - 1] {
                writeln!(buf_writer)?;
                writeln!(buf_writer)?;
            } else if let Tag::WBookmarkStart { anchor: name } = &self.stack[n - 1] {
                write!(buf_writer, "\\hypertarget {{ {name} }} {{")?;
            } else if let Tag::WBookmarkEnd = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            }
        }
        Ok(())
    }
}

pub fn normalize(raw: &OwnedName) -> String {
    let mut id = if let Some(prefix) = raw.prefix_ref() {
        prefix.to_string() + ":"
    } else {
        "".to_string()
    };
    id.push_str(&raw.local_name);
    id
}
