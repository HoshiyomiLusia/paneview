use std::collections::HashMap;
use std::fmt;

use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub usize);

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// A vertical divider: panes are placed left and right.
    Vertical,
    /// A horizontal divider: panes are placed top and bottom.
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LayoutNode {
    Leaf(PaneId),
    Split {
        direction: SplitDirection,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneLayout {
    root: LayoutNode,
}

impl PaneLayout {
    pub fn new(initial: PaneId) -> Self {
        Self {
            root: LayoutNode::Leaf(initial),
        }
    }

    pub fn split(&mut self, target: PaneId, new_pane: PaneId, direction: SplitDirection) -> bool {
        split_node(&mut self.root, target, new_pane, direction)
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        let Some(new_root) = remove_node(self.root.clone(), pane) else {
            return false;
        };

        if new_root == self.root {
            return false;
        }

        self.root = new_root;
        true
    }

    pub fn leaves(&self) -> Vec<PaneId> {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves
    }

    pub fn first_leaf(&self) -> Option<PaneId> {
        self.leaves().into_iter().next()
    }

    pub fn rects(&self, area: Rect) -> HashMap<PaneId, Rect> {
        let mut rects = HashMap::new();
        collect_rects(&self.root, area, &mut rects);
        rects
    }

    pub fn next_leaf(&self, current: PaneId) -> Option<PaneId> {
        let leaves = self.leaves();
        let index = leaves.iter().position(|pane| *pane == current)?;
        Some(leaves[(index + 1) % leaves.len()])
    }

    pub fn previous_leaf(&self, current: PaneId) -> Option<PaneId> {
        let leaves = self.leaves();
        let index = leaves.iter().position(|pane| *pane == current)?;
        Some(leaves[(index + leaves.len() - 1) % leaves.len()])
    }
}

fn split_node(
    node: &mut LayoutNode,
    target: PaneId,
    new_pane: PaneId,
    direction: SplitDirection,
) -> bool {
    match node {
        LayoutNode::Leaf(id) if *id == target => {
            *node = LayoutNode::Split {
                direction,
                first: Box::new(LayoutNode::Leaf(*id)),
                second: Box::new(LayoutNode::Leaf(new_pane)),
            };
            true
        }
        LayoutNode::Leaf(_) => false,
        LayoutNode::Split { first, second, .. } => {
            split_node(first, target, new_pane, direction)
                || split_node(second, target, new_pane, direction)
        }
    }
}

fn remove_node(node: LayoutNode, target: PaneId) -> Option<LayoutNode> {
    match node {
        LayoutNode::Leaf(id) => {
            if id == target {
                None
            } else {
                Some(LayoutNode::Leaf(id))
            }
        }
        LayoutNode::Split {
            direction,
            first,
            second,
        } => {
            let first = remove_node(*first, target);
            let second = remove_node(*second, target);

            match (first, second) {
                (None, None) => None,
                (Some(node), None) | (None, Some(node)) => Some(node),
                (Some(first), Some(second)) => Some(LayoutNode::Split {
                    direction,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
            }
        }
    }
}

fn collect_leaves(node: &LayoutNode, leaves: &mut Vec<PaneId>) {
    match node {
        LayoutNode::Leaf(id) => leaves.push(*id),
        LayoutNode::Split { first, second, .. } => {
            collect_leaves(first, leaves);
            collect_leaves(second, leaves);
        }
    }
}

fn collect_rects(node: &LayoutNode, area: Rect, rects: &mut HashMap<PaneId, Rect>) {
    match node {
        LayoutNode::Leaf(id) => {
            rects.insert(*id, area);
        }
        LayoutNode::Split {
            direction,
            first,
            second,
        } => {
            let (first_area, second_area) = split_area(area, *direction);
            collect_rects(first, first_area, rects);
            collect_rects(second, second_area, rects);
        }
    }
}

fn split_area(area: Rect, direction: SplitDirection) -> (Rect, Rect) {
    match direction {
        SplitDirection::Vertical => {
            let first_width = area.width / 2;
            let second_width = area.width.saturating_sub(first_width);
            (
                Rect::new(area.x, area.y, first_width, area.height),
                Rect::new(
                    area.x.saturating_add(first_width),
                    area.y,
                    second_width,
                    area.height,
                ),
            )
        }
        SplitDirection::Horizontal => {
            let first_height = area.height / 2;
            let second_height = area.height.saturating_sub(first_height);
            (
                Rect::new(area.x, area.y, area.width, first_height),
                Rect::new(
                    area.x,
                    area.y.saturating_add(first_height),
                    area.width,
                    second_height,
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_split_places_panes_left_and_right() {
        let first = PaneId(1);
        let second = PaneId(2);
        let mut layout = PaneLayout::new(first);

        assert!(layout.split(first, second, SplitDirection::Vertical));

        let rects = layout.rects(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[&first], Rect::new(0, 0, 50, 20));
        assert_eq!(rects[&second], Rect::new(50, 0, 50, 20));
    }

    #[test]
    fn horizontal_split_places_panes_top_and_bottom() {
        let first = PaneId(1);
        let second = PaneId(2);
        let mut layout = PaneLayout::new(first);

        assert!(layout.split(first, second, SplitDirection::Horizontal));

        let rects = layout.rects(Rect::new(0, 0, 80, 25));
        assert_eq!(rects[&first], Rect::new(0, 0, 80, 12));
        assert_eq!(rects[&second], Rect::new(0, 12, 80, 13));
    }

    #[test]
    fn removing_a_leaf_collapses_its_parent() {
        let first = PaneId(1);
        let second = PaneId(2);
        let third = PaneId(3);
        let mut layout = PaneLayout::new(first);
        layout.split(first, second, SplitDirection::Vertical);
        layout.split(second, third, SplitDirection::Horizontal);

        assert!(layout.remove(second));
        assert_eq!(layout.leaves(), vec![first, third]);

        let rects = layout.rects(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[&first], Rect::new(0, 0, 50, 20));
        assert_eq!(rects[&third], Rect::new(50, 0, 50, 20));
    }
}
