use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::{
    app::App,
    ui::{
        footer::render_footer, side_by_side_diff::render_side_by_side_diff,
        unified_diff::render_unified_diff,
    },
};

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main layout with optional footer
    let (content_area, footer_area) = if app.show_shortcuts {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);
        (main_chunks[0], Some(main_chunks[1]))
    } else {
        (size, None)
    };

    // Content layout (file list and diff)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(content_area);

    // File list
    let files: Vec<ListItem> = app
        .files
        .iter()
        .map(|file| {
            let status_color = match file.get_status() {
                'A' => Color::Green,
                'D' => Color::Red,
                'M' => Color::Yellow,
                _ => Color::White,
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", file.get_status()),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(file.get_name()),
            ]))
        })
        .collect();

    let files_list = List::new(files)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(files_list, chunks[0], &mut app.file_list_state.clone());

    // Diff content
    if let Some(file) = app.files.get(app.selected_file) {
        let diff_area = chunks[1];

        if app.show_side_by_side && diff_area.width >= 120 {
            render_side_by_side_diff(f, diff_area, file, app.scroll_offset, app);
        } else {
            render_unified_diff(f, diff_area, file, app.scroll_offset, app);
        }
    }

    // Footer with keyboard shortcuts (if enabled)
    if let Some(footer_area) = footer_area {
        render_footer(f, footer_area);
    }
}
