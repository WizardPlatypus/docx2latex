use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{Error, EventReader, XmlEvent},
};

pub struct Element {
    pub name: OwnedName,
    pub attributes: Vec<OwnedAttribute>,
    pub children: Vec<Box<Element>>,
    pub content: Option<String>,
}

impl Element {
    pub fn new(name: OwnedName, attributes: Vec<OwnedAttribute>) -> Element {
        let children = vec![];
        Element {
            name,
            attributes,
            children,
            content: None,
        }
    }

    pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<Element, std::io::Error> {
        use std::io;

        let mut parser = EventReader::new(io::BufReader::new(std::fs::File::open(path)?));

        match parser.next() {
            Ok(XmlEvent::StartDocument { .. }) => {
                log::debug!("StartDocument");
            }
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        }

        Ok(match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                log::debug!("Found root: {}", &name.local_name);
                let mut element = Element::new(name, attributes);
                match element.populate(&mut parser) {
                    Ok(_) => {}
                    Err(_) => return Err(io::Error::from(io::ErrorKind::InvalidData)),
                }
                element
            }
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        })
    }

    pub fn populate(
        &mut self,
        parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
    ) -> Result<(), Error> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    log::debug!(
                        "StartElement {}:{}",
                        name.prefix_ref().unwrap_or(""),
                        &name.local_name
                    );
                    let mut child = Element::new(name, attributes);
                    child.populate(parser)?;
                    self.children.push(Box::new(child));
                }
                Ok(XmlEvent::EndElement { name }) => {
                    log::debug!(
                        "EndElement {}:{}",
                        name.prefix_ref().unwrap_or(""),
                        &name.local_name
                    );
                }
                Ok(XmlEvent::Characters(content)) => {
                    log::debug!("Characters {:?}", &content);
                    self.content = Some(content);
                }
                Ok(event) => {
                    log::debug!("Unmatched Event: {event:?}");
                }
                Err(e) => {
                    log::error!("Error: {e}");
                    return Err(e);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Prysm {
    stack: Vec<Word>,
}

#[derive(Debug)]
pub enum Word {
    Document,
    Paragraph,
    Run,
    Text,
    Content(String),
    Unknown { id: String }
}

impl Prysm {
    pub fn stream(
        &mut self,
        parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
    ) {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    let id = id(&name);
                    log::debug!("StartElement '{id}'",);
                    let element = if &id == "w:p" {
                        Word::Paragraph
                    } else if &id == "w:r" {
                        Word::Run
                    } else if &id == "w:t" {
                        Word::Text
                    } else {
                        log::warn!("Unknown");
                        Word::Unknown { id }
                    };
                    self.stack.push(element);
                }
                Ok(XmlEvent::EndElement { name }) => {
                    let id = id(&name);
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
                Ok(XmlEvent::StartDocument { version, ..}) => {
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
        // ["w:p", "w:r", "w:t", "text"] -> print(text)
        log::debug!("Stack: {:?}", &self.stack);
        let n = self.stack.len();
        if n > 3 {
            if let Word::Content(content) = &self.stack[n - 1] {
                if let Word::Text = &self.stack[n - 2] {
                    if let Word::Run = &self.stack[n - 3] {
                        if let Word::Paragraph = &self.stack[n - 4] {
                            print!("[LATEX] {}", content)
                        }
                    }
                }
            }
        } else {
            log::debug!("Stack too small: {n}")
        }
    }
}

pub fn id(raw: &OwnedName) -> String {
    let mut id = if let Some(prefix) = raw.prefix_ref() {
        prefix.to_string() + ":"
    } else {
        "".to_string()
    };
    id.push_str(&raw.local_name);
    id
}
