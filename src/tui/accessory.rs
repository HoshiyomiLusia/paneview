//! Accessory strip below the panes: filesystem listing + on-screen keyboard.

use std::fmt::Write as _;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::{App, DirListing};

use super::fmt::truncate;
use super::keyboard::draw_keyboard;
use super::widgets::{draw_line, draw_panel};

pub(super) fn draw_accessory(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let chunks = if area.width >= 120 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(34), Constraint::Min(20)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20)])
            .split(area)
    };

    if chunks.len() == 2 {
        draw_filesystem(frame, chunks[0], app.dir_listing());
        draw_keyboard(frame, chunks[1], app);
    } else if let Some(keyboard_area) = chunks.first() {
        draw_keyboard(frame, *keyboard_area, app);
    }
}

fn draw_filesystem(frame: &mut Frame<'_>, area: Rect, listing: &DirListing) {
    let inner = draw_panel(frame, area, "Filesystem");
    if inner.height == 0 {
        return;
    }

    let mut y = inner.y;
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(&listing.path, inner.width as usize),
    );

    let rows = inner.bottom().saturating_sub(y) as usize;
    if rows == 0 {
        return;
    }

    let columns = if inner.width >= 30 { 2 } else { 1 };
    let column_width = (inner.width as usize / columns).max(1);
    for row in 0..rows {
        let mut line = String::new();
        for column in 0..columns {
            let index = row + column * rows;
            let Some(entry) = listing.entries.get(index) else {
                continue;
            };
            let cell = truncate(entry, column_width.saturating_sub(1));
            let _ = write!(line, "{cell:<column_width$}");
        }
        draw_line(frame, inner, &mut y, line);
    }
}
