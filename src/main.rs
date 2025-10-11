use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;
use std::process::Command;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

mod test;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Git diff arguments (e.g., "HEAD~1", "main..feature")
    #[arg(default_value = "")]
    diff_args: String,
}

#[derive(Debug, Clone)]
struct DiffFile {
    name: String,
    status: char, // M, A, D, etc.
    lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
struct DiffLine {
    line_type: LineType,
    old_line_num: Option<u32>,
    new_line_num: Option<u32>,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
enum LineType {
    Context,
    Added,
    Removed,
    Header,
}

struct App {
    files: Vec<DiffFile>,
    selected_file: usize,
    file_list_state: ListState,
    scroll_offset: usize,
    show_side_by_side: bool,
    show_shortcuts: bool,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl App {
    fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            files: Vec::new(),
            selected_file: 0,
            file_list_state: state,
            scroll_offset: 0,
            show_side_by_side: false,
            show_shortcuts: true,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    fn load_diff(&mut self, args: &str) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("diff").arg("--no-prefix");

        if !args.is_empty() {
            for arg in args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        let output = cmd.output()?;
        let diff_text = String::from_utf8_lossy(&output.stdout);

        self.files = parse_diff(&diff_text);

        if !self.files.is_empty() {
            self.file_list_state.select(Some(0));
        }

        Ok(())
    }

    fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = (self.selected_file + 1) % self.files.len();
            self.file_list_state.select(Some(self.selected_file));
            self.scroll_offset = 0;
        }
    }

    fn previous_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = if self.selected_file == 0 {
                self.files.len() - 1
            } else {
                self.selected_file - 1
            };
            self.file_list_state.select(Some(self.selected_file));
            self.scroll_offset = 0;
        }
    }

    fn scroll_down(&mut self) {
        if let Some(file) = self.files.get(self.selected_file)
            && self.scroll_offset + 1 < file.lines.len()
        {
            self.scroll_offset += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn toggle_view_mode(&mut self, width: u16) {
        self.show_side_by_side = width >= 120 && !self.show_side_by_side;
    }

    fn toggle_shortcuts(&mut self) {
        self.show_shortcuts = !self.show_shortcuts;
    }

    fn get_syntax_for_file(&self, filename: &str) -> Option<&syntect::parsing::SyntaxReference> {
        // Try by extension first
        if let Some(extension) = std::path::Path::new(filename).extension()
            && let Some(ext_str) = extension.to_str()
        {
            // Handle TypeScript and JavaScript specifically
            match ext_str {
                "ts" | "tsx" => {
                    // TypeScript isn't in default syntect, use JavaScript syntax
                    return self
                        .syntax_set
                        .find_syntax_by_extension("js")
                        .or_else(|| self.syntax_set.find_syntax_by_name("JavaScript"));
                }
                "js" | "jsx" => return self.syntax_set.find_syntax_by_extension("js"),
                "rs" => return self.syntax_set.find_syntax_by_extension("rs"),
                "py" => return self.syntax_set.find_syntax_by_extension("py"),
                "go" => return self.syntax_set.find_syntax_by_extension("go"),
                "java" => return self.syntax_set.find_syntax_by_extension("java"),
                "cpp" | "cc" | "cxx" => return self.syntax_set.find_syntax_by_extension("cpp"),
                "c" => return self.syntax_set.find_syntax_by_extension("c"),
                "h" | "hpp" => return self.syntax_set.find_syntax_by_extension("h"),
                _ => {}
            }
        }

        // Fall back to the original method
        self.syntax_set
            .find_syntax_for_file(filename)
            .ok()
            .flatten()
    }
}

fn parse_diff(diff_text: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut old_line_num = 0u32;
    let mut new_line_num = 0u32;

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            // Extract filename from "diff --git a/file b/file"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let filename = parts[3].trim_start_matches("b/");
                current_file = Some(DiffFile {
                    name: filename.to_string(),
                    status: 'M', // Default to modified
                    lines: Vec::new(),
                });
            }
        } else if line.starts_with("@@") {
            // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            if let Some(ref mut file) = current_file {
                if let Some((old_info, new_info)) = line.split_once(' ').and_then(|(_, rest)| {
                    rest.split_once(' ').map(|(old, new)| {
                        (old.trim_start_matches('-'), new.trim_start_matches('+'))
                    })
                }) {
                    old_line_num = old_info
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                    new_line_num = new_info
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                }

                file.lines.push(DiffLine {
                    line_type: LineType::Header,
                    old_line_num: None,
                    new_line_num: None,
                    content: line.to_string(),
                });
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            if let Some(ref mut file) = current_file {
                file.lines.push(DiffLine {
                    line_type: LineType::Added,
                    old_line_num: None,
                    new_line_num: Some(new_line_num),
                    content: line[1..].to_string(),
                });
                new_line_num += 1;
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            if let Some(ref mut file) = current_file {
                file.lines.push(DiffLine {
                    line_type: LineType::Removed,
                    old_line_num: Some(old_line_num),
                    new_line_num: None,
                    content: line[1..].to_string(),
                });
                old_line_num += 1;
            }
        } else if line.starts_with(' ')
            && let Some(ref mut file) = current_file
        {
            file.lines.push(DiffLine {
                line_type: LineType::Context,
                old_line_num: Some(old_line_num),
                new_line_num: Some(new_line_num),
                content: line[1..].to_string(),
            });
            old_line_num += 1;
            new_line_num += 1;
        }
    }

    if let Some(file) = current_file {
        files.push(file);
    }

    files
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

