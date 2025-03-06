use super::Link;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

pub fn hyperlink(
    buf_writer: &mut BufWriter<File>,
    rels: &HashMap<String, String>,
    hyperlink: (&Link, &String),
) -> std::io::Result<()> {
    let (link, content) = hyperlink;
    match link {
        Link::Anchor(anchor) => {
            write!(buf_writer, "\\hyperlink{{{anchor}}}{{{content}}}")?;
        }
        Link::Relationship(rel_id) => {
            if let Some(url) = rels.get(rel_id) {
                write!(buf_writer, "\\href{{{url}}}{{{content}}}")?;
            } else {
                log::error!("Hyperlink relies on a missing relationship {rel_id:?}");
                write!(buf_writer, "{content}")?;
            }
        }
    }
    Ok(())
}

pub fn drawing(
    buf_writer: &mut BufWriter<File>,
    rels: &HashMap<String, String>,
    rel: &String,
) -> std::io::Result<()> {
    if let Some(path) = rels.get(rel) {
        let path = std::path::PathBuf::from(path);
        write!(
            buf_writer,
            "\\includegraphics[width=\\textwidth]{{{:?}}}",
            path.file_stem()
                .expect("Rels did not point to an image file")
        )?;
    } else {
        log::error!(
            "Drawing relies on a relationship that does not exist: {:?}",
            rel
        );
    }
    Ok(())
}
