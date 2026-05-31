//! String formatters and small animation generators used by the dashboard.
//!
//! Kept together because they're all pure, stateless, and only return
//! `String`/`&'static str`.

use crate::system::format_bytes;

pub(super) fn percent_label(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "N/A".to_string())
}

pub(super) fn rate_label(value: Option<f64>) -> String {
    value
        .map(|value| format!("{}/s", format_bytes(value as u64)))
        .unwrap_or_else(|| "N/A".to_string())
}

pub(super) fn compact_count(value: u64) -> String {
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

pub(super) fn ascii_bar(percent: f32, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "#".repeat(filled), ".".repeat(width - filled))
}

pub(super) fn short_mount(mount: &str) -> String {
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

/// Truncate `value` to at most `max_chars` characters, appending `~` if
/// truncation occurred and `max_chars > 1`.
pub(super) fn truncate(value: &str, max_chars: usize) -> String {
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

pub(super) fn scanline(width: usize, tick: u64) -> String {
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

pub(super) fn activity_indicator(tick: u64) -> &'static str {
    match (tick / 8) % 4 {
        0 => "|",
        1 => "/",
        2 => "-",
        _ => "\\",
    }
}

pub(super) fn animated_route(tick: u64) -> String {
    let mut chars: Vec<char> = "[host]--[net]--[disk]".chars().collect();
    let path = [7, 8, 14, 15];
    let index = path[((tick / 10) as usize) % path.len()];
    if let Some(ch) = chars.get_mut(index) {
        *ch = '*';
    }
    chars.into_iter().collect()
}

pub(super) fn animated_flow(tick: u64) -> String {
    let mut chars: Vec<char> = "[pane]=>[iface]=>[lan]".chars().collect();
    let path = [7, 8, 16, 17];
    let index = path[((tick / 7) as usize) % path.len()];
    if let Some(ch) = chars.get_mut(index) {
        *ch = '*';
    }
    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_appends_marker_when_cut() {
        assert_eq!(truncate("abcdef", 4), "abc~");
    }

    #[test]
    fn truncate_returns_input_when_fits() {
        assert_eq!(truncate("ab", 4), "ab");
    }

    #[test]
    fn short_mount_keeps_root() {
        assert_eq!(short_mount("/"), "/");
    }

    #[test]
    fn short_mount_takes_last_segment() {
        assert_eq!(short_mount("/System/Volumes/Data"), "Data");
    }

    #[test]
    fn compact_count_formats_thousands() {
        assert_eq!(compact_count(2_500), "2.5K");
        assert_eq!(compact_count(7_400_000), "7.4M");
    }
}
