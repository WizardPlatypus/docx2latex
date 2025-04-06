use super::{blink, Link, Tag};
use crate::peekaboo::Peek;

pub fn hyperlink<P: Peek<Item = Tag>>(boo: &P) -> Option<(&Link, &String)> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::WText))?;
    blink(matches!(boo.peek()?, Tag::WRun))?;
    let link = boo.peek()?.w_hyperlink()?;
    Some((link, content))
}

pub fn drawing<P: Peek<Item = Tag>>(boo: &P) -> Option<&String> {
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

pub fn word_text<P: Peek<Item = Tag>>(boo: &P) -> Option<&String> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::WText))?;
    blink(matches!(boo.peek()?, Tag::WRun))?;
    Some(content)
}

pub fn math_text<P: Peek<Item = Tag>>(boo: &P) -> Option<&String> {
    boo.reset();
    let content = boo.peek()?.content()?;
    blink(matches!(boo.peek()?, Tag::MText))?;
    blink(matches!(boo.peek()?, Tag::MRun))?;
    Some(content)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{peekaboo::MockPeek, Boo};
    use unimock::*;

    #[test]
    fn hyperlink_works() {
        let mut boo = Boo::default();
        assert!(hyperlink(&boo).is_none());

        boo.push(Tag::WHyperlink(Link::Anchor("Anchor".to_string())));
        assert!(hyperlink(&boo).is_none());

        boo.push(Tag::WRun);
        assert!(hyperlink(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(hyperlink(&boo).is_none());

        boo.pop();
        boo.push(Tag::WText);
        assert!(hyperlink(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(hyperlink(&boo).is_some());

        boo.reset();
        assert!(hyperlink(&boo).is_some());

        let (link, content) = hyperlink(&boo).unwrap();
        assert_eq!(link, &Link::Anchor("Anchor".to_string()));
        assert_eq!(content, "Content");
    }

    #[test]
    fn hyperlink_mock() {
        let boo = Unimock::new((
            MockPeek::reset.next_call(matching!()).returns(()),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::Content("Content".to_string()))),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WText)),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WRun)),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WHyperlink(Link::Anchor("Any".to_string())))),
        ));

        let (link, content) = hyperlink(&boo).unwrap();
        assert_eq!(link, &Link::Anchor("Any".to_string()));
        assert_eq!(content, "Content");
    }

    #[test]
    #[should_panic]
    fn hyperlink_fails() {
        let boo = Unimock::new((
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::Content("Content".to_string()))),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WText)),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WRun)),
            MockPeek::peek.next_call(matching!()).returns(Some(Tag::WHyperlink(Link::Anchor("Any".to_string())))),
        ));

        let (link, content) = hyperlink(&boo).unwrap();
        assert_eq!(link, &Link::Anchor("Any".to_string()));
        assert_eq!(content, "Content");
    }

    #[test]
    fn drawing_works() {
        let mut boo = Boo::default();
        assert!(drawing(&boo).is_none());

        boo.push(Tag::WDrawing);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::WDrawing);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::WPInline);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::AGraphic);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::AGraphicData);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::PicPic);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::PicBlipFill);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(drawing(&boo).is_some());

        boo.reset();
        assert!(drawing(&boo).is_some());

        let rel = drawing(&boo).unwrap();
        assert_eq!(rel, "RelId");

        boo.pop();
        assert!(drawing(&boo).is_none());
        boo.pop();
        assert!(drawing(&boo).is_none());
        boo.pop();
        assert!(drawing(&boo).is_none());
        boo.pop();
        assert!(drawing(&boo).is_none());
        boo.pop();
        assert!(drawing(&boo).is_none());
        boo.pop();
        assert!(drawing(&boo).is_none());

        boo.push(Tag::WPAnchor);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::AGraphic);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::AGraphicData);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::PicPic);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(drawing(&boo).is_none());

        boo.pop();
        boo.push(Tag::PicBlipFill);
        assert!(drawing(&boo).is_none());

        boo.push(Tag::ABlip {
            rel: "RelId".to_string(),
        });
        assert!(drawing(&boo).is_some());

        boo.reset();
        assert!(drawing(&boo).is_some());

        let rel = drawing(&boo).unwrap();
        assert_eq!(rel, "RelId");
    }

    #[test]
    fn word_text_works() {
        let mut boo = Boo::default();
        assert!(word_text(&boo).is_none());

        boo.push(Tag::WRun);
        assert!(word_text(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(word_text(&boo).is_none());

        boo.pop();
        boo.push(Tag::WText);
        assert!(word_text(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(word_text(&boo).is_some());

        boo.reset();
        assert!(word_text(&boo).is_some());

        let content = word_text(&boo).unwrap();
        assert_eq!(content, "Content");
    }

    #[test]
    fn math_text_works() {
        let mut boo = Boo::default();
        assert!(math_text(&boo).is_none());

        boo.push(Tag::MRun);
        assert!(math_text(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(math_text(&boo).is_none());

        boo.pop();
        boo.push(Tag::MText);
        assert!(math_text(&boo).is_none());

        boo.push(Tag::Content("Content".to_string()));
        assert!(math_text(&boo).is_some());

        boo.reset();
        assert!(math_text(&boo).is_some());

        let content = math_text(&boo).unwrap();
        assert_eq!(content, "Content");
    }
}
