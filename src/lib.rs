use std::fs::File;

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
                    log::info!("Unmatched Event: {event:?}");
                }
                Err(e) => {
                    log::error!("Error: {e}");
                    return Err(e);
                }
            }
        }
    }
}
