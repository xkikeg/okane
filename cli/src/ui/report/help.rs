//! The key help shown by `?` / `F1`.
//!
//! The bindings live here as data rather than as one prose blob so each screen
//! can list its own: `Enter` opens a register on the balance screen and does
//! nothing on the register one, and a help that says otherwise is worse than
//! none. [`popup`] renders one screen's table into the plain display lines a
//! [`TextPopup`] scrolls, with the key column padded to the widest key so the
//! descriptions line up.
//!
//! This is the one place the bindings are written down twice, so
//! [`tests::every_documented_key_is_mapped`] types every key the help names
//! back into [`super::event::key_to_message`] to keep the two copies together.

use unicode_width::UnicodeWidthStr;

use super::app::Screen;
use super::overlay::TextPopup;

/// One entry of the help: the keys, and what they do.
struct Binding {
    keys: &'static str,
    description: &'static str,
}

/// A titled group of bindings, drawn with a blank line above it.
struct Section {
    title: &'static str,
    bindings: Vec<Binding>,
}

fn binding(keys: &'static str, description: &'static str) -> Binding {
    Binding { keys, description }
}

/// Line movement, shared by both screens except for what an *item* is: an
/// account on the balance screen, a register entry on the other.
fn moving(item_step: &'static str) -> Section {
    Section {
        title: "Moving",
        bindings: vec![
            binding("↑ / k, C-p", "up one line"),
            binding("↓ / j, C-n", "down one line"),
            binding("K, J", item_step),
            binding("PgUp / PgDn, C-b / C-f", "up / down one page"),
            binding("g / Home, G / End", "first / last line"),
        ],
    }
}

fn balance_sections() -> Vec<Section> {
    vec![
        moving("previous / next account"),
        Section {
            title: "View",
            bindings: vec![
                binding("t", "toggle the flat / tree view"),
                binding("space", "fold / unfold the account (tree)"),
                binding("x", "fold / unfold every account (tree)"),
                binding("Enter", "open the register of the selection"),
            ],
        },
        Section {
            title: "Search",
            bindings: vec![
                binding("/", "search accounts by regex"),
                binding("n, N", "next / previous match, once fixed"),
                binding("C-s, C-r", "incremental search forward / back"),
                binding("Esc", "leave the search"),
            ],
        },
        Section {
            title: "Session",
            bindings: vec![
                binding("r, F5", "reload the ledger from disk"),
                binding("?, F1", "this help"),
                binding("q", "quit (asks to confirm)"),
                binding("C-c", "quit immediately"),
            ],
        },
    ]
}

fn register_sections() -> Vec<Section> {
    vec![
        moving("previous / next entry"),
        Section {
            title: "Session",
            bindings: vec![
                binding("r, F5", "reload the ledger from disk"),
                binding("?, F1", "this help"),
                binding("q, Esc", "back to the balance screen"),
                binding("C-c", "quit immediately"),
            ],
        },
    ]
}

/// The help popup for the screen it was opened from.
pub fn popup(screen: &Screen<'_>) -> TextPopup {
    let (title, sections) = match screen {
        Screen::Balance => (" Key bindings — balance ", balance_sections()),
        Screen::Register(_) => (" Key bindings — register ", register_sections()),
    };
    TextPopup::new(title.to_owned(), render_lines(&sections))
}

/// Formats the sections into display lines: a title, one line per binding with
/// the keys padded to the widest of the whole popup, and a blank line between
/// sections.
fn render_lines(sections: &[Section]) -> Vec<String> {
    let width = sections
        .iter()
        .flat_map(|section| &section.bindings)
        .map(|binding| display_width(binding.keys))
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for (nth, section) in sections.iter().enumerate() {
        if nth > 0 {
            lines.push(String::new());
        }
        lines.push(format!(" {}", section.title));
        for binding in &section.bindings {
            let pad = " ".repeat(width.saturating_sub(display_width(binding.keys)));
            lines.push(format!(
                "   {}{pad}   {}",
                binding.keys, binding.description
            ));
        }
    }
    lines
}

