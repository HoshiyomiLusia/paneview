use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::FocusDirection;
use crate::layout::SplitDirection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    /// Quit the application.
    Quit,
    /// Move the focused pane in the requested direction.
    Focus(FocusDirection),
    /// Split the focused pane.
    Split(SplitDirection),
    /// Close the focused pane.
    ClosePane,
    /// Toggle the system dashboard panel.
    ToggleSystem,
    /// Create a new pane (vertical split by default).
    NewPane,
    /// Enter scroll mode on the focused pane (view scrollback).
    EnterScrollMode,
    /// Scroll the focused pane (only valid inside scroll mode).
    Scroll(ScrollAction),
    /// Leave scroll mode and return to live PTY output.
    ExitScrollMode,
    /// Send raw bytes to the focused pane's PTY.
    Send(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAction {
    LineUp,
    LineDown,
    PageUp,
    PageDown,
    Top,
    Bottom,
}

/// Tracks whether the user just pressed the prefix key and is now mid-chord.
#[derive(Debug, Default)]
pub struct InputState {
    prefix_armed: bool,
    scroll_mode: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefix_armed(&self) -> bool {
        self.prefix_armed
    }

    pub fn scroll_mode(&self) -> bool {
        self.scroll_mode
    }

    pub fn set_scroll_mode(&mut self, on: bool) {
        self.scroll_mode = on;
        if on {
            // Don't carry an armed prefix into scroll mode.
            self.prefix_armed = false;
        }
    }

    /// Translate a key event into the next action, respecting prefix and
    /// scroll-mode state. Returns `None` when the key has no effect.
    pub fn handle(&mut self, key: KeyEvent) -> Option<InputAction> {
        if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
            return None;
        }

        if self.scroll_mode {
            return scroll_mode_action(key).or_else(|| {
                // Any unrecognised key exits scroll mode.
                self.scroll_mode = false;
                None
            });
        }

        if self.prefix_armed {
            self.prefix_armed = false;
            return prefix_action(key);
        }

        if is_prefix(key) {
            self.prefix_armed = true;
            return None;
        }

        // Otherwise the key is destined for the focused pane's PTY.
        key_to_bytes(key).map(InputAction::Send)
    }
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

    if let Some(label) = key_code_label(key.code)
        && !labels.contains(&label)
    {
        labels.push(label);
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

/// Ctrl+B is the prefix. Chosen because tmux uses it by default and it's not
/// a common shell shortcut (bash binds it to `backward-char`, which is rarely
/// noticed when masked).
fn is_prefix(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('b') | KeyCode::Char('B'))
}

/// Action triggered by the key that follows the prefix.
fn prefix_action(key: KeyEvent) -> Option<InputAction> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(InputAction::Quit),
        KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Left => {
            Some(InputAction::Focus(FocusDirection::Left))
        }
        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
            Some(InputAction::Focus(FocusDirection::Down))
        }
        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
            Some(InputAction::Focus(FocusDirection::Up))
        }
        KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Right => {
            Some(InputAction::Focus(FocusDirection::Right))
        }
        KeyCode::Char('|') | KeyCode::Char('\\') => {
            Some(InputAction::Split(SplitDirection::Vertical))
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            Some(InputAction::Split(SplitDirection::Horizontal))
        }
        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('x') | KeyCode::Char('X') => {
            Some(InputAction::ClosePane)
        }
        KeyCode::Char('s') | KeyCode::Char('S') => Some(InputAction::ToggleSystem),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('c') | KeyCode::Char('C') => {
            Some(InputAction::NewPane)
        }
        KeyCode::Char('[') | KeyCode::PageUp => Some(InputAction::EnterScrollMode),
        _ => None,
    }
}

/// Key handling while scroll mode is active. Most navigation keys move the
/// scrollback offset; anything else exits scroll mode at the call site.
fn scroll_mode_action(key: KeyEvent) -> Option<InputAction> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(InputAction::ExitScrollMode),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            Some(InputAction::Scroll(ScrollAction::LineUp))
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            Some(InputAction::Scroll(ScrollAction::LineDown))
        }
        KeyCode::PageUp | KeyCode::Char('b') | KeyCode::Char('B') => {
            Some(InputAction::Scroll(ScrollAction::PageUp))
        }
        KeyCode::PageDown | KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Char(' ') => {
            Some(InputAction::Scroll(ScrollAction::PageDown))
        }
        KeyCode::Home | KeyCode::Char('g') => Some(InputAction::Scroll(ScrollAction::Top)),
        KeyCode::End | KeyCode::Char('G') => Some(InputAction::Scroll(ScrollAction::Bottom)),
        _ => None,
    }
}

fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    // For navigation/function keys, the modifier rides inside the CSI
    // sequence (xterm style). For literal characters and Enter/Backspace
    // /Tab/Esc the Alt modifier is conveyed by a leading ESC byte instead.
    //
    // Distinguish the two cases up front so the encoding is unambiguous.
    let csi_mod = csi_modifier(ctrl, alt, shift);

    enum Kind {
        // Plain key — represented by raw bytes; Alt prepends ESC.
        Literal(Vec<u8>),
        // Navigation/function key — modifier already baked in.
        Navigation(Vec<u8>),
    }

    let kind = match key.code {
        KeyCode::Char(c) if ctrl => Kind::Literal(control_char_bytes(c)?),
        KeyCode::Char(c) => Kind::Literal(c.to_string().into_bytes()),
        KeyCode::Enter => Kind::Literal(b"\r".to_vec()),
        KeyCode::Backspace => Kind::Literal(vec![0x7f]),
        KeyCode::Tab => Kind::Literal(b"\t".to_vec()),
        KeyCode::BackTab => Kind::Navigation(b"\x1b[Z".to_vec()),
        KeyCode::Esc => Kind::Literal(b"\x1b".to_vec()),
        KeyCode::Left => Kind::Navigation(arrow_bytes(b'D', csi_mod)),
        KeyCode::Right => Kind::Navigation(arrow_bytes(b'C', csi_mod)),
        KeyCode::Up => Kind::Navigation(arrow_bytes(b'A', csi_mod)),
        KeyCode::Down => Kind::Navigation(arrow_bytes(b'B', csi_mod)),
        KeyCode::Home => Kind::Navigation(arrow_bytes(b'H', csi_mod)),
        KeyCode::End => Kind::Navigation(arrow_bytes(b'F', csi_mod)),
        KeyCode::Delete => Kind::Navigation(tilde_bytes(3, csi_mod)),
        KeyCode::Insert => Kind::Navigation(tilde_bytes(2, csi_mod)),
        KeyCode::PageUp => Kind::Navigation(tilde_bytes(5, csi_mod)),
        KeyCode::PageDown => Kind::Navigation(tilde_bytes(6, csi_mod)),
        KeyCode::F(n) => Kind::Navigation(function_key_bytes(n, csi_mod)?),
        _ => return None,
    };

    Some(match kind {
        Kind::Literal(bytes) if alt => {
            let mut prefixed = Vec::with_capacity(bytes.len() + 1);
            prefixed.push(0x1b);
            prefixed.extend_from_slice(&bytes);
            prefixed
        }
        Kind::Literal(bytes) | Kind::Navigation(bytes) => bytes,
    })
}

fn csi_modifier(ctrl: bool, alt: bool, shift: bool) -> Option<u8> {
    let mut mask = 0u8;
    if shift {
        mask |= 0b001;
    }
    if alt {
        mask |= 0b010;
    }
    if ctrl {
        mask |= 0b100;
    }
    if mask == 0 { None } else { Some(mask + 1) }
}

fn arrow_bytes(letter: u8, modifier: Option<u8>) -> Vec<u8> {
    match modifier {
        None => vec![0x1b, b'[', letter],
        Some(m) => vec![0x1b, b'[', b'1', b';', b'0' + m, letter],
    }
}

fn tilde_bytes(number: u8, modifier: Option<u8>) -> Vec<u8> {
    match modifier {
        None => vec![0x1b, b'[', b'0' + number, b'~'],
        Some(m) => vec![0x1b, b'[', b'0' + number, b';', b'0' + m, b'~'],
    }
}

fn function_key_bytes(n: u8, modifier: Option<u8>) -> Option<Vec<u8>> {
    // F1-F4 use SS3 (\x1bO?), F5+ use CSI with parameter codes per xterm.
    let plain: Vec<u8> = match n {
        1 => vec![0x1b, b'O', b'P'],
        2 => vec![0x1b, b'O', b'Q'],
        3 => vec![0x1b, b'O', b'R'],
        4 => vec![0x1b, b'O', b'S'],
        5 => b"\x1b[15~".to_vec(),
        6 => b"\x1b[17~".to_vec(),
        7 => b"\x1b[18~".to_vec(),
        8 => b"\x1b[19~".to_vec(),
        9 => b"\x1b[20~".to_vec(),
        10 => b"\x1b[21~".to_vec(),
        11 => b"\x1b[23~".to_vec(),
        12 => b"\x1b[24~".to_vec(),
        _ => return None,
    };

    let Some(m) = modifier else {
        return Some(plain);
    };

    // With a modifier, F1-F4 use CSI 1;Nx form (P/Q/R/S as the final byte);
    // tilde-form keys take a parameter slot.
    Some(match n {
        1 => vec![0x1b, b'[', b'1', b';', b'0' + m, b'P'],
        2 => vec![0x1b, b'[', b'1', b';', b'0' + m, b'Q'],
        3 => vec![0x1b, b'[', b'1', b';', b'0' + m, b'R'],
        4 => vec![0x1b, b'[', b'1', b';', b'0' + m, b'S'],
        5 => with_modifier(b"15", m),
        6 => with_modifier(b"17", m),
        7 => with_modifier(b"18", m),
        8 => with_modifier(b"19", m),
        9 => with_modifier(b"20", m),
        10 => with_modifier(b"21", m),
        11 => with_modifier(b"23", m),
        12 => with_modifier(b"24", m),
        _ => return None,
    })
}

