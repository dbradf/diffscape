use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::{
    app::App,
    diff_file::{DiffFile, LineType},
    ui::highlight_line::highlight_line_content,
};

pub fn render_unified_diff(
    f: &mut Frame,
    area: Rect,
    file: &DiffFile,
    scroll_offset: usize,
    app: &App,
) {
    let visible_lines = (area.height - 2) as usize; // Account for borders
    let _end_line = (scroll_offset + visible_lines).min(file.line_count());

    let syntax = app.get_syntax_for_file(file.get_name());
    let theme = app.get_theme("base16-ocean.dark");

    let lines: Vec<Line> = file
        .lines
        .iter()
        .skip(scroll_offset)
        .take(visible_lines)
        .map(|diff_line| {
            let line_num_text = match (&diff_line.old_line_num, &diff_line.new_line_num) {
                (Some(old), Some(new)) => format!("{:4}:{:4} ", old, new),
                (Some(old), None) => format!("{:4}:     ", old),
                (None, Some(new)) => format!("     {:4} ", new),
                (None, None) => "         ".to_string(),
            };

            let mut spans = vec![Span::styled(
                line_num_text,
                Style::default().fg(Color::DarkGray),
            )];

            let (bg_color, prefix) = match diff_line.line_type {
                LineType::Added => (Some(Color::Rgb(0, 100, 0)), "+ "),
                LineType::Removed => (Some(Color::Rgb(139, 0, 0)), "- "),
                LineType::Context => (None, "  "),
                LineType::Header => (Some(Color::Blue), "@ "),
            };

            // Add prefix
            spans.push(Span::styled(
                prefix,
                match bg_color {
                    Some(bg) => Style::default().bg(bg).fg(Color::White),
                    None => Style::default().fg(Color::White),
                },
            ));

            if diff_line.line_type == LineType::Header {
                // Headers don't get syntax highlighting
                spans.push(Span::styled(
                    &diff_line.content,
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if let Some(bg) = bg_color {
                // Syntax highlight the content but apply background color
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, app.get_syntax_set(), theme);
                for span in highlighted_spans {
                    let mut new_style = span.style;
                    new_style = new_style.bg(bg);
                    spans.push(Span::styled(span.content, new_style));
                }
            } else {
                // Context lines - just syntax highlight normally
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, app.get_syntax_set(), theme);
                spans.extend(highlighted_spans);
            }

            Line::from(spans)
        })
        .collect();

    let diff_text = Text::from(lines);
    let paragraph = Paragraph::new(diff_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(file.get_name()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    // Render scrollbar
    let total_lines = file.line_count();
    if total_lines > visible_lines {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll_offset);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        f.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}
