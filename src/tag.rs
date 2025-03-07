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

    pub fn to_owned(&self) -> Option<(super::OwnedName, Vec<super::OwnedAttribute>)> {
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

pub fn owned_name(prefix: &str, local: &str) -> OwnedName {
    OwnedName {
        local_name: local.to_string(),
        namespace: None,
        prefix: Some(prefix.to_string()),
    }
}

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
    use xml::{attribute::OwnedAttribute, name::OwnedName};

    use crate::tag::normalize;

    #[test]
    fn owned_name_works() {
        let name = super::owned_name("prefix", "local");
        assert!(name.prefix_ref().is_some());
        assert_eq!(name.prefix_ref().unwrap(), "prefix");
        assert_eq!(name.local_name, "local");
    }

    #[test]
    fn owned_attr_works() {
        let attr = super::owned_attr("prefix", "local", "value");
        assert!(attr.name.prefix_ref().is_some());
        assert_eq!(attr.name.prefix_ref().unwrap(), "prefix");
        assert_eq!(attr.name.local_name, "local");
        assert_eq!(attr.value, "value");
    }

    #[test]
    fn to_owned_works() {
        let expected = vec![
            (super::owned_name("a", "graphic"), vec![]),
            (super::owned_name("a", "graphicData"), vec![]),
            (super::owned_name("pic", "pic"), vec![]),
            (super::owned_name("pic", "blipFill"), vec![]),
            (super::owned_name("m", "oMathPara"), vec![]),
            (super::owned_name("m", "oMath"), vec![]),
            (super::owned_name("m", "d"), vec![]),
            (super::owned_name("m", "rad"), vec![]),
            (super::owned_name("m", "deg"), vec![]),
            (super::owned_name("m", "r"), vec![]),
            (super::owned_name("m", "t"), vec![]),
            (super::owned_name("m", "sub"), vec![]),
            (super::owned_name("m", "sup"), vec![]),
            (super::owned_name("m", "nary"), vec![]),
            (super::owned_name("m", "naryPr"), vec![]),
            (super::owned_name("m", "f"), vec![]),
            (super::owned_name("m", "func"), vec![]),
            (super::owned_name("m", "fName"), vec![]),
            (super::owned_name("m", "num"), vec![]),
            (super::owned_name("m", "den"), vec![]),
            (super::owned_name("wp", "inline"), vec![]),
            (super::owned_name("wp", "anchor"), vec![]),
            (super::owned_name("w", "bookmarkEnd"), vec![]),
            (super::owned_name("w", "drawing"), vec![]),
            (super::owned_name("w", "p"), vec![]),
            (super::owned_name("w", "r"), vec![]),
            (super::owned_name("w", "t"), vec![]),
            (
                super::owned_name("a", "blip"),
                vec![super::owned_attr("r", "id", "RelId")],
            ),
            (
                super::owned_name("m", "chr"),
                vec![super::owned_attr("m", "val", "X")],
            ),
            (
                super::owned_name("w", "bookmarkStart"),
                vec![super::owned_attr("w", "anchor", "Anchor")],
            ),
            (
                super::owned_name("w", "hyperlink"),
                vec![super::owned_attr("w", "anchor", "Anchor")],
            ),
            (
                super::owned_name("w", "hyperlink"),
                vec![super::owned_attr("r", "id", "RelId")],
            ),
        ];
        use super::Tag::*;
        let tags = vec![
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
            ABlip {
                rel: "RelId".to_string(),
            },
            MChr {
                value: "X".to_string(),
            },
            WBookmarkStart {
                anchor: "Anchor".to_string(),
            },
            WHyperlink(super::Link::Anchor("Anchor".to_string())),
            WHyperlink(super::Link::Relationship("RelId".to_string())),
        ];

        assert_eq!(expected.len(), tags.len());
        for i in 0..expected.len() {
            let (e_name, e_attrs) = &expected[i];
            let owned = tags[i].to_owned();

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
        let tag = super::Tag::ABlip {
            rel: "RelId".to_string(),
        };
        let extracted = tag.a_blip();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "RelId");
    }

    #[test]
    fn ablip_rejects_other() {
        let tag = super::Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.a_blip();
        assert!(extracted.is_none());
    }

    #[test]
    fn mchr_extracts_mchr() {
        let tag = super::Tag::MChr {
            value: "X".to_string(),
        };
        let extracted = tag.m_chr();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "X");
    }

    #[test]
    fn mchr_rejects_other() {
        let tag = super::Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.m_chr();
        assert!(extracted.is_none());
    }

    #[test]
    fn wbookmarkstart_extracts_wbookmarkstart() {
        let tag = super::Tag::WBookmarkStart {
            anchor: "Anchor".to_string(),
        };
        let extracted = tag.w_bookmark_start();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "Anchor");
    }

    #[test]
    fn wbookmarkstart_rejects_other() {
        let tag = super::Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.w_bookmark_start();
        assert!(extracted.is_none());
    }

    #[test]
    fn whyperlink_extracts_whyperlink_anchor() {
        let anchor = super::Tag::WHyperlink(super::Link::Anchor("Anchor".to_string()));
        let extracted = anchor.w_hyperlink();
        assert!(extracted.is_some());
        assert!(matches!(extracted.unwrap(), super::Link::Anchor(_)));
        if let Some(super::Link::Anchor(anchor)) = extracted {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn whyperlink_extracts_whyperlink_relationship() {
        let rel = super::Tag::WHyperlink(super::Link::Relationship("RelId".to_string()));
        let extracted = rel.w_hyperlink();
        assert!(extracted.is_some());
        assert!(matches!(extracted.unwrap(), super::Link::Relationship(_)));
        if let Some(super::Link::Relationship(rel)) = extracted {
            assert_eq!(rel, "RelId");
        }
    }

    #[test]
    fn whyperlink_rejects_other() {
        let tag = super::Tag::Unknown {
            id: "Junk".to_string(),
        };
        let extracted = tag.w_hyperlink();
        assert!(extracted.is_none());
    }

    #[test]
    fn content_extracts_content() {
        let tag = super::Tag::Content("Content".to_string());
        let extracted = tag.content();
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), "Content");
    }

    #[test]
    fn content_rejects_other() {
        let tag = super::Tag::Unknown {
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
        use super::Tag::*;
        let owned_names = vec![
            super::owned_name("a", "graphic"),
            super::owned_name("a", "graphicData"),
            super::owned_name("pic", "pic"),
            super::owned_name("pic", "blipFill"),
            super::owned_name("m", "oMathPara"),
            super::owned_name("m", "oMath"),
            super::owned_name("m", "d"),
            super::owned_name("m", "rad"),
            super::owned_name("m", "deg"),
            super::owned_name("m", "r"),
            super::owned_name("m", "t"),
            super::owned_name("m", "sub"),
            super::owned_name("m", "sup"),
            super::owned_name("m", "nary"),
            super::owned_name("m", "naryPr"),
            super::owned_name("m", "f"),
            super::owned_name("m", "func"),
            super::owned_name("m", "fName"),
            super::owned_name("m", "num"),
            super::owned_name("m", "den"),
            super::owned_name("wp", "inline"),
            super::owned_name("wp", "anchor"),
            super::owned_name("w", "bookmarkEnd"),
            super::owned_name("w", "drawing"),
            super::owned_name("w", "p"),
            super::owned_name("w", "r"),
            super::owned_name("w", "t"),
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
            let actual =
                super::Tag::try_from((name, &vec![])).expect("Input was constructed manually");
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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, super::Tag::ABlip { rel: _ }));
        if let super::Tag::ABlip { rel } = actual {
            assert_eq!(rel, "RelId");
        }
    }

    #[test]
    fn rejects_ablip_without_attribute() {
        let name = owned("a:blip");

        let actual = super::Tag::try_from((&name, &vec![]));

        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let super::InputError::MissingAttributes { id, missing } = actual;

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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, super::Tag::MChr { value: _ }));
        if let super::Tag::MChr { value } = actual {
            assert_eq!(value, "X");
        }
    }

    #[test]
    fn rejects_mchr_with_no_attribute() {
        let name = owned("m:chr");

        let actual = super::Tag::try_from((&name, &vec![]));
        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let super::InputError::MissingAttributes { id, missing } = actual;

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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, super::Tag::WBookmarkStart { anchor: _ }));
        if let super::Tag::WBookmarkStart { anchor } = actual {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn accepts_wbookmarkstart_with_no_attribute() {
        let name = owned("w:bookmarkStart");

        let actual = super::Tag::try_from((&name, &vec![]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, super::Tag::WBookmarkStart { anchor: _ }));
        if let super::Tag::WBookmarkStart { anchor } = actual {
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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(
            actual,
            super::Tag::WHyperlink(super::Link::Relationship(_))
        ));
        if let super::Tag::WHyperlink(super::Link::Relationship(rel)) = actual {
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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(
            actual,
            super::Tag::WHyperlink(super::Link::Anchor(_))
        ));
        if let super::Tag::WHyperlink(super::Link::Anchor(anchor)) = actual {
            assert_eq!(anchor, "Anchor");
        }
    }

    #[test]
    fn rejects_whyperlink_with_no_attributes() {
        let name = owned("w:hyperlink");

        let actual = super::Tag::try_from((&name, &vec![]));
        assert!(actual.is_err());
        let actual = actual.unwrap_err();
        let super::InputError::MissingAttributes { id, missing } = actual;

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

        let actual = super::Tag::try_from((&name, &vec![attribute]));
        assert!(actual.is_ok());
        let actual = actual.unwrap();

        assert!(matches!(actual, super::Tag::Unknown { id: _ }));
        if let super::Tag::Unknown { id } = actual {
            assert_eq!(id, "alien:tag");
        }
    }
}
