use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use syntect::highlighting::Style as SyntectStyle;
use syntect::{easy::HighlightLines, parsing::SyntaxSet};

pub fn highlight_line_content<'a>(
    content: &'a str,
    syntax: Option<&syntect::parsing::SyntaxReference>,
    syntax_set: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Vec<Span<'a>> {
    if let Some(syntax) = syntax {
        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(content, syntax_set) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| {
                    Span::styled(text.to_string(), syntect_style_to_ratatui(style))
                })
                .collect(),
            Err(_) => vec![Span::raw(content.to_string())],
        }
    } else {
        vec![Span::raw(content.to_string())]
    }
}

fn syntect_style_to_ratatui(syntect_style: SyntectStyle) -> Style {
    let fg_color = Color::Rgb(
        syntect_style.foreground.r,
        syntect_style.foreground.g,
        syntect_style.foreground.b,
    );

    let mut style = Style::default().fg(fg_color);

    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        style = style.add_modifier(Modifier::BOLD);
    }
    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::UNDERLINE)
    {
        style = style.add_modifier(Modifier::UNDERLINED);
    }

    style
}
