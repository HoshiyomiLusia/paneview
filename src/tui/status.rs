//! Status bar at the bottom of the TUI.

use ratatui::{Frame, layout::Rect, style::Style, text::Line, widgets::Paragraph};

use crate::app::App;
use crate::layout::PaneId;

use super::fmt::{activity_indicator, scanline, truncate};
use super::theme::{ACCENT, BG, BORDER};

pub(super) fn draw_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.height == 0 {
        return;
    }

    let mode = if app.scroll_mode() {
        "SCROLL"
    } else if app.prefix_armed() {
        "PREFIX"
    } else {
        "normal"
    };
    let info = format!(
        "mode:{} | pane:{}/{} | {} | msg:{}",
        mode,
        pane_label(app.focused()),
        app.pane_count(),
        if app.show_system_panel() {
            "sys:on"
        } else {
            "sys:off"
        },
        app.status(),
    );

    let content = if area.height > 1 {
        frame.render_widget(
            Paragraph::new(scanline(area.width as usize, app.animation_tick()))
                .style(Style::default().fg(BORDER).bg(BG)),
            Rect::new(area.x, area.y, area.width, 1),
        );
        Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        )
    } else {
        area
    };

    let lines = status_lines(&info, content.height, content.width, app.animation_tick());
    let paragraph = Paragraph::new(lines).style(Style::default().bg(BG).fg(ACCENT));
    frame.render_widget(paragraph, content);
}

fn pane_label(id: PaneId) -> String {
    id.to_string()
}

fn status_lines(info: &str, height: u16, width: u16, tick: u64) -> Vec<Line<'static>> {
    let width = width as usize;
    let pulse = activity_indicator(tick);
    let lines = match height {
        0 => Vec::new(),
        1 => vec![format!(
            "{} {} | C-b q quit | C-b h/j/k/l focus | C-b | / - split | C-b n new | C-b x close",
            pulse, info
        )],
        2 => vec![
            format!("{} {}", pulse, info),
            "Prefix: Ctrl+B then q quit | h/j/k/l focus | | / - split | n new | x close | s sys | [ scroll".to_string(),
        ],
        _ => vec![
            format!("{} {}", pulse, info),
            "Prefix: Ctrl+B then  q quit  |  h/j/k/l or arrows focus  |  | vertical split  |  - horizontal split".to_string(),
            "                     n new pane  |  x close pane  |  s system panel  |  [ enter scroll mode".to_string(),
        ],
    };

    lines
        .into_iter()
        .map(|line| Line::from(truncate(&line, width)))
        .collect()
}
