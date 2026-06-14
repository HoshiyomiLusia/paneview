use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

use anyhow::Context;
use crossterm::event::{KeyEvent, KeyEventKind};
use ratatui::layout::Rect;

use crate::input::{InputAction, InputState, ScrollAction, key_labels};
use crate::layout::{PaneId, PaneLayout, SplitDirection};
use crate::pane::Pane;
use crate::system::{SystemMonitor, SystemSnapshot};

/// One screen of scrollback per page-step.
const SCROLL_PAGE_LINES: usize = 20;
/// How often to re-scan the working directory for the filesystem panel.
/// The cwd doesn't change while PaneView runs (child-shell `cd` doesn't
/// propagate to the parent), so this only needs to catch external file
/// additions/removals — once a second is plenty.
const DIR_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Down,
    Up,
    Right,
}

pub struct App {
    panes: HashMap<PaneId, Pane>,
    layout: PaneLayout,
    focused: PaneId,
    next_pane_id: usize,
    show_system_panel: bool,
    should_quit: bool,
    system: SystemMonitor,
    /// Cached rects from the last layout pass. Authoritative — tui and
    /// directional-focus logic must both read from here.
    last_rects: HashMap<PaneId, Rect>,
    /// Height/width the rects were last computed for, so we can skip work
    /// if the area is unchanged.
    last_area: Option<Rect>,
    status: String,
    animation_tick: u64,
    active_keys: HashMap<&'static str, u64>,
    input_state: InputState,
    /// Cached working-directory listing for the filesystem panel, refreshed
    /// at most once per DIR_REFRESH_INTERVAL instead of every frame.
    dir_listing: DirListing,
}

/// Snapshot of the working directory shown in the accessory panel.
pub struct DirListing {
    pub path: String,
    pub entries: Vec<String>,
    refreshed_at: Option<Instant>,
}

impl DirListing {
    fn new() -> Self {
        Self {
            path: String::new(),
            entries: Vec::new(),
            refreshed_at: None,
        }
    }

    fn refresh_if_due(&mut self) {
        let due = match self.refreshed_at {
            None => true,
            Some(at) => at.elapsed() >= DIR_REFRESH_INTERVAL,
        };
        if !due {
            return;
        }

        let path = std::env::current_dir().unwrap_or_else(|_| ".".into());
        self.path = path.display().to_string();
        self.entries = fs::read_dir(&path)
            .map(|read_dir| {
                let mut names = read_dir
                    .filter_map(Result::ok)
                    .map(|entry| {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        let prefix = match entry.file_type() {
                            Ok(file_type) if file_type.is_dir() => "[d]",
                            Ok(_) => "[f]",
                            Err(_) => "[?]",
                        };
                        format!("{prefix} {name}")
                    })
                    .collect::<Vec<_>>();
                names.sort_by_key(|entry| entry.to_ascii_lowercase());
                names
            })
            .unwrap_or_else(|_| vec!["N/A".to_string()]);
        self.refreshed_at = Some(Instant::now());
    }
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let first_id = PaneId(1);
        let first_pane = Pane::spawn(first_id, 24, 80)?;
        let mut panes = HashMap::new();
        panes.insert(first_id, first_pane);

