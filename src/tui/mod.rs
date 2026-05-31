//! Top-level TUI orchestration. The flow is:
//!
//!   1. `init_terminal` / `restore_terminal` bookend raw mode.
//!   2. The main loop computes the available area, asks `pane_region` for
//!      the chunk that PTY panes get (so app can resize them before draw).
//!   3. `draw` partitions that area into `Regions` and dispatches to the
//!      specialised submodule renderers.
//!
//! All the actual drawing lives in the submodules; this file should stay
//! small and stay focused on layout.

mod accessory;
mod dashboard;
mod fmt;
mod keyboard;
mod panes;
mod sidebar;
mod status;
mod theme;
mod widgets;

use std::io::{Stdout, stdout};

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, layout::Rect};

use crate::app::App;

pub fn init_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableBracketedPaste,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let regions = Regions::from(area, app.show_system_panel());

    if let Some(system_region) = regions.system {
        match system_region {
            SystemRegion::Dashboard(area) => {
                dashboard::draw_system(frame, area, app.system_snapshot(), app.animation_tick());
            }
            SystemRegion::SidePanel(area) => {
                sidebar::draw_system_sidebar(
                    frame,
                    area,
                    app.system_snapshot(),
                    app.animation_tick(),
                );
            }
        }
    }

    if let Some(network_area) = regions.network {
        sidebar::draw_network_sidebar(
            frame,
            network_area,
            app.system_snapshot(),
            app.animation_tick(),
        );
    }

    panes::draw_panes(frame, app);
    if let Some(accessory_area) = regions.accessory {
        accessory::draw_accessory(frame, accessory_area, app);
    }
    status::draw_status(frame, regions.status, app);
}

pub fn pane_region(area: Rect, show_system: bool) -> Rect {
    Regions::from(area, show_system).panes
}

struct Regions {
    panes: Rect,
    system: Option<SystemRegion>,
    network: Option<Rect>,
    accessory: Option<Rect>,
    status: Rect,
}

#[derive(Debug, Clone, Copy)]
enum SystemRegion {
    Dashboard(Rect),
    SidePanel(Rect),
}

impl Regions {
    fn from(area: Rect, show_system: bool) -> Self {
        let status_height = status_bar_height(area.height);
        let content = Rect::new(
            area.x,
            area.y,
            area.width,
            area.height.saturating_sub(status_height),
        );
        let status = Rect::new(
            area.x,
            area.y.saturating_add(content.height),
            area.width,
            status_height,
        );

        if !show_system || content.width < 50 || content.height < 18 {
            return Self {
                panes: content,
                system: None,
                network: None,
                accessory: None,
                status,
            };
        }

        if let Some(edex) = Self::edex(content, status) {
            return edex;
        }

        let system_height = dashboard_height(content.height);
        let remaining_height = content.height.saturating_sub(system_height);
        let acc_height = accessory_height(content.width, remaining_height);
        let panes_height = remaining_height.saturating_sub(acc_height);
        let panes_y = content.y.saturating_add(system_height);
        Self {
            panes: Rect::new(content.x, panes_y, content.width, panes_height),
            system: Some(SystemRegion::Dashboard(Rect::new(
                content.x,
                content.y,
                content.width,
                system_height,
            ))),
            network: None,
            accessory: if acc_height > 0 {
                Some(Rect::new(
                    content.x,
                    panes_y.saturating_add(panes_height),
                    content.width,
                    acc_height,
                ))
            } else {
                None
            },
            status,
        }
    }

    fn edex(content: Rect, status: Rect) -> Option<Self> {
        if content.width < 96 || content.height < 22 {
            return None;
        }

        let acc_height = edex_accessory_height(content.width, content.height);
        let top_height = content.height.saturating_sub(acc_height);
        if top_height < 14 {
            return None;
        }

        let side_width = side_panel_width(content.width);
        let network_width = side_width;
        let panes_width = content
            .width
            .saturating_sub(side_width)
            .saturating_sub(network_width);
        if panes_width < 34 {
            return None;
        }

        let top_y = content.y;
        let accessory_y = content.y.saturating_add(top_height);
        let system = Rect::new(content.x, top_y, side_width, top_height);
        let panes = Rect::new(
            content.x.saturating_add(side_width),
            top_y,
            panes_width,
            top_height,
        );
        let network = Rect::new(
            panes.x.saturating_add(panes_width),
            top_y,
            network_width,
            top_height,
        );
        let accessory = if acc_height > 0 {
            Some(Rect::new(content.x, accessory_y, content.width, acc_height))
        } else {
            None
        };

        Some(Self {
            panes,
            system: Some(SystemRegion::SidePanel(system)),
            network: Some(network),
            accessory,
            status,
        })
    }
}

fn dashboard_height(content_height: u16) -> u16 {
    if content_height < 18 {
        return 0;
    }

    let preferred = ((u32::from(content_height) * 45) / 100) as u16;
    preferred.clamp(8, 18).min(content_height.saturating_sub(6))
}

fn status_bar_height(total_height: u16) -> u16 {
    match total_height {
        0 => 0,
        1..=11 => 1,
        12..=17 => 2,
        _ => 3,
    }
}

fn accessory_height(width: u16, available_height: u16) -> u16 {
    if width < 80 || available_height < 11 {
        return 0;
    }

    7.min(available_height.saturating_sub(5))
}

fn edex_accessory_height(width: u16, content_height: u16) -> u16 {
    if width < 96 || content_height < 25 {
        return 0;
    }

    let preferred = (content_height / 4).clamp(7, 12);
    preferred.min(content_height.saturating_sub(14))
}

fn side_panel_width(width: u16) -> u16 {
    match width {
        0..=109 => 22,
        110..=139 => 26,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_terminal_uses_edex_regions() {
        let regions = Regions::from(Rect::new(0, 0, 130, 36), true);

        assert!(matches!(regions.system, Some(SystemRegion::SidePanel(_))));
        assert!(regions.network.is_some());
        assert!(regions.accessory.is_some());
        assert!(regions.panes.width >= 34);
    }

    #[test]
    fn narrow_terminal_keeps_pane_area_usable() {
        let regions = Regions::from(Rect::new(0, 0, 70, 22), true);

        assert!(matches!(regions.system, Some(SystemRegion::Dashboard(_))));
        assert!(regions.network.is_none());
        assert!(regions.panes.height > 0);
    }

    #[test]
    fn system_off_returns_full_pane_area() {
        let regions = Regions::from(Rect::new(0, 0, 100, 30), false);
        assert!(regions.system.is_none());
        assert!(regions.network.is_none());
        assert_eq!(regions.panes.width, 100);
    }
}
