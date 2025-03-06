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
    blink(matches!(boo.peek()?, Tag::WPInline) || matches!(boo.top()?, Tag::WPAnchor))?;
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