fn highlight_line_content<'a>(
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

fn ui(f: &mut Frame, app: &App) {
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
            let status_color = match file.status {
                'A' => Color::Green,
                'D' => Color::Red,
                'M' => Color::Yellow,
                _ => Color::White,
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", file.status),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&file.name),
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

fn render_unified_diff(
    f: &mut Frame,
    area: Rect,
    file: &DiffFile,
    scroll_offset: usize,
    app: &App,
) {
    let visible_lines = (area.height - 2) as usize; // Account for borders
    let _end_line = (scroll_offset + visible_lines).min(file.lines.len());

    let syntax = app.get_syntax_for_file(&file.name);
    let theme = &app.theme_set.themes["base16-ocean.dark"];

    let lines: Vec<Line> = file
        .lines
        .iter()
        .skip(scroll_offset)
        .take(visible_lines)
        .map(|diff_line| {
            let line_num_text = match (&diff_line.old_line_num, &diff_line.new_line_num) {
                (Some(old), Some(new)) => format!("{:4}:{:4} ", old, new),
                (Some(old), None) => format!("{:4}:     ", old),
                (None, Some(new)) => format!("    {:4} ", new),
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
                    highlight_line_content(&diff_line.content, syntax, &app.syntax_set, theme);
                for span in highlighted_spans {
                    let mut new_style = span.style;
                    new_style = new_style.bg(bg);
                    spans.push(Span::styled(span.content, new_style));
                }
            } else {
                // Context lines - just syntax highlight normally
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, &app.syntax_set, theme);
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
                .title(file.name.as_str()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_side_by_side_diff(
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
    let syntax = app.get_syntax_for_file(&file.name);
    let theme = &app.theme_set.themes["base16-ocean.dark"];

    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();

    for diff_line in file.lines.iter().skip(scroll_offset).take(visible_lines) {
        match diff_line.line_type {
            LineType::Context => {
                let highlighted_spans =
                    highlight_line_content(&diff_line.content, syntax, &app.syntax_set, theme);

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
                    highlight_line_content(&diff_line.content, syntax, &app.syntax_set, theme);

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
                    highlight_line_content(&diff_line.content, syntax, &app.syntax_set, theme);

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

    let old_paragraph = Paragraph::new(old_text)
        .block(Block::default().borders(Borders::ALL).title("Old"));

    let new_paragraph = Paragraph::new(new_text)
        .block(Block::default().borders(Borders::ALL).title("New"));

    f.render_widget(old_paragraph, chunks[0]);
    f.render_widget(new_paragraph, chunks[1]);
}

fn render_footer(f: &mut Frame, area: Rect) {
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
            "s",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":Toggle View  "),
        Span::styled(
            "h",
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => app.next_file(),
                KeyCode::Char('k') | KeyCode::Up => app.previous_file(),
                KeyCode::Char('d') | KeyCode::PageDown => {
                    for _ in 0..10 {
                        app.scroll_down();
                    }
                }
                KeyCode::Char('u') | KeyCode::PageUp => {
                    for _ in 0..10 {
                        app.scroll_up();
                    }
                }
                KeyCode::Char('s') => {
                    let width = terminal.size()?.width;
                    app.toggle_view_mode(width);
                }
                KeyCode::Char('h') => app.toggle_shortcuts(),
                KeyCode::Char('g') => app.scroll_offset = 0,
                KeyCode::Char('G') => {
                    if let Some(file) = app.files.get(app.selected_file) {
                        app.scroll_offset = file.lines.len().saturating_sub(1);
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.load_diff(&args.diff_args)?;

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
