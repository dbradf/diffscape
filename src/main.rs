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

use crate::app::{Action, App};
use crate::ui::render_ui::ui;

mod app;
mod diff_file;
mod ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Diff the staged files.
    #[arg(long)]
    staged: bool,

    /// Diff the given commit.
    #[arg(long)]
    commit: Option<String>,

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
    let diff_args = if let Some(commit) = args.commit {
        format!("{}^..{}", &commit, &commit)
    } else if args.staged {
        "--cached".to_string()
    } else {
        args.diff_args
    };
    app.load_diff(&diff_args)?;

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
    while app.running {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.perform_action(Action::Quit),
                KeyCode::Char('j') | KeyCode::Down => app.perform_action(Action::NextFile),
                KeyCode::Char('k') | KeyCode::Up => app.perform_action(Action::PrevFile),
                KeyCode::Char('d') | KeyCode::PageDown => {
                    app.perform_action(Action::ScrollDown { amount: 10 })
                }
                KeyCode::Char('u') | KeyCode::PageUp => {
                    app.perform_action(Action::ScrollUp { amount: 10 });
                }
                KeyCode::Char('s') => {
                    let width = terminal.size()?.width;
                    app.perform_action(Action::ToggleSplit { width });
                }
                KeyCode::Char('?') => app.perform_action(Action::Help),
                KeyCode::Char('g') => app.perform_action(Action::Top),
                KeyCode::Char('G') => app.perform_action(Action::Bottom),
                KeyCode::Left | KeyCode::Char('h') => {
                    app.perform_action(Action::ScrollLeft { amount: 1 })
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.perform_action(Action::ScrollRight { amount: 1 })
                }
                KeyCode::Char('H') => app.perform_action(Action::ScrollLeft { amount: 10 }),
                KeyCode::Char('L') => app.perform_action(Action::ScrollRight { amount: 10 }),
                _ => {}
            }
        }
    }

    Ok(())
}
