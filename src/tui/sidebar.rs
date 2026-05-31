//! Tall side panels used by the wide-terminal "edex" layout: one for the
//! system overview, one for network traffic.

use ratatui::{Frame, layout::Rect};

use crate::system::{SystemSnapshot, format_bytes, format_duration};

use super::fmt::{
    animated_flow, animated_route, ascii_bar, percent_label, rate_label, short_mount, truncate,
};
use super::theme::{ACCENT, TRACE};
use super::widgets::{
    chart_height, draw_gap, draw_gauge_row, draw_line, draw_memory_grid, draw_panel,
    draw_section_label, draw_series_chart, draw_sparkline_row,
};

pub(super) fn draw_system_sidebar(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &SystemSnapshot,
    tick: u64,
) {
    let inner = draw_panel(frame, area, "PANEL | SYSTEM");
    if inner.height == 0 {
        return;
    }

    let mut y = inner.y;
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!("HOST {}", snapshot.host_name),
            inner.width as usize,
        ),
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
        truncate(
            &format!("kernel {}", snapshot.kernel_version),
            inner.width as usize,
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        format!("uptime {}", format_duration(snapshot.uptime_secs)),
    );
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "CPU USAGE");
    draw_gauge_row(frame, inner, &mut y, "total", snapshot.cpu_usage, ACCENT);
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "load",
        &snapshot.cpu_history,
        Some(100),
        TRACE,
    );
    for core in snapshot.cpu_cores.iter().take(2) {
        draw_line(
            frame,
            inner,
            &mut y,
            format!(
                "{} {} {:>4.0}%",
                truncate(&core.name, 5),
                ascii_bar(core.usage, inner.width.saturating_sub(11) as usize),
                core.usage
            ),
        );
    }
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "MEMORY");
    draw_gauge_row(frame, inner, &mut y, "RAM", snapshot.memory_percent, ACCENT);
    draw_memory_grid(frame, inner, &mut y, snapshot.memory_percent);
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!(
                "{} / {}",
                format_bytes(snapshot.memory_used),
                format_bytes(snapshot.memory_total)
            ),
            inner.width as usize,
        ),
    );
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "DISK");
    for disk in snapshot
        .disks
        .iter()
        .take(inner.bottom().saturating_sub(y) as usize)
    {
        draw_line(
            frame,
            inner,
            &mut y,
            truncate(
                &format!(
                    "{} {}",
                    short_mount(&disk.mount),
                    percent_label(disk.percent)
                ),
                inner.width as usize,
            ),
        );
    }

    if y < inner.bottom() {
        draw_line(
            frame,
            inner,
            &mut y,
            truncate(&animated_route(tick), inner.width as usize),
        );
    }
}

pub(super) fn draw_network_sidebar(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &SystemSnapshot,
    tick: u64,
) {
    let inner = draw_panel(frame, area, "PANEL | NETWORK");
    if inner.height == 0 {
        return;
    }

    let mut y = inner.y;
    draw_section_label(frame, inner, &mut y, "NETWORK STATUS");
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!(
                "iface {}",
                snapshot.active_interface.as_deref().unwrap_or("N/A")
            ),
            inner.width as usize,
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!("IPv4 {}", snapshot.primary_ipv4.as_deref().unwrap_or("N/A")),
            inner.width as usize,
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!("IPv6 {}", snapshot.primary_ipv6.as_deref().unwrap_or("N/A")),
            inner.width as usize,
        ),
    );
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!(
                "in {} out {}",
                format_bytes(snapshot.total_received),
                format_bytes(snapshot.total_transmitted)
            ),
            inner.width as usize,
        ),
    );
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "DOWN TRAFFIC");
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!("rate {}", rate_label(snapshot.rx_per_sec)),
            inner.width as usize,
        ),
    );
    let down_height = chart_height(inner, y, 5);
    draw_series_chart(
        frame,
        Rect::new(inner.x, y, inner.width, down_height),
        &snapshot.rx_history,
        ACCENT,
    );
    y = y.saturating_add(down_height);
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "UP TRAFFIC");
    draw_line(
        frame,
        inner,
        &mut y,
        truncate(
            &format!("rate {}", rate_label(snapshot.tx_per_sec)),
            inner.width as usize,
        ),
    );
    let up_height = chart_height(inner, y, 5);
    draw_series_chart(
        frame,
        Rect::new(inner.x, y, inner.width, up_height),
        &snapshot.tx_history,
        TRACE,
    );
    y = y.saturating_add(up_height);
    draw_gap(&mut y, inner.bottom());

    draw_section_label(frame, inner, &mut y, "INTERFACES");
    for interface in snapshot
        .interfaces
        .iter()
        .take(inner.bottom().saturating_sub(y) as usize)
    {
        let state = match interface.is_up {
            Some(true) => "up",
            Some(false) => "down",
            None => "N/A",
        };
        draw_line(
            frame,
            inner,
            &mut y,
            truncate(
                &format!(
                    "{} {} rx {} tx {}",
                    interface.name,
                    state,
                    format_bytes(interface.received),
                    format_bytes(interface.transmitted)
                ),
                inner.width as usize,
            ),
        );
    }

    if y < inner.bottom() {
        draw_line(
            frame,
            inner,
            &mut y,
            truncate(&animated_flow(tick), inner.width as usize),
        );
    }
}
