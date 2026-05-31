//! On-screen keyboard visualisation. Renders a full QWERTY layout when the
//! accessory pane is wide enough, falling back to a single-row variant for
//! narrow terminals.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

use super::fmt::truncate;
use super::theme::{BG, TEXT};
use super::widgets::draw_panel;

pub(super) fn draw_keyboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let inner = draw_panel(frame, area, "Keyboard");
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if inner.width < 70 || inner.height < 5 {
        draw_compact_keyboard(frame, inner, app);
        return;
    }

    draw_full_keyboard(frame, inner, app);
}

#[derive(Debug, Clone, Copy)]
struct KeySpec {
    label: &'static str,
    width: u16,
}

const fn key(label: &'static str, width: u16) -> KeySpec {
    KeySpec { label, width }
}

fn draw_full_keyboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let row1 = [
        key("ESC", 5),
        key("~", 3),
        key("1", 3),
        key("2", 3),
        key("3", 3),
        key("4", 3),
        key("5", 3),
        key("6", 3),
        key("7", 3),
        key("8", 3),
        key("9", 3),
        key("0", 3),
        key("-", 3),
        key("=", 3),
        key("BACK", 7),
    ];
    let row2 = [
        key("TAB", 6),
        key("Q", 3),
        key("W", 3),
        key("E", 3),
        key("R", 3),
        key("T", 3),
        key("Y", 3),
        key("U", 3),
        key("I", 3),
        key("O", 3),
        key("P", 3),
        key("{", 3),
        key("}", 3),
        key("\\", 4),
    ];
    let row3 = [
        key("CAPS", 7),
        key("A", 3),
        key("S", 3),
        key("D", 3),
        key("F", 3),
        key("G", 3),
        key("H", 3),
        key("J", 3),
        key("K", 3),
        key("L", 3),
        key(";", 3),
        key("'", 3),
        key("ENTER", 8),
    ];
    let row4 = [
        key("SHIFT", 8),
        key("Z", 3),
        key("X", 3),
        key("C", 3),
        key("V", 3),
        key("B", 3),
        key("N", 3),
        key("M", 3),
        key(",", 3),
        key(".", 3),
        key("/", 3),
        key("SHIFT", 8),
    ];
    let row5 = [
        key("CTRL", 6),
        key("FN", 4),
        key("ALT", 5),
        key("SPACE", 24),
        key("ALTGR", 7),
        key("CTRL", 6),
    ];
    let rows: [(&[KeySpec], u16); 5] = [(&row1, 0), (&row2, 2), (&row3, 4), (&row4, 6), (&row5, 8)];

    for (offset, (row, indent)) in rows.iter().enumerate().take(area.height as usize) {
        draw_key_specs(
            frame,
            area,
            area.y.saturating_add(offset as u16),
            *indent,
            row,
            app,
        );
    }

    if area.height > 5 {
        let y = area.y.saturating_add(5);
        let cluster_width = 17.min(area.width);
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(cluster_width));
        draw_key_cell(frame, Rect::new(x.saturating_add(6), y, 5, 1), "UP", app);
        if y.saturating_add(1) < area.bottom() {
            let y = y.saturating_add(1);
            draw_key_cell(frame, Rect::new(x, y, 5, 1), "LEFT", app);
            draw_key_cell(frame, Rect::new(x.saturating_add(6), y, 5, 1), "DOWN", app);
            draw_key_cell(
                frame,
                Rect::new(x.saturating_add(12), y, 5, 1),
                "RIGHT",
                app,
            );
        }
    }
}

fn draw_compact_keyboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows: [&[&str]; 5] = [
        &[
            "ESC", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "BACK",
        ],
        &["TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
        &["CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", "ENTER"],
        &["SHIFT", "Z", "X", "C", "V", "B", "N", "M", "/", "SHIFT"],
        &["CTRL", "ALT", "SPACE", "UP", "LEFT", "DOWN", "RIGHT"],
    ];

    for (offset, row) in rows.iter().take(area.height as usize).enumerate() {
        draw_key_row(frame, area, area.y.saturating_add(offset as u16), row, app);
    }
}

fn draw_key_specs(
    frame: &mut Frame<'_>,
    area: Rect,
    y: u16,
    indent: u16,
    specs: &[KeySpec],
    app: &App,
) {
    if y >= area.bottom() {
        return;
    }

    let total_width = specs
        .iter()
        .fold(indent, |acc, spec| acc.saturating_add(spec.width))
        .saturating_add(specs.len().saturating_sub(1) as u16);
    let scale = if total_width > area.width {
        area.width as f32 / total_width as f32
    } else {
        1.0
    };
    let mut x = area
        .x
        .saturating_add(((indent as f32) * scale).round() as u16);

    for spec in specs {
        if x >= area.right() {
            break;
        }

        let width = (((spec.width as f32) * scale).round() as u16)
            .max(1)
            .min(area.right().saturating_sub(x));
        draw_key_cell(frame, Rect::new(x, y, width, 1), spec.label, app);
        x = x.saturating_add(width).saturating_add(1);
    }
}

fn draw_key_cell(frame: &mut Frame<'_>, area: Rect, label: &str, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = centered_key_text(label, area.width as usize);
    let style = if app.is_key_active(label) {
        Style::default()
            .fg(BG)
            .bg(TEXT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT).bg(BG)
    };
    frame.render_widget(
        Paragraph::new(text).style(style),
        Rect::new(area.x, area.y, area.width, 1),
    );
}

fn centered_key_text(label: &str, width: usize) -> String {
    let value = format!(" {label} ");
    if value.len() >= width {
        return truncate(&value, width);
    }

    let left = (width - value.len()) / 2;
    let right = width - value.len() - left;
    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn draw_key_row(frame: &mut Frame<'_>, area: Rect, y: u16, labels: &[&str], app: &App) {
    if y >= area.bottom() {
        return;
    }

    let mut spans = Vec::with_capacity(labels.len() * 2);
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        let key_text = format!(" {label} ");
        let style = if app.is_key_active(label) {
            Style::default()
                .fg(BG)
                .bg(TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT).bg(BG)
        };
        spans.push(Span::styled(key_text, style));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        Rect::new(area.x, y, area.width, 1),
    );
}
