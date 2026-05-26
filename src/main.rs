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
use crate::input::event_to_action;

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

    loop {
        drain_events(&mut app)?;
        app.tick();

        let size = terminal.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let pane_area = tui::pane_region(area, app.show_system_panel());
        app.resize_panes(pane_area);

        terminal.draw(|frame| tui::draw(frame, &app))?;

        if app.should_quit() {
            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn drain_events(app: &mut App) -> anyhow::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let event = event::read()?;
        if let Event::Key(key) = event {
            app.record_key_event(key);
            if let Some(action) = event_to_action(key) {
                app.handle_action(action)?;
            }
        } else if let Event::Paste(text) = event {
            app.handle_action(input::InputAction::Send(
                text.replace('\n', "\r").into_bytes(),
            ))?;
        }
    }

    Ok(())
}
