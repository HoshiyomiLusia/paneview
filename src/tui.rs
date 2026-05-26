use std::{
    fs,
    io::{Stdout, stdout},
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline, Wrap},
};

use crate::app::App;
use crate::layout::PaneId;
use crate::pane::Pane;
use crate::system::{SystemSnapshot, format_bytes, format_duration};

const BG: Color = Color::Black;
const TEXT: Color = Color::LightGreen;
const TEXT_DIM: Color = Color::Green;
const BORDER: Color = Color::Green;
const BORDER_DIM: Color = Color::DarkGray;
const ACCENT: Color = Color::LightGreen;
const WARN: Color = Color::Yellow;
const BAD: Color = Color::Red;
const TRACE: Color = Color::Rgb(70, 220, 120);

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
        draw_system(
            frame,
            system_area,
            app.system_snapshot(),
            app.animation_tick(),
        );
    }

    draw_panes(frame, regions.panes, app);
    if let Some(accessory_area) = regions.accessory {
        draw_accessory(frame, accessory_area, app);
    }
    draw_status(frame, regions.status, app);
}

pub fn pane_region(area: Rect, show_system: bool) -> Rect {
    Regions::from(area, show_system).panes
}

struct Regions {
    panes: Rect,
    system: Option<Rect>,
    accessory: Option<Rect>,
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
                accessory: None,
                status,
            };
        }

        let system_height = dashboard_height(content.height);
        let remaining_height = content.height.saturating_sub(system_height);
        let accessory_height = accessory_height(content.width, remaining_height);
        let panes_height = remaining_height.saturating_sub(accessory_height);
        let panes_y = content.y.saturating_add(system_height);
        Self {
            panes: Rect::new(content.x, panes_y, content.width, panes_height),
            system: Some(Rect::new(
                content.x,
                content.y,
                content.width,
                system_height,
            )),
            accessory: if accessory_height > 0 {
                Some(Rect::new(
                    content.x,
                    panes_y.saturating_add(panes_height),
                    content.width,
                    accessory_height,
                ))
            } else {
                None
            },
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

fn accessory_height(width: u16, available_height: u16) -> u16 {
    if width < 100 || available_height < 18 {
        return 0;
    }

    8.min(available_height.saturating_sub(7))
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
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(BORDER_DIM)
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
        .style(Style::default().fg(TEXT).bg(BG))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn draw_system(frame: &mut Frame<'_>, area: Rect, snapshot: &SystemSnapshot, tick: u64) {
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

fn draw_panel(frame: &mut Frame<'_>, area: Rect, title: &str) -> Rect {
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

fn draw_accessory(frame: &mut Frame<'_>, area: Rect, app: &App) {
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

fn draw_keyboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let inner = draw_panel(frame, area, "Keyboard");
    let rows: [&[&str]; 5] = [
        &[
            "ESC", "~", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "BACK",
        ],
        &[
            "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "{", "}", "\\",
        ],
        &[
            "CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'",
        ],
        &[
            "SHIFT", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "SHIFT",
        ],
        &[
            "CTRL", "FN", "CMD", "ALT", "SPACE", "ALTGR", "MENU", "CTRL", "UP", "LEFT", "DOWN",
            "RIGHT",
        ],
    ];

    let max_rows = inner.height.min(rows.len() as u16);
    for (offset, row) in rows.iter().take(max_rows as usize).enumerate() {
        draw_key_row(
            frame,
            inner,
            inner.y.saturating_add(offset as u16),
            row,
            app,
        );
    }
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

fn current_dir_entries() -> (String, Vec<String>) {
    let path = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let path_label = path.display().to_string();
    let mut entries = fs::read_dir(&path)
        .map(|read_dir| {
            read_dir
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let prefix = match entry.file_type() {
                        Ok(file_type) if file_type.is_dir() => "[d]",
                        Ok(_) => "[f]",
                        Err(_) => "[?]",
                    };
                    Some(format!("{prefix} {name}"))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|_| vec!["N/A".to_string()]);
    entries.sort_by_key(|entry| entry.to_ascii_lowercase());
    (path_label, entries)
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
            "{} {} | ^Q quit | ^H/J/K/L focus | ^\\ /^- split | ^N new | ^W close | ^S sys",
            pulse, info
        )],
        2 => vec![
            format!("{} {}", pulse, info),
            "Keys: ^Q quit | ^H/J/K/L focus | ^\\ Vsplit | ^- Hsplit | ^N new | ^W close | ^S sys | ^C interrupt".to_string(),
        ],
        _ => vec![
            format!("{} {}", pulse, info),
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
    frame.render_widget(
        Paragraph::new(line).style(Style::default().fg(TEXT).bg(BG)),
        Rect::new(area.x, *y, area.width, 1),
    );
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

fn draw_scanline(frame: &mut Frame<'_>, area: Rect, tick: u64) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    frame.render_widget(
        Paragraph::new(scanline(area.width as usize, tick))
            .style(Style::default().fg(TRACE).bg(BG)),
        Rect::new(area.x, area.y, area.width, 1),
    );
}

fn scanline(width: usize, tick: u64) -> String {
    if width == 0 {
        return String::new();
    }

    let mut chars = vec!['-'; width];
    let pos = ((tick / 2) as usize) % width;
    let pulse = ['=', '=', '>', '*', '>', '=', '='];

    for (offset, ch) in pulse.iter().enumerate() {
        let index = (pos + offset).min(width - 1);
        chars[index] = *ch;
    }

    chars.into_iter().collect()
}

fn activity_indicator(tick: u64) -> &'static str {
    match (tick / 8) % 4 {
        0 => "|",
        1 => "/",
        2 => "-",
        _ => "\\",
    }
}

fn animated_route(tick: u64) -> String {
    let mut chars: Vec<char> = "[host]--[net]--[disk]".chars().collect();
    let path = [7, 8, 14, 15];
    let index = path[((tick / 10) as usize) % path.len()];
    if let Some(ch) = chars.get_mut(index) {
        *ch = '*';
    }
    chars.into_iter().collect()
}

fn animated_flow(tick: u64) -> String {
    let mut chars: Vec<char> = "[pane]=>[iface]=>[lan]".chars().collect();
    let path = [7, 8, 16, 17];
    let index = path[((tick / 7) as usize) % path.len()];
    if let Some(ch) = chars.get_mut(index) {
        *ch = '*';
    }
    chars.into_iter().collect()
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

fn compact_count(value: u64) -> String {
    if value >= 1_000_000_000 {
        format!("{:.1}B", value as f64 / 1_000_000_000.0)
    } else if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
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
