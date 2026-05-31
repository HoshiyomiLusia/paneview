//! Top-bar dashboard renderer with system/CPU/memory/network/disk/interfaces
//! cards plus a compact fallback for short terminals.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};

use crate::system::{SystemSnapshot, format_bytes, format_duration};

use super::fmt::{
    activity_indicator, animated_flow, animated_route, ascii_bar, compact_count, percent_label,
    rate_label, short_mount, truncate,
};
use super::theme::{ACCENT, BAD, BG, BORDER, BORDER_DIM, TEXT_DIM, TRACE, WARN};
use super::widgets::{
    draw_gap, draw_gauge_row, draw_gauge_row_with_text, draw_line, draw_panel, draw_rich_line,
    draw_scanline, draw_sparkline_row, split_columns,
};

pub(super) fn draw_system(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot, tick: u64) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(vec![
            Span::styled(
                " dashboard ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(activity_indicator(tick), Style::default().fg(TEXT_DIM)),
        ]))
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    draw_scanline(frame, inner, tick);
    let content = Rect::new(
        inner.x,
        inner.y.saturating_add(1),
        inner.width,
        inner.height.saturating_sub(1),
    );

    if content.height == 0 {
        return;
    }

    if content.height < 8 {
        draw_compact_dashboard(frame, content, snapshot, tick);
        return;
    }

    let top_height = (content.height / 2).max(4);
    let top = Rect::new(content.x, content.y, content.width, top_height);
    let bottom = Rect::new(
        content.x,
        content.y.saturating_add(top_height),
        content.width,
        content.height.saturating_sub(top_height),
    );

    let [system_area, cpu_area, memory_area] = split_columns(top, 28, 36);
    draw_system_card(frame, system_area, snapshot, tick);
    draw_cpu_card(frame, cpu_area, snapshot);
    draw_memory_card(frame, memory_area, snapshot);

    if bottom.height > 0 {
        let [network_area, disk_area, interface_area] = split_columns(bottom, 32, 40);
        draw_network_card(frame, network_area, snapshot, tick);
        draw_disk_card(frame, disk_area, snapshot);
        draw_interfaces_card(frame, interface_area, snapshot);
    }
}

fn draw_compact_dashboard(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot, tick: u64) {
    let mut y = area.y;
    draw_line(
        frame,
        area,
        &mut y,
        format!(
            "{} | CPU {} | MEM {} | NET down {} up {}",
            snapshot.host_name,
            percent_label(snapshot.cpu_usage),
            percent_label(snapshot.memory_percent),
            rate_label(snapshot.rx_per_sec),
            rate_label(snapshot.tx_per_sec)
        ),
    );
    draw_sparkline_row(
        frame,
        area,
        &mut y,
        "load",
        &snapshot.cpu_history,
        Some(100),
        ACCENT,
    );
    draw_line(
        frame,
        area,
        &mut y,
        truncate(&animated_flow(tick), area.width as usize),
    );
}

fn draw_system_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot, tick: u64) {
    let inner = draw_panel(frame, area, "System");
    let mut y = inner.y;
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(&snapshot.host_name, inner.width as usize),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(&snapshot.os_name, inner.width as usize),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "kernel {}",
            truncate(
                &snapshot.kernel_version,
                inner.width.saturating_sub(7) as usize
            )
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!("uptime {}", format_duration(snapshot.uptime_secs)),
    );
    draw_gap(&mut y, inner.bottom());
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(&animated_route(tick), inner.width as usize),
    );
}

fn draw_cpu_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "CPU");
    let mut y = inner.y;
    draw_gauge_row(frame, inner, &mut y, "total", snapshot.cpu_usage, ACCENT);
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "trend",
        &snapshot.cpu_history,
        Some(100),
        TRACE,
    );

    for core in snapshot
        .cpu_cores
        .iter()
        .take(inner.bottom().saturating_sub(y) as usize)
    {
        draw_line(
            frame,
            inner,
            &mut y,
            format!(
                "{} {} {:>4.0}%",
                truncate(&core.name, 5),
                ascii_bar(core.usage, 10),
                core.usage
            ),
        );
    }
}

fn draw_memory_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "Memory");
    let mut y = inner.y;
    draw_gauge_row(frame, inner, &mut y, "RAM", snapshot.memory_percent, ACCENT);
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "trend",
        &snapshot.memory_history,
        Some(100),
        TEXT_DIM,
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "used {} / {}",
            format_bytes(snapshot.memory_used),
            format_bytes(snapshot.memory_total)
        ),
    );
    draw_gap(&mut y, inner.bottom());
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!(
                "map {}",
                ascii_bar(snapshot.memory_percent.unwrap_or(0.0), 18)
            ),
            inner.width as usize,
        ),
    );
}

fn draw_network_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot, tick: u64) {
    let inner = draw_panel(frame, area, "Network");
    let mut y = inner.y;
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "{} | down {} up {}",
            snapshot.active_interface.as_deref().unwrap_or("N/A"),
            rate_label(snapshot.rx_per_sec),
            rate_label(snapshot.tx_per_sec)
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "IPv4 {} | IPv6 {}",
            snapshot.primary_ipv4.as_deref().unwrap_or("N/A"),
            snapshot.primary_ipv6.as_deref().unwrap_or("N/A")
        ),
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "down",
        &snapshot.rx_history,
        None,
        ACCENT,
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "up",
        &snapshot.tx_history,
        None,
        TRACE,
    );
    draw_gap(&mut y, inner.bottom());
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "total in {} out {}",
            format_bytes(snapshot.total_received),
            format_bytes(snapshot.total_transmitted)
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "packets rx {} tx {}",
            compact_count(snapshot.packets_received),
            compact_count(snapshot.packets_transmitted)
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(&animated_flow(tick), inner.width as usize),
    );
}

fn draw_disk_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "Disk");
    let mut y = inner.y;
    for disk in snapshot
        .disks
        .iter()
        .take(inner.height.saturating_sub(1) as usize)
    {
        draw_gauge_row_with_text(
            frame,
            inner,
            &mut y,
            short_mount(&disk.mount).as_str(),
            disk.percent,
            disk.percent
                .map(|value| {
                    format!(
                        "{} / {} {value:.0}%",
                        format_bytes(disk.used),
                        format_bytes(disk.total)
                    )
                })
                .unwrap_or_else(|| "N/A".to_string()),
            WARN,
        );
    }
}

fn draw_interfaces_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "Interfaces");
    let mut y = inner.y;
    for interface in snapshot.interfaces.iter().take(inner.height as usize) {
        let state = match interface.is_up {
            Some(true) => Span::styled("up", Style::default().fg(ACCENT)),
            Some(false) => Span::styled("down", Style::default().fg(BAD)),
            None => Span::styled("N/A", Style::default().fg(BORDER_DIM)),
        };
        let ips = if interface.ips.is_empty() {
            "N/A".to_string()
        } else {
            interface.ips.join(", ")
        };
        draw_rich_line(
            frame,
            inner,
            &mut y,
            Line::from(vec![
                Span::raw(format!("{} ", truncate(&interface.name, 8))),
                state,
                Span::raw(format!(
                    " {} rx {} tx {}",
                    truncate(&ips, inner.width.saturating_sub(31) as usize),
                    format_bytes(interface.received),
                    format_bytes(interface.transmitted)
                )),
            ]),
        );
    }
}
