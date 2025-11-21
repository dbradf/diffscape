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

    let mut i = scroll_offset;
    let end_line = (scroll_offset + visible_lines).min(file.line_count());

    while i < end_line {
        let diff_line = &file.lines[i];
        
        // Check for intra-line diff opportunity
        if diff_line.line_type == LineType::Removed && i + 1 < file.line_count() {
            let next_line = &file.lines[i + 1];
            if next_line.line_type == LineType::Added {
                 if i + 1 < end_line {
                    let (old_ranges, new_ranges) = crate::ui::diff_utils::compute_intra_line_diff(
                        &diff_line.content,
                        &next_line.content,
                    );

                    old_lines.push(render_diff_line(
                        diff_line,
                        syntax,
                        app.get_syntax_set(),
                        theme,
                        Some((&old_ranges, Color::Rgb(139, 0, 0), Color::Rgb(80, 0, 0))),
                    ));

                    new_lines.push(render_diff_line(
                        next_line,
                        syntax,
                        app.get_syntax_set(),
                        theme,
                        Some((&new_ranges, Color::Rgb(0, 100, 0), Color::Rgb(0, 60, 0))),
                    ));

                    i += 2;
                    continue;
                }
            }
        }

        match diff_line.line_type {
            LineType::Context => {
                let line = render_diff_line(diff_line, syntax, app.get_syntax_set(), theme, None);
                old_lines.push(line.clone());
                new_lines.push(line);
            }
            LineType::Removed => {
                old_lines.push(render_diff_line(diff_line, syntax, app.get_syntax_set(), theme, None));
                
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

                new_lines.push(render_diff_line(diff_line, syntax, app.get_syntax_set(), theme, None));
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
        i += 1;
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
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll_offset);

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

fn render_diff_line<'a>(
    diff_line: &'a crate::diff_file::DiffLine,
    syntax: Option<&syntect::parsing::SyntaxReference>,
    syntax_set: &syntect::parsing::SyntaxSet,
    theme: &syntect::highlighting::Theme,
    intra_line_highlight: Option<(&[std::ops::Range<usize>], Color, Color)>,
) -> Line<'a> {
    let line_num_text = match (&diff_line.old_line_num, &diff_line.new_line_num) {
        (Some(old), Some(new)) => format!("{:4}:{:4} ", old, new),
        (Some(old), None) => format!("{:4}:     ", old),
        (None, Some(new)) => format!("     {:4} ", new),
        (None, None) => "         ".to_string(),
    };

    // For side-by-side, we only show one line number per panel usually, but the DiffLine struct has both.
    // However, render_diff_line is generic.
    // In side-by-side, we might want to show only the relevant line number?
    // The original code did:
    // old: format!("{:4} ", diff_line.old_line_num.unwrap_or(0))
    // new: format!("{:4} ", diff_line.new_line_num.unwrap_or(0))
    
    // But render_diff_line shows both if present.
    // If I reuse render_diff_line, I'll get "   1:   2 " which might be too wide or redundant for side-by-side.
    // But side-by-side split is 50%.
    
    // Let's stick to the original behavior for side-by-side: show only the relevant line number.
    // So I should NOT reuse render_diff_line exactly as is, or I should make it configurable.
    // Or I can just implement a specific one for side-by-side.
    
    // Actually, looking at the original code:
    // old_spans.push(Span::styled(format!("{:4} ", diff_line.old_line_num...), ...))
    
    // So I should probably customize it.
    // But I already used `render_diff_line` in the loop replacement above!
    // So I MUST define `render_diff_line` now.
    
    // If I use the same `render_diff_line` as unified, it will show both numbers.
    // This might be acceptable or even better?
    // But if I want to preserve the exact behavior, I should change it.
    
    // Let's define `render_diff_line` to match `unified_diff` for now.
    // If it looks bad, I can change it.
    // But wait, `unified_diff` shows "old:new".
    // In side-by-side, the left panel is "Old", right is "New".
    // Left panel should probably only show old line num.
    // Right panel should probably only show new line num.
    
    // The `render_diff_line` I used in the loop is called for both old and new lines in the pair case.
    // And for single lines too.
    
    // If I use the unified one:
    // Left panel (Removed): shows "  10:     " (if new is None)
    // Right panel (Added): shows "     :  11 " (if old is None)
    // This seems fine! It aligns them.
    
    // But for Context lines, it shows "  10:  10 ".
    // In side-by-side, we put the SAME context line in both panels.
    // So left panel gets "  10:  10 " and right gets "  10:  10 ".
    // That might be redundant.
    
    // Let's use the unified style for now. It's consistent.
    
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
        spans.push(Span::styled(
            &diff_line.content,
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        let highlighted_spans =
            highlight_line_content(&diff_line.content, syntax, syntax_set, theme);
        
        if let Some((ranges, base_bg, highlight_bg)) = intra_line_highlight {
             let diff_spans = crate::ui::diff_utils::apply_diff_highlight(
                highlighted_spans,
                ranges,
                base_bg,
                highlight_bg,
            );
            spans.extend(diff_spans);
        } else if let Some(bg) = bg_color {
            for span in highlighted_spans {
                let mut new_style = span.style;
                new_style = new_style.bg(bg);
                spans.push(Span::styled(span.content, new_style));
            }
        } else {
            spans.extend(highlighted_spans);
        }
    }

    Line::from(spans)
}
}
