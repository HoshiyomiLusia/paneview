//! Shared rendering primitives used by the dashboard, sidebars, and accessory.
//!
//! These are intentionally low-level: each helper draws a single row or
//! decoration into a target area and (where relevant) advances a `y` cursor.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};

use super::fmt::truncate;
use super::theme::{ACCENT, BG, BORDER_DIM, TEXT, TEXT_DIM};

pub(super) fn draw_panel(frame: &mut Frame<'_>, area: Rect, title: &str) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(title.to_string(), Style::default().fg(ACCENT)),
            Span::styled(" ", Style::default()),
        ]))
        .border_style(Style::default().fg(BORDER_DIM))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

pub(super) fn split_columns(area: Rect, first_percent: u16, second_percent: u16) -> [Rect; 3] {
    let first_width = ((u32::from(area.width) * u32::from(first_percent)) / 100) as u16;
    let second_width = ((u32::from(area.width) * u32::from(second_percent)) / 100) as u16;
    let third_width = area
        .width
        .saturating_sub(first_width)
        .saturating_sub(second_width);

    [
        Rect::new(area.x, area.y, first_width, area.height),
        Rect::new(
            area.x.saturating_add(first_width),
            area.y,
            second_width,
            area.height,
        ),
        Rect::new(
            area.x
                .saturating_add(first_width)
                .saturating_add(second_width),
            area.y,
            third_width,
            area.height,
        ),
    ]
}

pub(super) fn draw_line(frame: &mut Frame<'_>, area: Rect, y: &mut u16, text: String) {
    draw_rich_line(frame, area, y, Line::from(text));
}

pub(super) fn draw_rich_line(frame: &mut Frame<'_>, area: Rect, y: &mut u16, line: Line<'_>) {
    if *y >= area.bottom() {
        return;
    }
    frame.render_widget(
        Paragraph::new(line).style(Style::default().fg(TEXT).bg(BG)),
        Rect::new(area.x, *y, area.width, 1),
    );
    *y = y.saturating_add(1);
}

pub(super) fn draw_section_label(frame: &mut Frame<'_>, area: Rect, y: &mut u16, label: &str) {
    if *y >= area.bottom() {
        return;
    }

    frame.render_widget(
        Paragraph::new(truncate(label, area.width as usize)).style(
            Style::default()
                .fg(ACCENT)
                .bg(BG)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(area.x, *y, area.width, 1),
    );
    *y = y.saturating_add(1);
}

pub(super) fn draw_gap(y: &mut u16, bottom: u16) {
    if y.saturating_add(1) < bottom {
        *y = y.saturating_add(1);
    }
}

pub(super) fn draw_memory_grid(
    frame: &mut Frame<'_>,
    area: Rect,
    y: &mut u16,
    percent: Option<f32>,
) {
    let height = area.bottom().saturating_sub(*y).min(3);
    if height == 0 || area.width == 0 {
        return;
    }

    let cells = usize::from(height) * usize::from(area.width);
    let filled = percent
        .map(|value| ((value.clamp(0.0, 100.0) / 100.0) * cells as f32).round() as usize)
        .unwrap_or(0)
        .min(cells);

    for row in 0..height {
        let mut line = String::with_capacity(area.width as usize);
        for column in 0..area.width {
            let index = usize::from(row) * usize::from(area.width) + usize::from(column);
            line.push(if index < filled { '#' } else { '.' });
        }
        frame.render_widget(
            Paragraph::new(line).style(Style::default().fg(TEXT).bg(BG)),
            Rect::new(area.x, y.saturating_add(row), area.width, 1),
        );
    }

    *y = y.saturating_add(height);
}

pub(super) fn chart_height(area: Rect, y: u16, preferred: u16) -> u16 {
    area.bottom().saturating_sub(y).min(preferred)
}

pub(super) fn draw_series_chart(frame: &mut Frame<'_>, area: Rect, data: &[u64], color: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;
    let start = data.len().saturating_sub(width);
    let samples = &data[start..];
    let max_value = samples.iter().copied().max().unwrap_or(1).max(1);
    let left_padding = width.saturating_sub(samples.len());
    let mut lines = Vec::with_capacity(height);

    for row in 0..height {
        let threshold = ((height - row) as u64 * max_value).div_ceil(height as u64);
        let mut line = String::with_capacity(width);
        for column in 0..width {
            if column < left_padding {
                line.push(' ');
                continue;
            }

            let value = samples[column - left_padding];
            if value >= threshold {
                line.push('#');
            } else if (row + column) % 6 == 0 {
                line.push('.');
            } else {
                line.push(' ');
            }
        }
        lines.push(Line::from(line));
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(color).bg(BG)),
        area,
    );
}

