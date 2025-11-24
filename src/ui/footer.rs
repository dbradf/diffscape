use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_footer(f: &mut Frame, area: Rect) {
    let shortcuts = vec![Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Quit  "),
        Span::styled(
            "j/k",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Files  "),
        Span::styled(
            "d/u",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Scroll  "),
        Span::styled(
            "g/G",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Top/Bottom  "),
        Span::styled(
            "←/→/h/l/H/L",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":H-Scroll  "),
        Span::styled(
            "s",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Toggle View  "),
        Span::styled(
            "?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Hide Help"),
    ])];

    let footer = Paragraph::new(shortcuts)
        .block(Block::default().borders(Borders::ALL).title("Shortcuts"))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    f.render_widget(footer, area);
}
