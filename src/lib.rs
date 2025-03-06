use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{ErrorKind, EventReader, XmlEvent},
};

pub mod peekaboo;

use peekaboo::Boo;

pub struct Prism {
    stack: Boo<Tag>,
    rels: HashMap<String, String>,
    fa: State,
    context: Context,
}

#[derive(Debug)]
pub enum Link {
    Anchor(String),
    Relationship(String),
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

#[derive(Debug)]
pub enum Tag {
    AGraphic,
    AGraphicData,
    PicPic,
    PicBlipFill,
    MoMathPara,
    MoMath,
    MDelim,
    MRad,
    MDeg,
    MRun,
    MText,
    MSub,
    MSup,
    MNary,
    MNaryPr,
    MFraction,
    MFunc,
    MFName,
    MNum,
    MDen,
    WPInline,
    WPAnchor,
    WBookmarkEnd,
    WDocument,
    WDrawing,
    WParagraph,
    WRun,
    WText,
    ABlip { rel: String },
    MChr { value: String },
    WBookmarkStart { anchor: String },
    WHyperlink(Link),
    Content(String),
    Unknown { id: String },
}

impl Tag {
    pub fn a_blip(&self) -> Option<&String> {
        if let Tag::ABlip { rel } = self {
            Some(rel)
        } else {
            None
        }
    }

