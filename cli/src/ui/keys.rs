//! Module providing small utilities related to key events.

use crossterm::event::KeyModifiers;

/// Returns true when `modifiers` is Ctrl, ignoring Shift.
///
/// Shift is excluded so that Ctrl+Shift+letter sequences (common on some
/// keyboard layouts and terminal emulators) still register as Ctrl combos.
/// Other modifiers (Alt, Super, …) prevent a match so that e.g. AltGr
/// sequences on European layouts are not mis-classified as Ctrl.
pub fn is_ctrl(modifiers: KeyModifiers) -> bool {
    (modifiers & !KeyModifiers::SHIFT) == KeyModifiers::CONTROL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_modifiers_is_not_ctrl() {
        assert!(!is_ctrl(KeyModifiers::NONE));
    }

    #[test]
    fn shift_alone_is_not_ctrl() {
        assert!(!is_ctrl(KeyModifiers::SHIFT));
    }

    #[test]
    fn ctrl_alone_is_ctrl() {
        assert!(is_ctrl(KeyModifiers::CONTROL));
    }

    #[test]
    fn shift_ctrl_is_ctrl() {
        assert!(is_ctrl(KeyModifiers::CONTROL | KeyModifiers::SHIFT));
    }

    #[test]
    fn ctrl_alt_is_not_ctrl() {
        assert!(!is_ctrl(KeyModifiers::CONTROL | KeyModifiers::ALT));
    }
}
