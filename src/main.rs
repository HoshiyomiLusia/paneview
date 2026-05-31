mod app;
mod cli;
mod input;
mod layout;
mod pane;
mod system;
mod tui;

use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event};
use ratatui::layout::Rect;

use crate::app::App;
use crate::cli::CliAction;

/// Tick when no PTY activity and no input — ~30 fps.
const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(33);
/// Tick when something is actively producing output — ~60 fps.
const ACTIVE_POLL_INTERVAL: Duration = Duration::from_millis(16);

fn main() -> anyhow::Result<()> {
    if cli::handle_args()? == CliAction::Exit {
        return Ok(());
    }

    let mut terminal = tui::init_terminal().context("failed to initialize terminal")?;
    let run_result = run(&mut terminal);
    let restore_result = tui::restore_terminal(&mut terminal);

    if let Err(err) = restore_result {
        eprintln!("failed to restore terminal: {err:#}");
    }

    run_result
}

fn run(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    let mut app = App::new().context("failed to start initial pane")?;
    let mut last_activity = true;

    loop {
        // Poll for input; the timeout is the only sleep in this loop. When
        // panes are quiet we poll for IDLE_POLL_INTERVAL, when active we
        // poll for ACTIVE_POLL_INTERVAL so the screen feels responsive.
        let poll_timeout = if last_activity {
            ACTIVE_POLL_INTERVAL
        } else {
            IDLE_POLL_INTERVAL
        };

        let mut had_input = false;
        if event::poll(poll_timeout)? {
            // Drain everything that's ready so we batch input.
            while event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(key) => {
                        app.handle_key(key)?;
                        had_input = true;
                    }
                    Event::Paste(text) => {
                        app.handle_paste(&text)?;
                        had_input = true;
                    }
                    Event::Resize(_, _) => {
                        had_input = true;
                    }
                    _ => {}
                }
            }
        }

        let pty_activity = app.tick();
        last_activity = had_input || pty_activity;

        let size = terminal.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let pane_area = tui::pane_region(area, app.show_system_panel());
        app.resize_panes(pane_area);

        terminal.draw(|frame| tui::draw(frame, &app))?;

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