    pub fn m_chr(&self) -> Option<&String> {
        if let Tag::MChr { value } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn w_bookmark_start(&self) -> Option<&String> {
        if let Tag::WBookmarkStart { anchor } = self {
            Some(anchor)
        } else {
            None
        }
    }

    pub fn w_hyperlink(&self) -> Option<&Link> {
        if let Tag::WHyperlink(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn content(&self) -> Option<&String> {
        if let Tag::Content(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

fn blink(value: bool) -> Option<()> {
    if value {
        Some(())
    } else {
        None
    }
}

pub enum InputError {
    MissingAttributes {
        id: String,
        missing: Vec<&'static str>,
    },
}

impl TryFrom<(OwnedName, Vec<OwnedAttribute>)> for Tag {
    type Error = InputError;

    fn try_from(value: (OwnedName, Vec<OwnedAttribute>)) -> Result<Self, Self::Error> {
        let (name, atts) = value;
        let id = normalize(&name);
        let tag = match id.as_str() {
            "a:graphic" => Tag::AGraphic,
            "a:graphicData" => Tag::AGraphicData,
            "a:blip" => {
                if let Some(rel_id) = atts.iter().find(|&a| normalize(&a.name) == "r:embed") {
                    Tag::ABlip {
                        rel: rel_id.value.clone(),
                    }
                } else {
                    return Err(InputError::MissingAttributes {
                        id,
                        missing: vec!["r:embed"],
                    });
                }
            }
            "pic:pic" => Tag::PicPic,
            "pic:blipFill" => Tag::PicBlipFill,
            "m:oMathPara" => Tag::MoMathPara,
            "m:oMath" => Tag::MoMath,
            "m:d" => Tag::MDelim,
            "m:rad" => Tag::MRad,
            "m:deg" => Tag::MDeg,
            "m:r" => Tag::MRun,
            "m:t" => Tag::MText,
            "m:sub" => Tag::MSub,
            "m:sup" => Tag::MSup,
            "m:nary" => Tag::MNary,
            "m:naryPr" => Tag::MNaryPr,
            "m:chr" => {
                if let Some(symbol) = atts.iter().find(|&a| normalize(&a.name) == "m:val") {
                    Tag::MChr {
                        value: symbol.value.clone(),
                    }
                } else {
                    return Err(InputError::MissingAttributes {
                        id,
                        missing: vec!["m:val"],
                    });
                }
            }
            "m:f" => Tag::MFraction,
            "m:func" => Tag::MFunc,
            "m:fName" => Tag::MFName,
            "m:num" => Tag::MNum,
            "m:den" => Tag::MDen,
            "wp:inline" => Tag::WPInline,
            "wp:anchor" => Tag::WPAnchor,
            "w:p" => Tag::WParagraph,
            "w:r" => Tag::WRun,
            "w:t" => Tag::WText,
            "w:hyperlink" => {
                if let Some(rel_id) = atts.iter().find(|&a| normalize(&a.name) == "r:id") {
                    Tag::WHyperlink(Link::Relationship(rel_id.value.clone()))
                } else if let Some(anchor) = atts.iter().find(|&a| normalize(&a.name) == "w:anchor")
                {
                    Tag::WHyperlink(Link::Anchor(anchor.value.clone()))
                } else {
                    return Err(InputError::MissingAttributes {
                        id,
                        missing: vec!["r:id", "w:anchor"],
                    });
                }
            }
            "w:bookmarkStart" => {
                let anchor = atts
                    .iter()
                    .find(|&a| normalize(&a.name) == "w:anchor")
                    .map(|a| a.value.clone())
                    .unwrap_or("".to_string());
                Tag::WBookmarkStart { anchor }
            }
            "w:bookmarkEnd" => Tag::WBookmarkEnd,
            "w:document" => Tag::WDocument,
            "w:drawing" => Tag::WDrawing,
            _ => Tag::Unknown { id },
        };
        Ok(tag)
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

    pub fn document(
        &mut self,
        parser: &mut EventReader<std::io::BufReader<std::fs::File>>,
        buf_writer: &mut std::io::BufWriter<std::fs::File>,
    ) -> std::io::Result<()> {
        loop {
            match parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let tag = match Tag::try_from((name, attributes)) {
                        Ok(ok) => ok,
                        Err(InputError::MissingAttributes { id, missing }) => {
                            if id == "m:chr" {
                                if let State::NaryPr = &self.fa {
                                    self.fa = State::Chr;
                                }
                            }
                            log::error!("Tag '{id}' is missing attributes: {missing:?}");
                            continue;
                        }
                    };
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
                }
                Ok(XmlEvent::EndElement { name }) => {
                    let id = normalize(&name);
                    // log::debug!("EndElement '{id}'",);
                    self.process(buf_writer)?;
                    if let Some(Tag::MNaryPr) = self.stack.pop() {
                        if let State::NaryPr = &self.fa {
                            // m:naryPr with no m:chr within are treated as integrals
                            write!(buf_writer, "\\int")?;
                        }
                        self.fa = State::None;
                    }
                }
                Ok(XmlEvent::Characters(content)) => {
                    log::debug!("Characters [Raw] {:?}", &content);
                    let content = escape(&self.context, content.as_str());
                    log::debug!("Characters [Escaped] {:?}", &content);
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
        // ["m:d"] -> )
        // [("m:sub"/"m:sup"/"m:num"/"m:den")] -> }
        // ["m:oMathPara"] -> $$
        // ["m:deg"] -> ]{
        // ["m:rad"] -> }
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
                                                "\\includegraphics[width=\\textwidth]{{{:?}}}",
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
            if let Some(hyperlink) = ooxml_hyperlink(&self.stack) {
                latex_hyperlink(buf_writer, &self.rels, hyperlink)?;
                return Ok(());
            }
        }
        if n > 2 {
            if let Tag::Content(content) = &self.stack[n - 1] {
                if let Tag::WText = &self.stack[n - 2] {
                    if let Tag::WRun = &self.stack[n - 3] {
                        write!(buf_writer, "{}", content)?;
                        return Ok(());
                    }
                } else if let Tag::MText = &self.stack[n - 2] {
                    if let Tag::MRun = &self.stack[n - 3] {
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
                write!(buf_writer, "\\hypertarget{{{name}}}{{")?;
            } else if let Tag::WBookmarkEnd = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            } else if let Tag::MDelim = &self.stack[n - 1] {
                write!(buf_writer, ")")?;
            } else if let Tag::MSub = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            } else if let Tag::MSup = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            } else if let Tag::MNum = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            } else if let Tag::MDen = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            } else if let Tag::MoMathPara = &self.stack[n - 1] {
                writeln!(buf_writer, "$$")?;
                self.context = Context::None;
            } else if let Tag::MDeg = &self.stack[n - 1] {
                write!(buf_writer, "]{{")?;
            } else if let Tag::MRad = &self.stack[n - 1] {
                write!(buf_writer, "}}")?;
            }
        }
        Ok(())
    }
}

// TODO: unit test candidate
pub fn normalize(raw: &OwnedName) -> String {
    let mut id = if let Some(prefix) = raw.prefix_ref() {
        prefix.to_string() + ":"
    } else {
        "".to_string()
    };
    id.push_str(&raw.local_name);
    id
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

fn ooxml_hyperlink(boo: &Boo<Tag>) -> Option<(&Link, &String)> {
    boo.reset();
    let content = boo.peek().content()?;
    blink(matches!(boo.peek(), Tag::WText))?;
    blink(matches!(boo.peek(), Tag::WRun))?;
    let link = boo.peek().w_hyperlink()?;
    Some((link, content))
}

fn latex_hyperlink(
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
