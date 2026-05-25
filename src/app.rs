use std::collections::HashMap;

use anyhow::Context;
use ratatui::layout::Rect;

use crate::input::InputAction;
use crate::layout::{PaneId, PaneLayout, SplitDirection};
use crate::pane::Pane;
use crate::system::{SystemMonitor, SystemSnapshot};

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
    last_rects: HashMap<PaneId, Rect>,
    status: String,
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
            status: "normal".to_string(),
        })
    }

    pub fn tick(&mut self) {
        for pane in self.panes.values_mut() {
            pane.drain_output();
        }
        self.system.refresh_if_due();
    }

    pub fn handle_action(&mut self, action: InputAction) -> anyhow::Result<()> {
        match action {
            InputAction::Quit => self.should_quit = true,
            InputAction::Focus(direction) => self.focus(direction),
            InputAction::Split(direction) => self.split_focused(direction)?,
            InputAction::ClosePane => self.close_focused(),
            InputAction::ToggleSystem => {
                self.show_system_panel = !self.show_system_panel;
                self.status = if self.show_system_panel {
                    "system panel shown".to_string()
                } else {
                    "system panel hidden".to_string()
                };
            }
            InputAction::NewPane => self.split_focused(SplitDirection::Vertical)?,
            InputAction::Send(bytes) => {
                if let Some(pane) = self.panes.get_mut(&self.focused) {
                    pane.write_input(&bytes)?;
                }
            }
        }

        Ok(())
    }

    pub fn resize_panes(&mut self, pane_area: Rect) {
        self.last_rects = self.layout.rects(pane_area);
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

    pub fn layout(&self) -> &PaneLayout {
        &self.layout
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

    pub fn pane_count(&self) -> usize {
        self.panes.len()
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
            self.status = format!("closed pane {closing}");
        }
    }

    fn focus(&mut self, direction: FocusDirection) {
        let Some(next) = self.directional_neighbor(direction) else {
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

    fn directional_neighbor(&self, direction: FocusDirection) -> Option<PaneId> {
        let current_rect = *self.last_rects.get(&self.focused)?;
        let current_center = center(current_rect);

        self.last_rects
            .iter()
            .filter(|(id, rect)| {
                **id != self.focused && is_in_direction(current_center, **rect, direction)
            })
            .min_by_key(|(_, rect)| neighbor_score(current_rect, **rect, direction))
            .map(|(id, _)| *id)
    }
}

fn center(rect: Rect) -> (i32, i32) {
    (
        i32::from(rect.x) + i32::from(rect.width) / 2,
        i32::from(rect.y) + i32::from(rect.height) / 2,
    )
}

fn is_in_direction(current_center: (i32, i32), candidate: Rect, direction: FocusDirection) -> bool {
    let candidate_center = center(candidate);
    match direction {
        FocusDirection::Left => candidate_center.0 < current_center.0,
        FocusDirection::Right => candidate_center.0 > current_center.0,
        FocusDirection::Up => candidate_center.1 < current_center.1,
        FocusDirection::Down => candidate_center.1 > current_center.1,
    }
}

fn neighbor_score(current: Rect, candidate: Rect, direction: FocusDirection) -> i32 {
    let current_center = center(current);
    let candidate_center = center(candidate);
    let primary = match direction {
        FocusDirection::Left | FocusDirection::Right => {
            (current_center.0 - candidate_center.0).abs()
        }
        FocusDirection::Up | FocusDirection::Down => (current_center.1 - candidate_center.1).abs(),
    };
    let secondary = match direction {
        FocusDirection::Left | FocusDirection::Right => {
            (current_center.1 - candidate_center.1).abs()
        }
        FocusDirection::Up | FocusDirection::Down => (current_center.0 - candidate_center.0).abs(),
    };

    primary * 100 + secondary
}
