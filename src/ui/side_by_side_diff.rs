use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    app::App,
    diff_file::{DiffFile, LineType},
    ui::highlight_line::highlight_line_content,
};

pub fn render_side_by_side_diff(
    f: &mut Frame,
    area: Rect,
    file: &DiffFile,
    scroll_offset: usize,
    app: &App,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let visible_lines = (area.height - 2) as usize;
    let panel_width = (chunks[0].width.saturating_sub(2)) as usize; // Width minus borders
    let syntax = app.get_syntax_for_file(file.get_name());
    let theme = app.get_theme("base16-ocean.dark");

    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();

    for diff_line in file.lines.iter().skip(scroll_offset).take(visible_lines) {
        match diff_line.line_type {
            LineType::Context => {
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, app.get_syntax_set(), theme);

                let mut old_spans = vec![Span::styled(
                    format!("{:4} ", diff_line.old_line_num.unwrap_or(0)),
                    Style::default().fg(Color::DarkGray),
                )];
                old_spans.extend(highlighted_spans.clone());
                old_lines.push(Line::from(old_spans));

                let mut new_spans = vec![Span::styled(
                    format!("{:4} ", diff_line.new_line_num.unwrap_or(0)),
                    Style::default().fg(Color::DarkGray),
                )];
                new_spans.extend(highlighted_spans);
                new_lines.push(Line::from(new_spans));
            }
            LineType::Removed => {
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, app.get_syntax_set(), theme);

                let mut old_spans = vec![Span::styled(
                    format!("{:4} ", diff_line.old_line_num.unwrap_or(0)),
                    Style::default().fg(Color::DarkGray),
                )];

                for span in highlighted_spans {
                    let mut new_style = span.style;
                    new_style = new_style.bg(Color::Rgb(139, 0, 0));
                    old_spans.push(Span::styled(span.content, new_style));
                }
                old_lines.push(Line::from(old_spans));

                // Empty line with background fill matching panel width
                let empty_content = " ".repeat(panel_width);
                new_lines.push(Line::from(Span::styled(
                    empty_content,
                    Style::default().bg(Color::Rgb(40, 40, 40)),
                )));
            }
            LineType::Added => {
                // Empty line with background fill matching panel width
                let empty_content = " ".repeat(panel_width);
                old_lines.push(Line::from(Span::styled(
                    empty_content,
                    Style::default().bg(Color::Rgb(40, 40, 40)),
                )));

                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, app.get_syntax_set(), theme);

                let mut new_spans = vec![Span::styled(
                    format!("{:4} ", diff_line.new_line_num.unwrap_or(0)),
                    Style::default().fg(Color::DarkGray),
                )];

                for span in highlighted_spans {
                    let mut new_style = span.style;
                    new_style = new_style.bg(Color::Rgb(0, 100, 0));
                    new_spans.push(Span::styled(span.content, new_style));
                }
                new_lines.push(Line::from(new_spans));
            }
            LineType::Header => {
                let header_line = Line::from(vec![Span::styled(
                    &diff_line.content,
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]);
                old_lines.push(header_line.clone());
                new_lines.push(header_line);
            }
        }
    }

    let old_text = Text::from(old_lines);
    let new_text = Text::from(new_lines);

    let old_title = format!("Old: {}", file.get_name());
    let new_title = format!("New: {}", file.get_name());

    let old_paragraph =
        Paragraph::new(old_text).block(Block::default().borders(Borders::ALL).title(old_title));

    let new_paragraph =
        Paragraph::new(new_text).block(Block::default().borders(Borders::ALL).title(new_title));

    f.render_widget(old_paragraph, chunks[0]);
    f.render_widget(new_paragraph, chunks[1]);

    // Render scrollbars for both panels
    let total_lines = file.line_count();
    if total_lines > visible_lines {
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll_offset);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        // Scrollbar for old (left) panel
        f.render_stateful_widget(
            scrollbar.clone(),
            chunks[0].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state.clone(),
        );

        // Scrollbar for new (right) panel
        f.render_stateful_widget(
            scrollbar,
            chunks[1].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}
