//! PTY pane rendering.
//!
//! Unlike the old `Paragraph::new(pane.screen_text())` approach which threw
//! away every ANSI colour and attribute, this walks `vt100::Screen::cell()`
//! row-by-row and maps each cell to a ratatui `Span`, preserving fg/bg,
//! bold/italic/underline, and rendering the cursor.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::pane::Pane;

use super::theme::{ACCENT, BG, BORDER_DIM, CURSOR_BG, CURSOR_BG_DIM, CURSOR_FG, TEXT, WARN};

pub(super) fn draw_panes(frame: &mut Frame<'_>, app: &App) {
    // Use the cached layout from app instead of recomputing — main loop
    // already called resize_panes() with the same region before drawing.
    let rects = app.cached_rects();
    if rects.is_empty() {
        return;
    }

    for pane in app.panes_in_layout_order() {
        let Some(rect) = rects.get(&pane.id()) else {
            continue;
        };
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let focused = pane.id() == app.focused();
        let scroll = focused && app.scroll_mode();
        draw_pane(frame, *rect, pane, focused, scroll);
    }
}

fn draw_pane(frame: &mut Frame<'_>, area: Rect, pane: &Pane, focused: bool, scroll_mode: bool) {
    let border_style = if scroll_mode {
        Style::default().fg(WARN).add_modifier(Modifier::BOLD)
    } else if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(BORDER_DIM)
    };
    let state = if pane.is_alive() {
        "running".to_string()
    } else {
        pane.exit_status().unwrap_or("exited").to_string()
    };
    let mut title = format!(" pane {} | {} | {} ", pane.id(), pane.shell_name(), state);
    if scroll_mode {
        title.push_str(&format!("[scroll +{}] ", pane.scrollback_offset()));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let lines = screen_to_lines(pane, inner.width, inner.height, focused);
    // No wrap — vt100 has already laid out at the right column count.
    let paragraph = Paragraph::new(lines).style(Style::default().fg(TEXT).bg(BG));
    frame.render_widget(paragraph, inner);
}

/// Convert the pane's vt100 screen into ratatui `Line`s, applying the
/// cursor overlay when appropriate.
fn screen_to_lines(pane: &Pane, width: u16, height: u16, focused: bool) -> Vec<Line<'static>> {
    let screen = pane.screen();
    let (screen_rows, screen_cols) = screen.size();
    let rows = height.min(screen_rows);
    let cols = width.min(screen_cols);

    let (cursor_row, cursor_col, cursor_visible) = pane.cursor();
    // Hide the cursor in scrollback (the screen the user sees isn't live).
    let cursor_visible = cursor_visible && pane.scrollback_offset() == 0;

    let mut lines = Vec::with_capacity(rows as usize);
    for row in 0..rows {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default();

        for col in 0..cols {
            let cell = screen.cell(row, col);
            let (mut style, ch) = match cell {
                Some(cell) => (cell_style(cell), cell_char(cell)),
                None => (Style::default(), ' '),
            };

            // Cursor overlay: invert fg/bg on the cursor cell.
            let is_cursor = cursor_visible && row == cursor_row && col == cursor_col;
            if is_cursor {
                style = style
                    .fg(CURSOR_FG)
                    .bg(if focused { CURSOR_BG } else { CURSOR_BG_DIM });
            }

            if style != current_style && !current_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut current_text),
                    current_style,
                ));
            }
            current_style = style;
            current_text.push(ch);
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, current_style));
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn cell_char(cell: &vt100::Cell) -> char {
    if cell.is_wide_continuation() {
        // The wide character was rendered in the previous column; this
        // slot is just a placeholder.
        return ' ';
    }
    let contents = cell.contents();
    if contents.is_empty() {
        ' '
    } else {
        // ratatui Spans are character-grained, so take the first scalar;
        // combining marks are rare in shell output and would just be
        // dropped here. Acceptable trade-off for now.
        contents.chars().next().unwrap_or(' ')
    }
}

fn cell_style(cell: &vt100::Cell) -> Style {
    let mut fg = vt_color(cell.fgcolor(), TEXT);
    let mut bg = vt_color(cell.bgcolor(), BG);

    if cell.inverse() {
        std::mem::swap(&mut fg, &mut bg);
    }

    let mut modifier = Modifier::empty();
    if cell.bold() {
        modifier |= Modifier::BOLD;
    }
    if cell.dim() {
        modifier |= Modifier::DIM;
    }
    if cell.italic() {
        modifier |= Modifier::ITALIC;
    }
    if cell.underline() {
        modifier |= Modifier::UNDERLINED;
    }

    Style::default().fg(fg).bg(bg).add_modifier(modifier)
}

fn vt_color(color: vt100::Color, default: ratatui::style::Color) -> ratatui::style::Color {
    use ratatui::style::Color as RatColor;
    match color {
        vt100::Color::Default => default,
        vt100::Color::Idx(idx) => indexed_color(idx),
        vt100::Color::Rgb(r, g, b) => RatColor::Rgb(r, g, b),
    }
}

fn indexed_color(idx: u8) -> ratatui::style::Color {
    use ratatui::style::Color as RatColor;
    match idx {
        0 => RatColor::Black,
        1 => RatColor::Red,
        2 => RatColor::Green,
        3 => RatColor::Yellow,
        4 => RatColor::Blue,
        5 => RatColor::Magenta,
        6 => RatColor::Cyan,
        7 => RatColor::Gray,
        8 => RatColor::DarkGray,
        9 => RatColor::LightRed,
        10 => RatColor::LightGreen,
        11 => RatColor::LightYellow,
        12 => RatColor::LightBlue,
        13 => RatColor::LightMagenta,
        14 => RatColor::LightCyan,
        15 => RatColor::White,
        // 256-colour palette: pass through the index — ratatui supports it.
        n => RatColor::Indexed(n),
    }
}