pub(super) fn draw_gauge_row(
    frame: &mut Frame<'_>,
    area: Rect,
    y: &mut u16,
    label: &str,
    percent: Option<f32>,
    color: Color,
) {
    let gauge_text = percent
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "N/A".to_string());
    draw_gauge_row_with_text(frame, area, y, label, percent, gauge_text, color);
}

pub(super) fn draw_gauge_row_with_text(
    frame: &mut Frame<'_>,
    area: Rect,
    y: &mut u16,
    label: &str,
    percent: Option<f32>,
    gauge_text: String,
    color: Color,
) {
    if *y >= area.bottom() {
        return;
    }

    let label_width = area.width.min(7);
    let gauge_width = area.width.saturating_sub(label_width);
    let label_area = Rect::new(area.x, *y, label_width, 1);
    let gauge_area = Rect::new(area.x.saturating_add(label_width), *y, gauge_width, 1);

    frame.render_widget(
        Paragraph::new(truncate(label, label_width as usize))
            .style(Style::default().fg(TEXT_DIM).bg(BG)),
        label_area,
    );

    if gauge_width > 0 {
        let ratio = percent
            .map(|value| f64::from(value.clamp(0.0, 100.0)) / 100.0)
            .unwrap_or(0.0);
        let gauge = Gauge::default()
            .ratio(ratio)
            .label(gauge_text)
            .use_unicode(true)
            .style(Style::default().fg(TEXT).bg(BG))
            .gauge_style(Style::default().fg(color).bg(BORDER_DIM));
        frame.render_widget(gauge, gauge_area);
    }

    *y = y.saturating_add(1);
}

pub(super) fn draw_sparkline_row(
    frame: &mut Frame<'_>,
    area: Rect,
    y: &mut u16,
    label: &str,
    data: &[u64],
    max: Option<u64>,
    color: Color,
) {
    if *y >= area.bottom() {
        return;
    }

    let label_width = area.width.min(7);
    let graph_width = area.width.saturating_sub(label_width);
    frame.render_widget(
        Paragraph::new(truncate(label, label_width as usize))
            .style(Style::default().fg(TEXT_DIM).bg(BG)),
        Rect::new(area.x, *y, label_width, 1),
    );

    if graph_width > 0 {
        let mut sparkline = Sparkline::default()
            .data(data.iter().copied())
            .style(Style::default().fg(color));
        if let Some(max) = max {
            sparkline = sparkline.max(max.max(1));
        }
        frame.render_widget(
            sparkline,
            Rect::new(area.x.saturating_add(label_width), *y, graph_width, 1),
        );
    }

    *y = y.saturating_add(1);
}

pub(super) fn draw_scanline(frame: &mut Frame<'_>, area: Rect, tick: u64) {
    use super::fmt::scanline;
    use super::theme::TRACE;
    if area.width == 0 || area.height == 0 {
        return;
    }

    frame.render_widget(
        Paragraph::new(scanline(area.width as usize, tick))
            .style(Style::default().fg(TRACE).bg(BG)),
        Rect::new(area.x, area.y, area.width, 1),
    );
}
