use xml::{attribute::OwnedAttribute, name::OwnedName};

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Link {
    Anchor(String),
    Relationship(String),
}

impl Tag {
    pub fn a_blip(&self) -> Option<&String> {
        if let Tag::ABlip { rel } = self {
            Some(rel)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn m_chr(&self) -> Option<&String> {
        if let Tag::MChr { value } = self {
            Some(value)
        } else {
            None
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn to_owned(&self) -> Option<(OwnedName, Vec<OwnedAttribute>)> {
        use Tag::*;

        let ok = match &self {
            AGraphic => (owned_name("a", "graphic"), vec![]),
            AGraphicData => (owned_name("a", "graphicData"), vec![]),
            PicPic => (owned_name("pic", "pic"), vec![]),
            PicBlipFill => (owned_name("pic", "blipFill"), vec![]),
            MoMathPara => (owned_name("m", "oMathPara"), vec![]),
            MoMath => (owned_name("m", "oMath"), vec![]),
            MDelim => (owned_name("m", "d"), vec![]),
            MRad => (owned_name("m", "rad"), vec![]),
            MDeg => (owned_name("m", "deg"), vec![]),
            MRun => (owned_name("m", "r"), vec![]),
            MText => (owned_name("m", "t"), vec![]),
            MSub => (owned_name("m", "sub"), vec![]),
            MSup => (owned_name("m", "sup"), vec![]),
            MNary => (owned_name("m", "nary"), vec![]),
            MNaryPr => (owned_name("m", "naryPr"), vec![]),
            MFraction => (owned_name("m", "f"), vec![]),
            MFunc => (owned_name("m", "func"), vec![]),
            MFName => (owned_name("m", "fName"), vec![]),
            MNum => (owned_name("m", "num"), vec![]),
            MDen => (owned_name("m", "den"), vec![]),
            WPInline => (owned_name("wp", "inline"), vec![]),
            WPAnchor => (owned_name("wp", "anchor"), vec![]),
            WBookmarkEnd => (owned_name("w", "bookmarkEnd"), vec![]),
            WDrawing => (owned_name("w", "drawing"), vec![]),
            WParagraph => (owned_name("w", "p"), vec![]),
            WRun => (owned_name("w", "r"), vec![]),
            WText => (owned_name("w", "t"), vec![]),
            ABlip { rel } => (owned_name("a", "blip"), vec![owned_attr("r", "id", rel)]),
            MChr { value } => (owned_name("m", "chr"), vec![owned_attr("m", "val", value)]),
            WBookmarkStart { anchor } => (
                owned_name("w", "bookmarkStart"),
                vec![owned_attr("w", "anchor", anchor)],
            ),
            WHyperlink(link) => (
                owned_name("w", "hyperlink"),
                vec![match link {
                    Link::Anchor(anchor) => owned_attr("w", "anchor", anchor),
                    Link::Relationship(rel) => owned_attr("r", "id", rel),
                }],
            ),
            Content(content) => (
                owned_name("docx2latex", "content"),
                vec![owned_attr("docx2latex", "characters", content)],
            ),
            Unknown { id } => (
                owned_name("docx2latex", "unknown"),
                vec![owned_attr("docx2latex", "id", id)],
            ),
        };
        Some(ok)
    }
}

#[allow(dead_code)]
pub fn owned_name(prefix: &str, local: &str) -> OwnedName {
    OwnedName {
        local_name: local.to_string(),
        namespace: None,
        prefix: Some(prefix.to_string()),
    }
}

#[allow(dead_code)]
pub fn owned_attr(prefix: &str, local: &str, value: &str) -> OwnedAttribute {
    OwnedAttribute {
        name: owned_name(prefix, local),
        value: value.to_string(),
    }
}

impl TryFrom<(&OwnedName, &Vec<OwnedAttribute>)> for Tag {
    type Error = InputError;

    fn try_from(value: (&OwnedName, &Vec<OwnedAttribute>)) -> Result<Self, Self::Error> {
        let (name, atts) = value;
        let id = normalize(name);
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
            "w:drawing" => Tag::WDrawing,
            _ => Tag::Unknown { id },
        };
        Ok(tag)
    }
}

#[derive(Debug)]
pub enum InputError {
    MissingAttributes {
        id: String,
        missing: Vec<&'static str>,
    },
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

#[cfg(test)]
mod test {
    use rstest::rstest;
    use xml::{attribute::OwnedAttribute, name::OwnedName};

    use super::*;

    #[test]
    fn owned_name_works() {
        let name = owned_name("prefix", "local");
        assert!(name.prefix_ref().is_some());
        assert_eq!(name.prefix_ref().unwrap(), "prefix");
        assert_eq!(name.local_name, "local");
    }

    #[test]
    fn owned_attr_works() {
        let attr = owned_attr("prefix", "local", "value");
        assert!(attr.name.prefix_ref().is_some());
        assert_eq!(attr.name.prefix_ref().unwrap(), "prefix");
        assert_eq!(attr.name.local_name, "local");
        assert_eq!(attr.value, "value");
    }

    #[rstest]
    #[case(Tag::AGraphic, (owned_name("a", "graphic"), vec![]))]
    #[case(Tag::AGraphicData, (owned_name("a", "graphicData"), vec![]))]
    #[case(Tag::PicPic, (owned_name("pic", "pic"), vec![]))]
    #[case(Tag::PicBlipFill, (owned_name("pic", "blipFill"), vec![]))]
    #[case(Tag::MoMathPara, (owned_name("m", "oMathPara"), vec![]))]
    #[case(Tag::MoMath, (owned_name("m", "oMath"), vec![]))]
    #[case(Tag::MDelim, (owned_name("m", "d"), vec![]))]
    #[case(Tag::MRad, (owned_name("m", "rad"), vec![]))]
    #[case(Tag::MDeg, (owned_name("m", "deg"), vec![]))]
    #[case(Tag::MRun, (owned_name("m", "r"), vec![]))]
    #[case(Tag::MText, (owned_name("m", "t"), vec![]))]
    #[case(Tag::MSub, (owned_name("m", "sub"), vec![]))]
    #[case(Tag::MSup, (owned_name("m", "sup"), vec![]))]
    #[case(Tag::MNary, (owned_name("m", "nary"), vec![]))]
    #[case(Tag::MNaryPr, (owned_name("m", "naryPr"), vec![]))]
    #[case(Tag::MFraction, (owned_name("m", "f"), vec![]))]
    #[case(Tag::MFunc, (owned_name("m", "func"), vec![]))]
    #[case(Tag::MFName, (owned_name("m", "fName"), vec![]))]
    #[case(Tag::MNum, (owned_name("m", "num"), vec![]))]
    #[case(Tag::MDen, (owned_name("m", "den"), vec![]))]
    #[case(Tag::WPInline, (owned_name("wp", "inline"), vec![]))]
    #[case(Tag::WPAnchor, (owned_name("wp", "anchor"), vec![]))]
    #[case(Tag::WBookmarkEnd, (owned_name("w", "bookmarkEnd"), vec![]))]
    #[case(Tag::WDrawing, (owned_name("w", "drawing"), vec![]))]
    #[case(Tag::WParagraph, (owned_name("w", "p"), vec![]))]
    #[case(Tag::WRun, (owned_name("w", "r"), vec![]))]
    #[case(Tag::WText, (owned_name("w", "t"), vec![]))]
    #[case(Tag::ABlip { rel: "RelId".to_string() }, (owned_name("a", "blip"), vec![owned_attr("r", "id", "RelId")]))]
    #[case(Tag::MChr { value: "X".to_string() }, (owned_name("m", "chr"), vec![owned_attr("m", "val", "X")]))]
    #[case(Tag::WBookmarkStart { anchor: "Anchor".to_string() }, (owned_name("w", "bookmarkStart"), vec![owned_attr("w", "anchor", "Anchor")]))]
    #[case(Tag::WHyperlink(Link::Anchor("Anchor".to_string())), (owned_name("w", "hyperlink"), vec![owned_attr("w", "anchor", "Anchor")]))]
    #[case(Tag::WHyperlink(Link::Relationship("RelId".to_string())), (owned_name("w", "hyperlink"), vec![owned_attr("r", "id", "RelId")]))]
    fn to_owned_works(#[case] input: Tag, #[case] output: (OwnedName, Vec<OwnedAttribute>)) {
        let (e_name, e_attrs) = &output;
        let owned = input.to_owned();

        assert!(owned.is_some());
        let (a_name, a_attrs) = owned.unwrap();

        assert_eq!(e_name.local_name, a_name.local_name);
        assert_eq!(e_name.prefix, a_name.prefix);

        assert_eq!(e_attrs.len(), a_attrs.len());
        for j in 0..e_attrs.len() {
            let e_attr = &e_attrs[j];
            let a_attr = &a_attrs[j];

            assert_eq!(e_attr.name.local_name, a_attr.name.local_name);
            assert_eq!(e_attr.name.prefix, a_attr.name.prefix);
            assert_eq!(e_attr.value, a_attr.value);
        }
    }

    #[test]
    fn normalize_works_with_prefix() {
        let raw = OwnedName {
            local_name: "local_name".to_string(),
            namespace: None,
            prefix: Some("prefix".to_string()),
        };
        assert_eq!(normalize(&raw), "prefix:local_name");
    }

    #[test]
    fn normalize_works_without_prefix() {
        let raw = OwnedName {
            local_name: "local_name".to_string(),
            namespace: None,
            prefix: None,
        };
        assert_eq!(normalize(&raw), "local_name");
    }

    #[test]
    fn normalize_ignores_namespace() {
        let no_namespace = OwnedName {
            local_name: "local_name".to_string(),
            namespace: None,
            prefix: None,
        };
        let yes_namespace = OwnedName {
            local_name: "local_name".to_string(),
            namespace: Some("namespace".to_string()),
            prefix: None,
        };
        assert_eq!(normalize(&no_namespace), normalize(&yes_namespace));
    }

    #[test]
    fn normalize_accepts_empty_prefix() {
        let raw = OwnedName {
            local_name: "local_name".to_string(),
            namespace: None,
            prefix: Some("".to_string()),
        };
        assert_eq!(normalize(&raw), ":local_name");
    }

    #[test]
    fn ablip_extracts_ablip() {
        let tag = Tag::ABlip {
            rel: "RelId".to_string(),
        };
        let extracted = tag.a_blip();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "RelId");
    }

    #[test]
    fn ablip_rejects_other() {
        let tag = Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.a_blip();
        assert!(extracted.is_none());
    }

    #[test]
    fn mchr_extracts_mchr() {
        let tag = Tag::MChr {
            value: "X".to_string(),
        };
        let extracted = tag.m_chr();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "X");
    }

    #[test]
    fn mchr_rejects_other() {
        let tag = Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.m_chr();
        assert!(extracted.is_none());
    }

    #[test]
    fn wbookmarkstart_extracts_wbookmarkstart() {
        let tag = Tag::WBookmarkStart {
            anchor: "Anchor".to_string(),
        };
        let extracted = tag.w_bookmark_start();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "Anchor");
    }

    #[test]
    fn wbookmarkstart_rejects_other() {
        let tag = Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.w_bookmark_start();
        assert!(extracted.is_none());
    }

    #[test]
    fn whyperlink_extracts_whyperlink_anchor() {
        let anchor = Tag::WHyperlink(Link::Anchor("Anchor".to_string()));
        let extracted = anchor.w_hyperlink();
        assert!(extracted.is_some());
        assert!(matches!(extracted.unwrap(), Link::Anchor(_)));
        if let Some(Link::Anchor(anchor)) = extracted {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn whyperlink_extracts_whyperlink_relationship() {
        let rel = Tag::WHyperlink(Link::Relationship("RelId".to_string()));
        let extracted = rel.w_hyperlink();
        assert!(extracted.is_some());
        assert!(matches!(extracted.unwrap(), Link::Relationship(_)));
        if let Some(Link::Relationship(rel)) = extracted {
            assert_eq!(rel, "RelId");
        }
    }

    #[test]
    fn whyperlink_rejects_other() {
        let tag = Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.w_hyperlink();
        assert!(extracted.is_none());
    }

    #[test]
    fn content_extracts_content() {
        let tag = Tag::Content("Content".to_string());
        let extracted = tag.content();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "Content");
    }

    #[test]
    fn content_rejects_other() {
        let tag = Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.content();
        assert!(extracted.is_none());
    }

    fn owned(raw: &'static str) -> OwnedName {
        let parts: Vec<_> = raw.split(':').collect();
        OwnedName {
            local_name: parts[1].to_string(),
            namespace: None,
            prefix: Some(parts[0].to_string()),
        }
    }

    #[test]
    fn converts_empty_tags() {
        use Tag::*;
        let owned_names = vec![
            owned_name("a", "graphic"),
            owned_name("a", "graphicData"),
            owned_name("pic", "pic"),
            owned_name("pic", "blipFill"),
            owned_name("m", "oMathPara"),
            owned_name("m", "oMath"),
            owned_name("m", "d"),
            owned_name("m", "rad"),
            owned_name("m", "deg"),
            owned_name("m", "r"),
            owned_name("m", "t"),
            owned_name("m", "sub"),
            owned_name("m", "sup"),
            owned_name("m", "nary"),
            owned_name("m", "naryPr"),
            owned_name("m", "f"),
            owned_name("m", "func"),
            owned_name("m", "fName"),
            owned_name("m", "num"),
            owned_name("m", "den"),
            owned_name("wp", "inline"),
            owned_name("wp", "anchor"),
            owned_name("w", "bookmarkEnd"),
            owned_name("w", "drawing"),
            owned_name("w", "p"),
            owned_name("w", "r"),
            owned_name("w", "t"),
        ];
        let expected = vec![
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
            WDrawing,
            WParagraph,
            WRun,
            WText,
        ];
        assert_eq!(owned_names.len(), expected.len());
        for i in 0..owned_names.len() {
            let name = &owned_names[i];
            let actual = Tag::try_from((name, &vec![])).expect("Input was constructed manually");
            assert_eq!(actual, expected[i]);
        }
    }

    #[test]
    fn converts_ablip_with_attribute() {
        let name = owned("a:blip");
        let attribute = OwnedAttribute {
            name: owned("r:embed"),
            value: "RelId".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::ABlip { rel: _ }));
        if let Tag::ABlip { rel } = actual {
            assert_eq!(rel, "RelId");
        }
    }

    #[test]
    fn rejects_ablip_without_attribute() {
        let name = owned("a:blip");

        let actual = Tag::try_from((&name, &vec![]));

        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let InputError::MissingAttributes { id, missing } = actual;

        assert_eq!(id, "a:blip");
        assert_eq!(missing, vec!["r:embed"]);
    }

    #[test]
    fn converts_mchr_with_attribute() {
        let name = owned("m:chr");
        let attribute = OwnedAttribute {
            name: owned("m:val"),
            value: "X".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::MChr { value: _ }));
        if let Tag::MChr { value } = actual {
            assert_eq!(value, "X");
        }
    }

