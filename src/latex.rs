use super::{Link, State};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

pub fn hyperlink(
    buf_writer: &mut BufWriter<File>,
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

pub fn drawing(
    buf_writer: &mut BufWriter<File>,
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