/// Plain (non-CJK) width: the one ratatui lays the glyphs out with, so the
/// padding computed here is the padding the terminal draws. The arrow keys are
/// East-Asian *ambiguous*, which is exactly where the two measures disagree.
fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::super::app::App;
    use super::super::event::key_to_message;
    use super::super::register::RegisterScope;
    use super::super::search::{Search, SearchIntent, SearchMatch, SearchMode, SearchPhase};
    use super::super::testing::{make_account, template};

    #[test]
    fn balance_help_lists_the_balance_only_keys() {
        let text = popup(&Screen::Balance).lines.join("\n");
        assert!(text.contains("tree"), "{text}");
        assert!(text.contains("open the register"), "{text}");
    }

    /// The register screen has no tree, no search and no quit prompt, so its
    /// help must not offer them.
    #[test]
    fn register_help_omits_the_balance_only_keys() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");
        let mut app = App::new("test".to_owned(), Vec::new(), template());
        app.show_register(RegisterScope::Single(account), Vec::new());

        let text = popup(&app.screen).lines.join("\n");
        assert!(!text.contains("tree"), "{text}");
        assert!(!text.contains("search"), "{text}");
        assert!(text.contains("back to the balance screen"), "{text}");
    }

    /// The column is measured in display width, not bytes: the arrow keys are
    /// multi-byte, so the gap starts at a different byte on every other line.
    #[test]
    fn descriptions_are_aligned_in_one_column() {
        let lines = render_lines(&balance_sections());
        let columns: Vec<usize> = lines
            .iter()
            .filter(|line| line.starts_with("   "))
            .map(|line| {
                let gap = line.rfind("   ").expect("a binding line has a gap");
                display_width(&line[..gap + 3])
            })
            .collect();
        assert!(
            !columns.is_empty() && columns.windows(2).all(|w| w[0] == w[1]),
            "descriptions start at different columns: {columns:?}"
        );
    }

    /// Every key the help names is one the UI actually maps.
    ///
    /// The balance app carries a fixed search so the `n`/`N` bindings — which
    /// only mean "next match" once a search has been submitted — are checked in
    /// the state they belong to; every other balance key falls through that
    /// phase unchanged.
    #[test]
    fn every_documented_key_is_mapped() {
        let arena = Bump::new();
        let (_ctx, account) = make_account(&arena, "Assets:Cash");

        let mut balance = App::new("test".to_owned(), Vec::new(), template());
        balance.balance.search = Some(fixed_search());
        let mut register = App::new("test".to_owned(), Vec::new(), template());
        register.show_register(RegisterScope::Single(account), Vec::new());

        for (app, sections) in [
            (&balance, balance_sections()),
            (&register, register_sections()),
        ] {
            for binding in sections.iter().flat_map(|section| &section.bindings) {
                for key in binding.keys.split(", ").flat_map(|k| k.split(" / ")) {
                    let event = parse_key(key).unwrap_or_else(|| panic!("untypable key {key:?}"));
                    assert!(
                        key_to_message(app, event).is_some(),
                        "the help documents {key:?} ({}) but nothing maps it",
                        binding.description
                    );
                }
            }
        }
    }

    fn fixed_search() -> Search {
        Search {
            intent: SearchIntent {
                mode: SearchMode::Modal(SearchPhase::Fixed),
                dir: super::super::search::SearchDirection::Forward,
                input: "a".to_owned(),
                no_previous: false,
                origin: 0,
            },
            matches: Some(Ok(SearchMatch::from(vec![]))),
        }
    }

    /// Turns a help label (`C-s`, `PgUp`, `↑`) back into the key event it
    /// stands for.
    fn parse_key(label: &str) -> Option<KeyEvent> {
        let (code, modifiers) = match label {
            "↑" => (KeyCode::Up, KeyModifiers::NONE),
            "↓" => (KeyCode::Down, KeyModifiers::NONE),
            "Enter" => (KeyCode::Enter, KeyModifiers::NONE),
            "Esc" => (KeyCode::Esc, KeyModifiers::NONE),
            "space" => (KeyCode::Char(' '), KeyModifiers::NONE),
            "PgUp" => (KeyCode::PageUp, KeyModifiers::NONE),
            "PgDn" => (KeyCode::PageDown, KeyModifiers::NONE),
            "Home" => (KeyCode::Home, KeyModifiers::NONE),
            "End" => (KeyCode::End, KeyModifiers::NONE),
            "F1" => (KeyCode::F(1), KeyModifiers::NONE),
            "F5" => (KeyCode::F(5), KeyModifiers::NONE),
            _ => match label.strip_prefix("C-") {
                Some(c) => (KeyCode::Char(one_char(c)?), KeyModifiers::CONTROL),
                None => (KeyCode::Char(one_char(label)?), KeyModifiers::NONE),
            },
        };
        Some(KeyEvent::new(code, modifiers))
    }

    fn one_char(s: &str) -> Option<char> {
        let mut chars = s.chars();
        let c = chars.next()?;
        chars.next().is_none().then_some(c)
    }
}
