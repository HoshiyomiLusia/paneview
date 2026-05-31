//! Accessory strip below the panes: filesystem listing + on-screen keyboard.

use std::fs;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::App;

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
        draw_filesystem(frame, chunks[0]);
        draw_keyboard(frame, chunks[1], app);
    } else if let Some(keyboard_area) = chunks.first() {
        draw_keyboard(frame, *keyboard_area, app);
    }
}

fn draw_filesystem(frame: &mut Frame<'_>, area: Rect) {
    let inner = draw_panel(frame, area, "Filesystem");
    if inner.height == 0 {
        return;
    }

    let (path, entries) = current_dir_entries();
    let mut y = inner.y;
    draw_line(frame, inner, &mut y, truncate(&path, inner.width as usize));

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
            let Some(entry) = entries.get(index) else {
                continue;
            };
            let cell = truncate(entry, column_width.saturating_sub(1));
            line.push_str(&format!("{cell:<column_width$}"));
        }
        draw_line(frame, inner, &mut y, line);
    }
}

fn current_dir_entries() -> (String, Vec<String>) {
    let path = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let path_label = path.display().to_string();
    let mut entries = fs::read_dir(&path)
        .map(|read_dir| {
            read_dir
                .filter_map(Result::ok)
                .map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let prefix = match entry.file_type() {
                        Ok(file_type) if file_type.is_dir() => "[d]",
                        Ok(_) => "[f]",
                        Err(_) => "[?]",
                    };
                    format!("{prefix} {name}")
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|_| vec!["N/A".to_string()]);
    entries.sort_by_key(|entry| entry.to_ascii_lowercase());
    (path_label, entries)
}