    #[test]
    fn rejects_mchr_with_no_attribute() {
        let name = owned("m:chr");

        let actual = Tag::try_from((&name, &vec![]));
        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let InputError::MissingAttributes { id, missing } = actual;

        assert_eq!(id, "m:chr");
        assert_eq!(missing, vec!["m:val"]);
    }

    #[test]
    fn converts_wbookmarkstart_with_attribute() {
        let name = owned("w:bookmarkStart");
        let attribute = OwnedAttribute {
            name: owned("w:anchor"),
            value: "Anchor".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::WBookmarkStart { anchor: _ }));
        if let Tag::WBookmarkStart { anchor } = actual {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn accepts_wbookmarkstart_with_no_attribute() {
        let name = owned("w:bookmarkStart");

        let actual = Tag::try_from((&name, &vec![]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::WBookmarkStart { anchor: _ }));
        if let Tag::WBookmarkStart { anchor } = actual {
            assert_eq!(anchor, "");
        }
    }

    #[test]
    fn converts_whyperlink_with_relationship() {
        let name = owned("w:hyperlink");
        let attribute = OwnedAttribute {
            name: owned("r:id"),
            value: "RelId".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::WHyperlink(Link::Relationship(_))));
        if let Tag::WHyperlink(Link::Relationship(rel)) = actual {
            assert_eq!(rel, "RelId");
        }
    }

    #[test]
    fn converts_whyperlink_with_anchor() {
        let name = owned("w:hyperlink");
        let attribute = OwnedAttribute {
            name: owned("w:anchor"),
            value: "Anchor".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::WHyperlink(Link::Anchor(_))));
        if let Tag::WHyperlink(Link::Anchor(anchor)) = actual {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn rejects_whyperlink_with_no_attributes() {
        let name = owned("w:hyperlink");

        let actual = Tag::try_from((&name, &vec![]));
        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let InputError::MissingAttributes { id, missing } = actual;

        assert_eq!(id, "w:hyperlink");
        assert_eq!(missing, vec!["r:id", "w:anchor"]);
    }

    #[test]
    fn accepts_unknown_tags() {
        let name = owned("alien:tag");
        let attribute = OwnedAttribute {
            name: owned("alien:attribute"),
            value: "Alien".to_string(),
        };

        let actual = Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, Tag::Unknown { id: _ }));
        if let Tag::Unknown { id } = actual {
            assert_eq!(id, "alien:tag");
        }
    }
}
