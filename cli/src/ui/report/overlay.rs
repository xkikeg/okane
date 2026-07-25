//! Modal overlays drawn on top of the report screens.
//!
//! [`Overlay`] is either the quit-confirmation prompt or a scrollable
//! [`ErrorPopup`] showing a full error report.

use std::cmp::{max, min};

use crate::ui::table::NavCommand;

/// A scroll request against a scrollable overlay body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDelta {
    Lines(i16),
    Pages(i16),
    Top,
    Bottom,
}

/// The error popup reuses the table navigation keys for scrolling: a row step
/// scrolls one line, a page step scrolls a page, and first/last jump to the
/// ends.
impl From<NavCommand> for ScrollDelta {
    fn from(cmd: NavCommand) -> Self {
        match cmd {
            NavCommand::Up => ScrollDelta::Lines(-1),
            NavCommand::Down => ScrollDelta::Lines(1),
            NavCommand::PageUp => ScrollDelta::Pages(-1),
            NavCommand::PageDown => ScrollDelta::Pages(1),
            NavCommand::First => ScrollDelta::Top,
            NavCommand::Last => ScrollDelta::Bottom,
        }
    }
}

/// Body of the error modal: a full error report the user scrolls through.
///
/// The message is pre-split into display lines, and the renderer does not
/// re-wrap them (annotate-snippets output is column-aligned — soft wrapping
/// would move the carets away from what they point at). That keeps
/// `lines.len()` the exact rendered line count, so the scroll bound is
/// computable — and testable — without a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup {
    /// Modal title, e.g. `failed to load main.ledger`.
    pub title: String,
    /// The error report, one entry per display line.
    pub lines: Vec<String>,
    /// Index of the first visible line.
    pub scroll: u16,
    /// Last known height of the body. Updated each frame, the same way
    /// [`crate::ui::table::TableNav::viewport_height`] is.
    pub viewport_height: u16,
}

impl ErrorPopup {
    pub fn new(title: String, lines: Vec<String>) -> Self {
        Self {
            title,
            lines,
            scroll: 0,
            viewport_height: 0,
        }
    }

    /// Rows the body can scroll before its last line reaches the bottom of the
    /// viewport. Zero when everything already fits.
    fn max_scroll(&self) -> u16 {
        let lines = u16::try_from(self.lines.len()).unwrap_or(u16::MAX);
        lines.saturating_sub(max(self.viewport_height, 1))
    }

    /// Applies a scroll request, clamped to the scrollable range.
    pub fn scroll(&mut self, delta: ScrollDelta) {
        let page = i32::from(max(self.viewport_height, 1));
        let current = i32::from(self.scroll);
        let target = match delta {
            ScrollDelta::Lines(n) => current + i32::from(n),
            ScrollDelta::Pages(n) => current + i32::from(n) * page,
            ScrollDelta::Top => 0,
            ScrollDelta::Bottom => i32::from(self.max_scroll()),
        };
        self.scroll = target.clamp(0, i32::from(self.max_scroll())) as u16;
    }

    /// Re-clamps the offset after the viewport height changes (terminal resize).
    pub fn clamp(&mut self) {
        self.scroll = min(self.scroll, self.max_scroll());
    }
}

/// Modal overlay drawn on top of the current screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    /// "Quit? y/n" prompt shown when leaving the balance screen.
    QuitConfirm,
    /// A failure the user must acknowledge, shown in full.
    Error(ErrorPopup),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn popup(lines: usize, viewport_height: u16) -> ErrorPopup {
        let mut popup = ErrorPopup::new(
            "failed to load test.ledger".to_owned(),
            (0..lines).map(|i| format!("line {i}")).collect(),
        );
        popup.viewport_height = viewport_height;
        popup
    }

    #[test]
    fn popup_scroll_clamps_at_top() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Lines(-1));
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_scroll_clamps_at_bottom() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Lines(100));
        assert_eq!(p.scroll, 6);
    }

    #[test]
    fn popup_scroll_pinned_when_body_fits() {
        let mut p = popup(5, 10);
        assert_eq!(p.max_scroll(), 0);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_page_scroll_uses_viewport_height() {
        let mut p = popup(100, 4);
        p.scroll(ScrollDelta::Pages(1));
        assert_eq!(p.scroll, 4);
        p.scroll(ScrollDelta::Pages(-1));
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_top_and_bottom_jump() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 6);
        p.scroll(ScrollDelta::Top);
        assert_eq!(p.scroll, 0);
    }

    #[test]
    fn popup_scroll_without_viewport_does_not_panic() {
        // The first key can in principle arrive before a frame has been drawn;
        // an unknown viewport falls back to a single line per page.
        let mut p = popup(10, 0);
        p.scroll(ScrollDelta::Pages(1));
        assert_eq!(p.scroll, 1);
        p.scroll(ScrollDelta::Bottom);
        assert_eq!(p.scroll, 9);
    }

    #[test]
    fn popup_clamps_after_viewport_shrink() {
        let mut p = popup(10, 4);
        p.scroll(ScrollDelta::Bottom);
        p.viewport_height = 10;
        p.clamp();
        assert_eq!(p.scroll, 0);
    }
}
