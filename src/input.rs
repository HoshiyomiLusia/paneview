use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::FocusDirection;
use crate::layout::SplitDirection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    Quit,
    Focus(FocusDirection),
    Split(SplitDirection),
    ClosePane,
    ToggleSystem,
    NewPane,
    Send(Vec<u8>),
}

pub fn event_to_action(key: KeyEvent) -> Option<InputAction> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let Some(action) = global_control_action(key.code) {
            return Some(action);
        }
    }

    key_to_bytes(key).map(InputAction::Send)
}

pub fn key_labels(key: KeyEvent) -> Vec<&'static str> {
    let mut labels = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        labels.push("CTRL");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        labels.push("ALT");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        labels.push("SHIFT");
    }

    if let Some(label) = key_code_label(key.code) {
        if !labels.contains(&label) {
            labels.push(label);
        }
    }

    labels
}

fn key_code_label(code: KeyCode) -> Option<&'static str> {
    match code {
        KeyCode::Esc => Some("ESC"),
        KeyCode::Backspace => Some("BACK"),
        KeyCode::Enter => Some("ENTER"),
        KeyCode::Tab | KeyCode::BackTab => Some("TAB"),
        KeyCode::Left => Some("LEFT"),
        KeyCode::Right => Some("RIGHT"),
        KeyCode::Up => Some("UP"),
        KeyCode::Down => Some("DOWN"),
        KeyCode::Char(' ') => Some("SPACE"),
        KeyCode::Char('`') | KeyCode::Char('~') => Some("~"),
        KeyCode::Char('1') | KeyCode::Char('!') => Some("1"),
        KeyCode::Char('2') | KeyCode::Char('@') => Some("2"),
        KeyCode::Char('3') | KeyCode::Char('#') => Some("3"),
        KeyCode::Char('4') | KeyCode::Char('$') => Some("4"),
        KeyCode::Char('5') | KeyCode::Char('%') => Some("5"),
        KeyCode::Char('6') | KeyCode::Char('^') => Some("6"),
        KeyCode::Char('7') | KeyCode::Char('&') => Some("7"),
        KeyCode::Char('8') | KeyCode::Char('*') => Some("8"),
        KeyCode::Char('9') | KeyCode::Char('(') => Some("9"),
        KeyCode::Char('0') | KeyCode::Char(')') => Some("0"),
        KeyCode::Char('-') | KeyCode::Char('_') => Some("-"),
        KeyCode::Char('=') | KeyCode::Char('+') => Some("="),
        KeyCode::Char('[') | KeyCode::Char('{') => Some("{"),
        KeyCode::Char(']') | KeyCode::Char('}') => Some("}"),
        KeyCode::Char('\\') | KeyCode::Char('|') => Some("\\"),
        KeyCode::Char(';') | KeyCode::Char(':') => Some(";"),
        KeyCode::Char('\'') | KeyCode::Char('"') => Some("'"),
        KeyCode::Char(',') | KeyCode::Char('<') => Some(","),
        KeyCode::Char('.') | KeyCode::Char('>') => Some("."),
        KeyCode::Char('/') | KeyCode::Char('?') => Some("/"),
        KeyCode::Char(c) => match c.to_ascii_uppercase() {
            'A' => Some("A"),
            'B' => Some("B"),
            'C' => Some("C"),
            'D' => Some("D"),
            'E' => Some("E"),
            'F' => Some("F"),
            'G' => Some("G"),
            'H' => Some("H"),
            'I' => Some("I"),
            'J' => Some("J"),
            'K' => Some("K"),
            'L' => Some("L"),
            'M' => Some("M"),
            'N' => Some("N"),
            'O' => Some("O"),
            'P' => Some("P"),
            'Q' => Some("Q"),
            'R' => Some("R"),
            'S' => Some("S"),
            'T' => Some("T"),
            'U' => Some("U"),
            'V' => Some("V"),
            'W' => Some("W"),
            'X' => Some("X"),
            'Y' => Some("Y"),
            'Z' => Some("Z"),
            _ => None,
        },
        _ => None,
    }
}

fn global_control_action(code: KeyCode) -> Option<InputAction> {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(InputAction::Quit),
        KeyCode::Char('h') | KeyCode::Char('H') => Some(InputAction::Focus(FocusDirection::Left)),
        KeyCode::Char('j') | KeyCode::Char('J') => Some(InputAction::Focus(FocusDirection::Down)),
        KeyCode::Char('k') | KeyCode::Char('K') => Some(InputAction::Focus(FocusDirection::Up)),
        KeyCode::Char('l') | KeyCode::Char('L') => Some(InputAction::Focus(FocusDirection::Right)),
        KeyCode::Char('\\') => Some(InputAction::Split(SplitDirection::Vertical)),
        KeyCode::Char('-') | KeyCode::Char('_') => {
            Some(InputAction::Split(SplitDirection::Horizontal))
        }
        KeyCode::Char('w') | KeyCode::Char('W') => Some(InputAction::ClosePane),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(InputAction::ToggleSystem),
        KeyCode::Char('n') | KeyCode::Char('N') => Some(InputAction::NewPane),
        _ => None,
    }
}

fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    let mut bytes = match key.code {
        KeyCode::Char(c) if ctrl => control_char_bytes(c)?,
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        _ => return None,
    };

    if alt {
        let mut prefixed = vec![0x1b];
        prefixed.append(&mut bytes);
        bytes = prefixed;
    }

    Some(bytes)
}

fn control_char_bytes(c: char) -> Option<Vec<u8>> {
    if c == ' ' {
        return Some(vec![0]);
    }

    if c.is_ascii() {
        Some(vec![(c as u8) & 0x1f])
    } else {
        None
    }
}
