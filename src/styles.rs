type StyleId = String;

struct Style {
    style_id: StyleId,
    name: String,
    based_on: String,
    // q_format: bool, ???
    aliases: Vec<String>,
    default: bool,
    // custom_style: bool
    // next: StyleId,
    // hidden: bool,
    // locked: bool,
    // semi_hidden: bool
    // unhide_when_used: bool
    // ui_priority: i64,
}

enum Troll {
    Auto,
    True,
    False
}

enum Hanger {
    FirstLine(i64),
    Hanging(i64)
}

struct Indentation {
    /// left/start
    start: Option<i64>,
    /// right/end
    end: Option<i64>,
    hanger: Option<Hanger>,
}

enum Alignment {
    Start,
    End,
    Center,
    Both,
    Distribute
}

struct ParagraphStyle {
    /// linked character style
    character_style: Option<StyleId>,
    // frame_pr: bool
    /// <w:ind />
    indentation: Option<Indentation>,
    /// <w:jc />
    alignment: Option<Alignment>,
    /// <w:keepLines/>
    keep_lines: bool,
    /// <w:keepNext/>
    keep_next: bool,
    // numPr
    // outlineLvl
    // pBdr
    // shd
    // spacing
    // tabs
    // textAlignment
}

struct Color {
    theme_color: Option<String>,
    theme_shade: Option<String>,
    theme_tint: Option<String>,
    value: Option<String>
}

struct CharacterStyle {
    size: Option<i64>,
    /// linked paragraph style
    paragraph_style: Option<StyleId>,
    /// <w:b /> toggle
    bold: Option<Troll>,
    /// <w:i /> toggle
    italics: Option<Troll>,
    /// <w:caps /> toggle
    caps: Option<Troll>,
    /// <w:color />
    color: Option<Color>,
    /// <w:strike /> toggle
    strike: Option<Troll>,
    /// <w:dstrike /> toggle
    double_strike: Option<Troll>,
    /// <w:u />
    underline: Option<Troll>,
    // <w:emboss /> toggle
    // <w:imprint /> toggle
    // <w:outline /> toggle
    // <w:shadow /> toggle
    // <w:smallCaps /> toggle
    // <w:vanish /> toggle
}

// TODO: Table styles
// TODO: Numbering styles