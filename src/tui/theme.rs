//! Centralised colour palette. Everything else in `tui` imports from here.

use ratatui::style::Color;

pub(super) const BG: Color = Color::Black;
pub(super) const TEXT: Color = Color::LightGreen;
pub(super) const TEXT_DIM: Color = Color::Green;
pub(super) const BORDER: Color = Color::Green;
pub(super) const BORDER_DIM: Color = Color::DarkGray;
pub(super) const ACCENT: Color = Color::LightGreen;
pub(super) const WARN: Color = Color::Yellow;
pub(super) const BAD: Color = Color::Red;
pub(super) const TRACE: Color = Color::Rgb(70, 220, 120);
/// Cursor block when the pane is focused.
pub(super) const CURSOR_FG: Color = Color::Black;
pub(super) const CURSOR_BG: Color = Color::LightGreen;
/// Cursor block when the pane is unfocused (rendered hollow-ish).
pub(super) const CURSOR_BG_DIM: Color = Color::DarkGray;
