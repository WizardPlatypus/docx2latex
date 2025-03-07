use super::{blink, Boo, Link, Tag};

pub fn hyperlink(boo: &Boo<Tag>) -> Option<(&Link, &String)> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::WText))?;
    blink(matches!(boo.peek()?, Tag::WRun))?;
    let link = boo.peek()?.w_hyperlink()?;
    Some((link, content))
}

pub fn drawing(boo: &Boo<Tag>) -> Option<&String> {
    boo.reset();
    let rel = boo.peek()?.a_blip()?;
    blink(matches!(boo.peek()?, Tag::PicBlipFill))?;
    blink(matches!(boo.peek()?, Tag::PicPic))?;
    blink(matches!(boo.peek()?, Tag::AGraphicData))?;
    blink(matches!(boo.peek()?, Tag::AGraphic))?;
    let temp = boo.peek()?;
    blink(matches!(temp, Tag::WPInline) || matches!(temp, Tag::WPAnchor))?;
    blink(matches!(boo.peek()?, Tag::WDrawing))?;
    Some(rel)
}

pub fn word_text(boo: &Boo<Tag>) -> Option<&String> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::WText))?;
    blink(matches!(boo.peek()?, Tag::WRun))?;
    Some(content)
}

pub fn math_text(boo: &Boo<Tag>) -> Option<&String> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::MText))?;
    blink(matches!(boo.peek()?, Tag::MRun))?;
    Some(content)
}

#[cfg(test)]
mod test {
    #[test]
    fn hyperlink_works() {
        let mut boo = super::Boo::default();
        assert!(super::hyperlink(&boo).is_none());

        boo.push(super::Tag::WHyperlink(super::Link::Anchor(
            "Anchor".to_string(),
        )));
        assert!(super::hyperlink(&boo).is_none());

        boo.push(super::Tag::WRun);
        assert!(super::hyperlink(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::hyperlink(&boo).is_none());

        boo.pop();
        boo.push(super::Tag::WText);
        assert!(super::hyperlink(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::hyperlink(&boo).is_some());

        boo.reset();
        assert!(super::hyperlink(&boo).is_some());

        let (link, content) = super::hyperlink(&boo).unwrap();
        assert_eq!(link, &super::Link::Anchor("Anchor".to_string()));
        assert_eq!(content, "Content");
    }

    #[test]
    fn drawing_works() {
        let mut boo = super::Boo::default();
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::WDrawing);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::WDrawing);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::WPInline);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::AGraphic);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::AGraphicData);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::PicPic);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::PicBlipFill);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(super::drawing(&boo).is_some());

        boo.reset();
        assert!(super::drawing(&boo).is_some());

        let rel = super::drawing(&boo).unwrap();
        assert_eq!(rel, "RelId");

        boo.pop();
        assert!(super::drawing(&boo).is_none());
        boo.pop();
        assert!(super::drawing(&boo).is_none());
        boo.pop();
        assert!(super::drawing(&boo).is_none());
        boo.pop();
        assert!(super::drawing(&boo).is_none());
        boo.pop();
        assert!(super::drawing(&boo).is_none());
        boo.pop();
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::WPAnchor);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::AGraphic);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::AGraphicData);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::PicPic);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(super::drawing(&boo).is_none());

        boo.pop();
        boo.push(super::Tag::PicBlipFill);
        assert!(super::drawing(&boo).is_none());

        boo.push(super::Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(super::drawing(&boo).is_some());

        boo.reset();
        assert!(super::drawing(&boo).is_some());

        let rel = super::drawing(&boo).unwrap();
        assert_eq!(rel, "RelId");
    }

    #[test]
    fn word_text_works() {
        let mut boo = super::Boo::default();
        assert!(super::word_text(&boo).is_none());

        boo.push(super::Tag::WRun);
        assert!(super::word_text(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::word_text(&boo).is_none());

        boo.pop();
        boo.push(super::Tag::WText);
        assert!(super::word_text(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::word_text(&boo).is_some());

        boo.reset();
        assert!(super::word_text(&boo).is_some());

        let content = super::word_text(&boo).unwrap();
        assert_eq!(content, "Content");
    }

    #[test]
    fn math_text_works() {
        let mut boo = super::Boo::default();
        assert!(super::math_text(&boo).is_none());

        boo.push(super::Tag::MRun);
        assert!(super::math_text(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::math_text(&boo).is_none());

        boo.pop();
        boo.push(super::Tag::MText);
        assert!(super::math_text(&boo).is_none());

        boo.push(super::Tag::Content("Content".to_string()));
        assert!(super::math_text(&boo).is_some());

        boo.reset();
        assert!(super::math_text(&boo).is_some());

        let content = super::math_text(&boo).unwrap();
        assert_eq!(content, "Content");
    }
}
