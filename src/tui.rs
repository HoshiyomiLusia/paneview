use std::io::{Stdout, stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline, Wrap},
};

use crate::app::App;
use crate::layout::PaneId;
use crate::pane::Pane;
use crate::system::{SystemSnapshot, format_bytes, format_duration};

pub fn init_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let regions = Regions::from(area, app.show_system_panel());

    if let Some(system_area) = regions.system {
        draw_system(frame, system_area, app.system_snapshot());
    }

    draw_panes(frame, regions.panes, app);
    draw_status(frame, regions.status, app);
}

pub fn pane_region(area: Rect, show_system: bool) -> Rect {
    Regions::from(area, show_system).panes
}

struct Regions {
    panes: Rect,
    system: Option<Rect>,
    status: Rect,
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
                status,
            };
        }

        let system_height = dashboard_height(content.height);
        Self {
            panes: Rect::new(
                content.x,
                content.y.saturating_add(system_height),
                content.width,
                content.height.saturating_sub(system_height),
            ),
            system: Some(Rect::new(
                content.x,
                content.y,
                content.width,
                system_height,
            )),
            status,
        }
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

fn draw_panes(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rects = app.layout().rects(area);

    for pane in app.panes_in_layout_order() {
        let Some(rect) = rects.get(&pane.id()) else {
            continue;
        };
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        draw_pane(frame, *rect, pane, pane.id() == app.focused());
    }
}

fn draw_pane(frame: &mut Frame<'_>, area: Rect, pane: &Pane, focused: bool) {
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let state = if pane.is_alive() {
        "running".to_string()
    } else {
        pane.exit_status().unwrap_or("exited").to_string()
    };
    let title = format!(" pane {} | {} | {} ", pane.id(), pane.shell_name(), state);

    let paragraph = Paragraph::new(pane.screen_text())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn draw_system(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block = Block::default().borders(Borders::ALL).title(" dashboard ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if inner.height < 8 {
        draw_compact_dashboard(frame, inner, snapshot);
        return;
    }

    let top_height = (inner.height / 2).max(4);
    let top = Rect::new(inner.x, inner.y, inner.width, top_height);
    let bottom = Rect::new(
        inner.x,
        inner.y.saturating_add(top_height),
        inner.width,
        inner.height.saturating_sub(top_height),
    );

    let [system_area, cpu_area, memory_area] = split_columns(top, 28, 36);
    draw_system_card(frame, system_area, snapshot);
    draw_cpu_card(frame, cpu_area, snapshot);
    draw_memory_card(frame, memory_area, snapshot);

    if bottom.height > 0 {
        let [network_area, disk_area, interface_area] = split_columns(bottom, 32, 40);
        draw_network_card(frame, network_area, snapshot);
        draw_disk_card(frame, disk_area, snapshot);
        draw_interfaces_card(frame, interface_area, snapshot);
    }
}

fn draw_compact_dashboard(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
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
        Color::Green,
    );
}

fn draw_system_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
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
        truncate("[host]--[net]--[disk]", inner.width as usize),
    );
}

fn draw_cpu_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "CPU");
    let mut y = inner.y;
    draw_gauge_row(
        frame,
        inner,
        &mut y,
        "total",
        snapshot.cpu_usage,
        Color::LightGreen,
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "trend",
        &snapshot.cpu_history,
        Some(100),
        Color::Green,
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
    draw_gauge_row(
        frame,
        inner,
        &mut y,
        "RAM",
        snapshot.memory_percent,
        Color::LightBlue,
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "trend",
        &snapshot.memory_history,
        Some(100),
        Color::Blue,
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

fn draw_network_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "Network");
    let mut y = inner.y;
    draw_line(
        frame,
        inner,
        &mut y,
        format!(
            "down {}  up {}",
            rate_label(snapshot.rx_per_sec),
            rate_label(snapshot.tx_per_sec)
        ),
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "down",
        &snapshot.rx_history,
        None,
        Color::Cyan,
    );
    draw_sparkline_row(
        frame,
        inner,
        &mut y,
        "up",
        &snapshot.tx_history,
        None,
        Color::Magenta,
    );
    draw_gap(&mut y, inner.bottom());
    draw_line(
        frame,
        inner,
        &mut y,
        truncate("[pane]=>[iface]=>[lan]", inner.width as usize),
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
            Color::Yellow,
        );
    }
}

