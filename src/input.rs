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
