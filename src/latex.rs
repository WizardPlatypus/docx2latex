use super::{Link, State};
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

pub fn hyperlink<W: Write>(
    buf_writer: &mut BufWriter<W>,
    rels: &HashMap<String, String>,
    hyperlink: (&Link, &String),
) -> std::io::Result<State> {
    let (link, content) = hyperlink;
    match link {
        Link::Anchor(anchor) => {
            write!(buf_writer, "\\hyperlink{{{anchor}}}{{{content}}}")?;
            Ok(State::Happy)
        }
        Link::Relationship(rel_id) => {
            if let Some(url) = rels.get(rel_id) {
                write!(buf_writer, "\\href{{{url}}}{{{content}}}")?;
                Ok(State::Happy)
            } else {
                log::error!("Hyperlink relies on a missing relationship {rel_id:?}");
                write!(buf_writer, "{content}")?;
                Ok(State::RelationshipMissing)
            }
        }
    }
}

pub fn drawing<W: Write>(
    buf_writer: &mut BufWriter<W>,
    rels: &HashMap<String, String>,
    rel: &String,
) -> std::io::Result<State> {
    if let Some(path) = rels.get(rel) {
        let path = std::path::PathBuf::from(path);
        write!(
            buf_writer,
            "\\includegraphics[width=\\textwidth]{{{:?}}}",
            path.file_stem()
                .expect("Rels did not point to an image file")
        )?;
        Ok(State::Happy)
    } else {
        log::error!(
            "Drawing relies on a relationship that does not exist: {:?}",
            rel
        );
        Ok(State::RelationshipMissing)
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    #[test]
    fn hyperlink_with_anchor_works() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let rels = super::HashMap::new();
        let link = super::Link::Anchor("Anchor".to_string());
        let content = "Content".to_string();

        let state = super::hyperlink(&mut buf_writer, &rels, (&link, &content));
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state, super::State::Happy);

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "\\hyperlink{Anchor}{Content}");
    }

    #[test]
    fn hyperlink_with_present_relationship_works() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let mut rels = super::HashMap::new();
        rels.insert("TestKey".to_string(), "TestValue".to_string());
        let link = super::Link::Relationship("TestKey".to_string());
        let content = "Content".to_string();

        let state = super::hyperlink(&mut buf_writer, &rels, (&link, &content));
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state, super::State::Happy);


        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "\\href{TestValue}{Content}");
    }

    #[test]
    fn hyperlink_recognizes_missing_relationship() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let rels = super::HashMap::new();
        let link = super::Link::Relationship("TestKey".to_string());
        let content = "Content".to_string();

        let state = super::hyperlink(&mut buf_writer, &rels, (&link, &content));
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state, super::State::RelationshipMissing);


        let mut written = String::new();
        assert!(buf_writer.buffer().read_to_string(&mut written).is_ok());
        assert_eq!(written, "Content");
    }

    #[test]
    fn drawing_with_present_relationship_works() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let mut rels = super::HashMap::new();
        rels.insert("Key".to_string(), "value.test".to_string());

        let state = super::drawing(&mut buf_writer, &rels, &"Key".to_string());
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state, super::State::Happy);

        let mut written = String::new();
        buf_writer.buffer().read_to_string(&mut written).unwrap();
        assert_eq!(written, "\\includegraphics[width=\\textwidth]{\"value\"}");
    }

    #[test]
    fn drawing_recognizes_missing_relationship() {
        let mut buf_writer = super::BufWriter::new(Vec::new());
        let rels = super::HashMap::new();

        let state = super::drawing(&mut buf_writer, &rels, &"Key".to_string());
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state, super::State::RelationshipMissing);

        let mut written = String::new();
        assert!(buf_writer.buffer().read_to_string(&mut written).is_ok());
        assert_eq!(written, "");
    }
}
