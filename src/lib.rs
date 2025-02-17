use std::collections::HashMap;

use xml::{
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(Default)]
pub struct Prysm {
    stack: Vec<Word>,
    rels: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Link {
    Anchor(String),
    Relationship(String),
}

#[derive(Debug)]
pub enum Word {
    BookmarkStart { anchor: String },
    BookmarkEnd,
    Document,
    Paragraph,
    Run,
    Text,
    Hyperlink(Link),
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

    pub fn document(&mut self, parser: &mut EventReader<std::io::BufReader<std::fs::File>>) {
        let mut bookmark_start = 0;
        let mut hyperlink = 0;
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let tag = normalize(&name);
                    log::debug!("StartElement '{tag}'",);
                    let element = match tag.as_str() {
                        "w:p" => Word::Paragraph,
                        "w:r" => Word::Run,
                        "w:t" => Word::Text,
                        "w:hyperlink" => {
                            hyperlink += 1;
                            if let Some(rel_id) =
                                attributes.iter().find(|&a| normalize(&a.name) == "r:id")
                            {
                                Word::Hyperlink(Link::Relationship(rel_id.value.clone()))
                            } else if let Some(anchor) = attributes
                                .iter()
                                .find(|&a| normalize(&a.name) == "w:anchor")
                            {
                                Word::Hyperlink(Link::Anchor(anchor.value.clone()))
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
                                Word::BookmarkStart {
                                    anchor: anchor.value.clone(),
                                }
                            } else {
                                log::warn!("Tag \"w:bookmarkStart\" #{bookmark_start} is missing attribute \"w:anchor\"");
                                Word::BookmarkStart { anchor: "".to_string() }
                            }
                        }
                        "w:bookmarkEnd" => Word::BookmarkEnd,
                        _ => {
                            log::warn!("Unknown");
                            Word::Unknown { id: tag }
                        }
                    };
                    self.stack.push(element);
                }
                Ok(XmlEvent::EndElement { name }) => {
                    let id = normalize(&name);
                    log::debug!("EndElement '{id}'",);
                    self.process();
                    self.stack.pop();
                }
                Ok(XmlEvent::Characters(content)) => {
                    log::debug!("Characters {:?}", &content);
                    self.stack.push(Word::Content(content));
                    self.process();
                    self.stack.pop();
                }
                Ok(XmlEvent::StartDocument { version, .. }) => {
                    log::debug!("StartDocument {version}");
                    self.stack.push(Word::Document);
                }
                Ok(XmlEvent::EndDocument) => {
                    log::debug!("EndDocument");
                    self.stack.pop();
                    break;
                }
                Ok(event) => {
                    log::warn!("Unmatched Event: {event:?}");
                }
                Err(e) => {
                    log::error!("Error: {e}");
                    break;
                }
            }
        }
    }

    pub fn process(&self) {
        // ["w:p", "w:hyperlink", "w:r", "w:t", "text"] -> hyperlink(text)
        // ["w:p", "w:r", "w:t", "text"] -> text
        // ["w:p"] -> newline
        // ["w:bookmarkStart"] -> \hypertarget{anchor}{
        // ["w:bookmarkEnd"] -> }
        log::debug!("Stack: {:?}", &self.stack);
        let n = self.stack.len();
        if n > 4 {
            if let Word::Content(content) = &self.stack[n - 1] {
                if let Word::Text = &self.stack[n - 2] {
                    if let Word::Run = &self.stack[n - 3] {
                        if let Word::Hyperlink(link) = &self.stack[n - 4] {
                            if let Word::Paragraph = &self.stack[n - 5] {
                                match link {
                                    Link::Anchor(anchor) => {
                                        print!("\\hyperlink {{ {anchor} }} {{ {content} }}")
                                    }
                                    Link::Relationship(rel_id) => {
                                        if let Some(url) = self.rels.get(rel_id) {
                                            print!("\\href {{ {url} }} {{ {content} }}");
                                        } else {
                                            log::error!("Hyperlink relies on a missing relationship {rel_id:?}");
                                            print!("{content}");
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }
        if n > 3 {
            if let Word::Content(content) = &self.stack[n - 1] {
                if let Word::Text = &self.stack[n - 2] {
                    if let Word::Run = &self.stack[n - 3] {
                        if let Word::Paragraph = &self.stack[n - 4] {
                            print!("{}", content);
                            return;
                        }
                    }
                }
            }
        }
        if n > 0 {
            if let Word::Paragraph = &self.stack[n - 1] {
                println!();
                println!();
            } else if let Word::BookmarkStart { anchor: name } = &self.stack[n - 1] {
                print!("\\hypertarget {{ {name} }} {{")
            } else if let Word::BookmarkEnd = &self.stack[n - 1] {
                print!("}}")
            }
        }
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