fn draw_interfaces_card(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot) {
    let inner = draw_panel(frame, area, "Interfaces");
    let mut y = inner.y;
    for interface in snapshot.interfaces.iter().take(inner.height as usize) {
        let state = match interface.is_up {
            Some(true) => Span::styled("up", Style::default().fg(Color::Green)),
            Some(false) => Span::styled("down", Style::default().fg(Color::Red)),
            None => Span::styled("N/A", Style::default().fg(Color::DarkGray)),
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
                    " {}",
                    truncate(&ips, inner.width.saturating_sub(13) as usize)
                )),
            ]),
        );
    }
}

fn draw_panel(frame: &mut Frame<'_>, area: Rect, title: &str) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

fn split_columns(area: Rect, first_percent: u16, second_percent: u16) -> [Rect; 3] {
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

fn draw_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.height == 0 {
        return;
    }

    let info = format!(
        "mode:normal | pane:{}/{} | {} | msg:{}",
        pane_label(app.focused()),
        app.pane_count(),
        if app.show_system_panel() {
            "sys:on"
        } else {
            "sys:off"
        },
        app.status(),
    );

    let lines = status_lines(&info, area.height, area.width);
    let paragraph =
        Paragraph::new(lines).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(paragraph, area);
}

fn pane_label(id: PaneId) -> String {
    id.to_string()
}

fn status_lines(info: &str, height: u16, width: u16) -> Vec<Line<'static>> {
    let width = width as usize;
    let lines = match height {
        0 => Vec::new(),
        1 => vec![format!(
            "{} | ^Q quit | ^H/J/K/L focus | ^\\ /^- split | ^N new | ^W close | ^S sys",
            info
        )],
        2 => vec![
            info.to_string(),
            "Keys: ^Q quit | ^H/J/K/L focus | ^\\ Vsplit | ^- Hsplit | ^N new | ^W close | ^S sys | ^C interrupt".to_string(),
        ],
        _ => vec![
            info.to_string(),
            "Keys: Ctrl+Q quit | Ctrl+H/J/K/L focus | Ctrl+\\ vertical split | Ctrl+- horizontal split".to_string(),
            "      Ctrl+N new pane | Ctrl+W close pane | Ctrl+S system panel | Ctrl+C send interrupt".to_string(),
        ],
    };

    lines
        .into_iter()
        .map(|line| Line::from(truncate(&line, width)))
        .collect()
}

fn draw_line(frame: &mut Frame<'_>, area: Rect, y: &mut u16, text: String) {
    draw_rich_line(frame, area, y, Line::from(text));
}

fn draw_rich_line(frame: &mut Frame<'_>, area: Rect, y: &mut u16, line: Line<'_>) {
    if *y >= area.bottom() {
        return;
    }
    frame.render_widget(Paragraph::new(line), Rect::new(area.x, *y, area.width, 1));
    *y = y.saturating_add(1);
}

fn draw_gap(y: &mut u16, bottom: u16) {
    if y.saturating_add(1) < bottom {
        *y = y.saturating_add(1);
    }
}

fn draw_gauge_row(
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

fn draw_gauge_row_with_text(
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
        Paragraph::new(truncate(label, label_width as usize)),
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
            .gauge_style(Style::default().fg(color).bg(Color::DarkGray));
        frame.render_widget(gauge, gauge_area);
    }

    *y = y.saturating_add(1);
}

fn draw_sparkline_row(
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
        Paragraph::new(truncate(label, label_width as usize)),
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

fn short_mount(mount: &str) -> String {
    if mount == "/" {
        "/".to_string()
    } else {
        mount
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|part| !part.is_empty())
            .unwrap_or(mount)
            .to_string()
    }
}

fn percent_label(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "N/A".to_string())
}

fn rate_label(value: Option<f64>) -> String {
    value
        .map(|value| format!("{}/s", format_bytes(value as u64)))
        .unwrap_or_else(|| "N/A".to_string())
}

fn ascii_bar(percent: f32, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "#".repeat(filled), ".".repeat(width - filled))
}

fn truncate(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let mut chars = value.chars();
    let mut output = String::new();
    for _ in 0..max_chars {
        let Some(ch) = chars.next() else {
            return value.to_string();
        };
        output.push(ch);
    }

    if chars.next().is_some() && max_chars > 1 {
        output.pop();
        output.push('~');
    }
    output
}