        Ok(Self {
            panes,
            layout: PaneLayout::new(first_id),
            focused: first_id,
            next_pane_id: 2,
            show_system_panel: true,
            should_quit: false,
            system: SystemMonitor::new(),
            last_rects: HashMap::new(),
            last_area: None,
            status: "normal".to_string(),
            animation_tick: 0,
            active_keys: HashMap::new(),
            input_state: InputState::new(),
            dir_listing: DirListing::new(),
        })
    }

    /// Advance one frame. Returns `true` if any pane produced output or
    /// exited this tick; the main loop uses this to drop poll cadence
    /// when nothing is happening.
    pub fn tick(&mut self) -> bool {
        self.animation_tick = self.animation_tick.wrapping_add(1);
        let now = self.animation_tick;
        self.active_keys.retain(|_, expires_at| *expires_at > now);
        let mut activity = false;
        for pane in self.panes.values_mut() {
            if pane.drain_output() {
                activity = true;
            }
        }
        self.system.refresh_if_due();
        self.dir_listing.refresh_if_due();
        activity
    }

    /// Single entry point for key events: updates the visual key tracker
    /// AND routes the event through `InputState` for action dispatch.
    pub fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        self.record_key_event(key);
        if let Some(action) = self.input_state.handle(key) {
            self.handle_action(action)?;
        }
        Ok(())
    }

    /// Direct paste handling — bypasses the prefix/scroll state machine.
    pub fn handle_paste(&mut self, text: &str) -> anyhow::Result<()> {
        let bytes = text.replace('\n', "\r").into_bytes();
        self.handle_action(InputAction::Send(bytes))
    }

    fn record_key_event(&mut self, key: KeyEvent) {
        let labels = key_labels(key);
        if labels.is_empty() {
            return;
        }

        match key.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => {
                let expires_at = self.animation_tick.saturating_add(8);
                for label in labels {
                    self.active_keys.insert(label, expires_at);
                }
            }
            KeyEventKind::Release => {
                for label in labels {
                    self.active_keys.remove(label);
                }
            }
        }
    }

    fn handle_action(&mut self, action: InputAction) -> anyhow::Result<()> {
        match action {
            InputAction::Quit => self.should_quit = true,
            InputAction::Focus(direction) => self.focus(direction),
            InputAction::Split(direction) => self.split_focused(direction)?,
            InputAction::ClosePane => self.close_focused(),
            InputAction::ToggleSystem => {
                self.show_system_panel = !self.show_system_panel;
                // Invalidate cached layout — area changed.
                self.last_area = None;
                self.status = if self.show_system_panel {
                    "system panel shown".to_string()
                } else {
                    "system panel hidden".to_string()
                };
            }
            InputAction::NewPane => self.split_focused(SplitDirection::Vertical)?,
            InputAction::EnterScrollMode => self.enter_scroll_mode(),
            InputAction::ExitScrollMode => self.exit_scroll_mode(),
            InputAction::Scroll(scroll) => self.scroll_focused(scroll),
            InputAction::Send(bytes) => {
                if let Some(pane) = self.panes.get_mut(&self.focused) {
                    pane.write_input(&bytes)?;
                }
            }
        }

        Ok(())
    }

    /// Compute and cache pane rects for the given area. Idempotent on the
    /// same area, so it's safe to call once per frame from main.
    pub fn resize_panes(&mut self, pane_area: Rect) {
        if self.last_area == Some(pane_area) {
            return;
        }
        self.last_rects = self.layout.rects(pane_area);
        self.last_area = Some(pane_area);
        for (id, rect) in &self.last_rects {
            if let Some(pane) = self.panes.get_mut(id) {
                let rows = rect.height.saturating_sub(2).max(1);
                let cols = rect.width.saturating_sub(2).max(1);
                pane.resize(rows, cols);
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn show_system_panel(&self) -> bool {
        self.show_system_panel
    }

    pub fn focused(&self) -> PaneId {
        self.focused
    }

    pub fn cached_rects(&self) -> &HashMap<PaneId, Rect> {
        &self.last_rects
    }

    pub fn dir_listing(&self) -> &DirListing {
        &self.dir_listing
    }

    pub fn panes_in_layout_order(&self) -> Vec<&Pane> {
        self.layout
            .leaves()
            .into_iter()
            .filter_map(|id| self.panes.get(&id))
            .collect()
    }

    pub fn system_snapshot(&self) -> &SystemSnapshot {
        self.system.snapshot()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn animation_tick(&self) -> u64 {
        self.animation_tick
    }

    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    pub fn is_key_active(&self, label: &str) -> bool {
        self.active_keys.contains_key(label)
    }

    pub fn prefix_armed(&self) -> bool {
        self.input_state.prefix_armed()
    }

    pub fn scroll_mode(&self) -> bool {
        self.input_state.scroll_mode()
    }

    fn enter_scroll_mode(&mut self) {
        self.input_state.set_scroll_mode(true);
        self.status = format!("scroll mode (pane {}) — q/Esc to exit", self.focused);
    }

    fn exit_scroll_mode(&mut self) {
        self.input_state.set_scroll_mode(false);
        if let Some(pane) = self.panes.get_mut(&self.focused) {
            pane.snap_to_live();
        }
        self.status = "normal".to_string();
    }

    fn scroll_focused(&mut self, scroll: ScrollAction) {
        let Some(pane) = self.panes.get_mut(&self.focused) else {
            return;
        };
        match scroll {
            ScrollAction::LineUp => pane.scroll_by(1),
            ScrollAction::LineDown => pane.scroll_by(-1),
            ScrollAction::PageUp => pane.scroll_by(SCROLL_PAGE_LINES as isize),
            ScrollAction::PageDown => pane.scroll_by(-(SCROLL_PAGE_LINES as isize)),
            ScrollAction::Top => pane.scroll_to_top(),
            ScrollAction::Bottom => pane.scroll_to_bottom(),
        }
        self.status = format!(
            "scroll mode (pane {}, offset {})",
            self.focused,
            pane.scrollback_offset()
        );
    }

    fn split_focused(&mut self, direction: SplitDirection) -> anyhow::Result<()> {
        let id = PaneId(self.next_pane_id);
        let pane = Pane::spawn(id, 24, 80).with_context(|| format!("failed to spawn pane {id}"))?;

        if !self.layout.split(self.focused, id, direction) {
            return Ok(());
        }

        self.panes.insert(id, pane);
        self.focused = id;
        self.next_pane_id += 1;
        // Layout changed.
        self.last_area = None;
        self.status = match direction {
            SplitDirection::Vertical => format!("created pane {id} with vertical split"),
            SplitDirection::Horizontal => format!("created pane {id} with horizontal split"),
        };
        Ok(())
    }

    fn close_focused(&mut self) {
        if self.panes.len() <= 1 {
            self.status = "cannot close the last pane".to_string();
            return;
        }

        let closing = self.focused;
        if self.layout.remove(closing) {
            self.panes.remove(&closing);
            self.focused = self.layout.first_leaf().unwrap_or(PaneId(1));
            self.last_area = None;
            self.status = format!("closed pane {closing}");
        }
    }

    fn focus(&mut self, direction: FocusDirection) {
        let Some(next) = self.directional_neighbor(direction) else {
            // Fallback: cycle through leaves in layout order.
            let fallback = match direction {
                FocusDirection::Left | FocusDirection::Up => {
                    self.layout.previous_leaf(self.focused)
                }
                FocusDirection::Right | FocusDirection::Down => self.layout.next_leaf(self.focused),
            };

            if let Some(next) = fallback {
                self.focused = next;
                self.status = format!("focused pane {next}");
            }
            return;
        };

        self.focused = next;
        self.status = format!("focused pane {next}");
    }

    /// Pick the best neighbor in `direction`.
    ///
    /// Strategy: first filter by "is this candidate actually on the right
    /// side of the focused pane" using edge geometry rather than centers,
    /// then prefer candidates whose perpendicular axis overlaps the focused
    /// pane's axis (e.g. moving Right, prefer something at the same height).
    /// Ties broken by distance along the move axis.
    fn directional_neighbor(&self, direction: FocusDirection) -> Option<PaneId> {
        let current = *self.last_rects.get(&self.focused)?;

        let candidates: Vec<(PaneId, Rect)> = self
            .last_rects
            .iter()
            .filter_map(|(id, rect)| {
                if *id == self.focused {
                    return None;
                }
                if in_direction(current, *rect, direction) {
                    Some((*id, *rect))
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Prefer candidates with positive overlap on the perpendicular axis;
        // fall back to all of them if none overlap.
        let with_overlap: Vec<&(PaneId, Rect)> = candidates
            .iter()
            .filter(|(_, rect)| axis_overlap(current, *rect, direction) > 0)
            .collect();

        let pool: Vec<&(PaneId, Rect)> = if with_overlap.is_empty() {
            candidates.iter().collect()
        } else {
            with_overlap
        };

        pool.into_iter()
            .min_by_key(|(_, rect)| {
                // Sort: shortest primary distance first; among equal,
                // largest perpendicular overlap wins (negate for min).
                (
                    primary_distance(current, *rect, direction),
                    u32::MAX - axis_overlap(current, *rect, direction),
                )
            })
            .map(|(id, _)| *id)
    }
}

fn in_direction(current: Rect, candidate: Rect, direction: FocusDirection) -> bool {
    // Edges in unsigned coords; saturating_sub keeps us safe at origin.
    let cur_right = current.x.saturating_add(current.width);
    let cur_bottom = current.y.saturating_add(current.height);
    let cand_right = candidate.x.saturating_add(candidate.width);
    let cand_bottom = candidate.y.saturating_add(candidate.height);

    match direction {
        FocusDirection::Left => cand_right <= current.x.saturating_add(1),
        FocusDirection::Right => candidate.x.saturating_add(1) >= cur_right,
        FocusDirection::Up => cand_bottom <= current.y.saturating_add(1),
        FocusDirection::Down => candidate.y.saturating_add(1) >= cur_bottom,
    }
}

fn axis_overlap(current: Rect, candidate: Rect, direction: FocusDirection) -> u32 {
    match direction {
        FocusDirection::Left | FocusDirection::Right => {
            let top = current.y.max(candidate.y);
            let bottom = current
                .y
                .saturating_add(current.height)
                .min(candidate.y.saturating_add(candidate.height));
            u32::from(bottom.saturating_sub(top))
        }
        FocusDirection::Up | FocusDirection::Down => {
            let left = current.x.max(candidate.x);
            let right = current
                .x
                .saturating_add(current.width)
                .min(candidate.x.saturating_add(candidate.width));
            u32::from(right.saturating_sub(left))
        }
    }
}

fn primary_distance(current: Rect, candidate: Rect, direction: FocusDirection) -> u32 {
    let cur_cx = i32::from(current.x) + i32::from(current.width) / 2;
    let cur_cy = i32::from(current.y) + i32::from(current.height) / 2;
    let cand_cx = i32::from(candidate.x) + i32::from(candidate.width) / 2;
    let cand_cy = i32::from(candidate.y) + i32::from(candidate.height) / 2;

    match direction {
        FocusDirection::Left | FocusDirection::Right => (cur_cx - cand_cx).unsigned_abs(),
        FocusDirection::Up | FocusDirection::Down => (cur_cy - cand_cy).unsigned_abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn r(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect::new(x, y, w, h)
    }

    #[test]
    fn right_neighbor_prefers_axis_overlap() {
        // Layout:
        //   +-------+---+
        //   |   A   | B |   B is aligned with A on the y-axis
        //   +-------+---+
        //   |     C     |
        //   +-----------+
        // Moving Right from A should pick B, not C (which has higher
        // center-Y similarity in the old scoring).
        let mut last_rects = HashMap::new();
        last_rects.insert(PaneId(1), r(0, 0, 40, 10));
        last_rects.insert(PaneId(2), r(40, 0, 20, 10));
        last_rects.insert(PaneId(3), r(0, 10, 60, 10));

        let app = stub_app(last_rects, PaneId(1));
        assert_eq!(
            app.directional_neighbor(FocusDirection::Right),
            Some(PaneId(2))
        );
    }

    #[test]
    fn left_with_no_neighbor_returns_none() {
        let mut last_rects = HashMap::new();
        last_rects.insert(PaneId(1), r(0, 0, 40, 10));
        let app = stub_app(last_rects, PaneId(1));
        assert_eq!(app.directional_neighbor(FocusDirection::Left), None);
    }

    #[test]
    fn down_picks_overlapping_pane() {
        let mut last_rects = HashMap::new();
        last_rects.insert(PaneId(1), r(0, 0, 40, 10));
        // Stacked below A, partial overlap on x.
        last_rects.insert(PaneId(2), r(20, 10, 60, 10));
        let app = stub_app(last_rects, PaneId(1));
        assert_eq!(
            app.directional_neighbor(FocusDirection::Down),
            Some(PaneId(2))
        );
    }

    #[test]
    fn dir_listing_refreshes_once_then_throttles() {
        let mut listing = DirListing::new();
        assert!(listing.refreshed_at.is_none());

        // First refresh populates and stamps the time.
        listing.refresh_if_due();
        let first_stamp = listing.refreshed_at;
        assert!(first_stamp.is_some());

        // An immediate second call is within the throttle window, so the
        // timestamp must not advance (i.e. no rescan happened).
        listing.refresh_if_due();
        assert_eq!(listing.refreshed_at, first_stamp);
    }

    /// Build an `App` shaped object for testing `directional_neighbor`
    /// without spawning PTYs. We bypass `Pane::spawn` by leaving the panes
    /// map empty — `directional_neighbor` only reads `last_rects`.
    fn stub_app(last_rects: HashMap<PaneId, Rect>, focused: PaneId) -> App {
        App {
            panes: HashMap::new(),
            layout: PaneLayout::new(focused),
            focused,
            next_pane_id: 99,
            show_system_panel: false,
            should_quit: false,
            system: SystemMonitor::new(),
            last_rects,
            last_area: None,
            status: String::new(),
            animation_tick: 0,
            active_keys: HashMap::new(),
            input_state: InputState::new(),
            dir_listing: DirListing::new(),
        }
    }
}
