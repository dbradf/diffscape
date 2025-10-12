use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;

use crate::app::App;
use crate::ui::render_ui::ui;

mod app;
mod diff_file;
mod ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Git diff arguments (e.g., "HEAD~1", "main..feature")
    #[arg(default_value = "")]
    diff_args: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let width = terminal.size()?.width;

    // Enable side-by-side view by default if terminal is wide enough
    let mut app = App::new(width >= 100);
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
                        app.scroll_offset = file.line_count().saturating_sub(1);
                    }
                }
                _ => {}
            }
        }
    }
}