fn with_modifier(num: &[u8], modifier: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(num.len() + 6);
    out.extend_from_slice(b"\x1b[");
    out.extend_from_slice(num);
    out.push(b';');
    out.push(b'0' + modifier);
    out.push(b'~');
    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        // KeyEvent::new defaults kind to KeyEventKind::Press.
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn ctrl_s_in_normal_mode_is_passed_to_pty() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(InputAction::Send(vec![0x13])));
    }

    #[test]
    fn ctrl_q_in_normal_mode_is_passed_to_pty() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(InputAction::Send(vec![0x11])));
    }

    #[test]
    fn prefix_then_q_quits() {
        let mut state = InputState::new();
        assert!(
            state
                .handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL))
                .is_none()
        );
        assert!(state.prefix_armed());
        let action = state.handle(press(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Quit));
        assert!(!state.prefix_armed());
    }

    #[test]
    fn prefix_then_arrow_moves_focus() {
        let mut state = InputState::new();
        state.handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL));
        let action = state.handle(press(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Focus(FocusDirection::Right)));
    }

    #[test]
    fn ctrl_up_uses_modifier_csi_sequence() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Up, KeyModifiers::CONTROL));
        assert_eq!(action, Some(InputAction::Send(b"\x1b[1;5A".to_vec())));
    }

    #[test]
    fn shift_left_uses_modifier_csi_sequence() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(action, Some(InputAction::Send(b"\x1b[1;2D".to_vec())));
    }

    #[test]
    fn plain_f1_sends_ss3_sequence() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::F(1), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Send(b"\x1bOP".to_vec())));
    }

    #[test]
    fn ctrl_f5_uses_csi_modifier_form() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::F(5), KeyModifiers::CONTROL));
        assert_eq!(action, Some(InputAction::Send(b"\x1b[15;5~".to_vec())));
    }

    #[test]
    fn alt_letter_gets_esc_prefix() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Char('x'), KeyModifiers::ALT));
        assert_eq!(action, Some(InputAction::Send(b"\x1bx".to_vec())));
    }

    #[test]
    fn scroll_mode_captures_navigation() {
        let mut state = InputState::new();
        state.set_scroll_mode(true);
        let action = state.handle(press(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Scroll(ScrollAction::PageUp)));
    }

    #[test]
    fn scroll_mode_q_exits() {
        let mut state = InputState::new();
        state.set_scroll_mode(true);
        let action = state.handle(press(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::ExitScrollMode));
    }

    #[test]
    fn prefix_then_pipe_splits_vertically() {
        let mut state = InputState::new();
        state.handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL));
        let action = state.handle(press(KeyCode::Char('|'), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Split(SplitDirection::Vertical)));
    }

    #[test]
    fn prefix_then_minus_splits_horizontally() {
        let mut state = InputState::new();
        state.handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL));
        let action = state.handle(press(KeyCode::Char('-'), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Split(SplitDirection::Horizontal)));
    }

    #[test]
    fn prefix_then_x_closes_pane() {
        let mut state = InputState::new();
        state.handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL));
        let action = state.handle(press(KeyCode::Char('x'), KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::ClosePane));
    }

    #[test]
    fn prefix_armed_consumed_even_on_unknown_key() {
        // An unknown key after prefix should consume the prefix (so we
        // don't get stuck waiting) and do nothing.
        let mut state = InputState::new();
        state.handle(press(KeyCode::Char('b'), KeyModifiers::CONTROL));
        assert!(state.prefix_armed());
        let action = state.handle(press(KeyCode::Char('?'), KeyModifiers::NONE));
        assert!(action.is_none());
        assert!(!state.prefix_armed());
    }

    #[test]
    fn ctrl_shift_left_uses_combined_modifier() {
        let mut state = InputState::new();
        let action = state.handle(press(
            KeyCode::Left,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
        // 6 == Ctrl+Shift in the xterm modifier encoding.
        assert_eq!(action, Some(InputAction::Send(b"\x1b[1;6D".to_vec())));
    }

    #[test]
    fn plain_arrow_has_no_modifier_param() {
        let mut state = InputState::new();
        let action = state.handle(press(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(action, Some(InputAction::Send(b"\x1b[A".to_vec())));
    }

    #[test]
    fn unrecognised_scroll_key_exits_mode_silently() {
        let mut state = InputState::new();
        state.set_scroll_mode(true);
        // A letter that isn't a scroll command exits scroll mode.
        let action = state.handle(press(KeyCode::Char('z'), KeyModifiers::NONE));
        assert!(action.is_none());
        assert!(!state.scroll_mode());
    }
}
