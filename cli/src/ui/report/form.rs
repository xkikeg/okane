//! The query-options form opened with `.`.
//!
//! A modal form over [`QueryOptions`]: one row per option, edited in place and
//! applied as a whole. It follows the same component shape as the screens —
//! own state, own [`FormMessage`], own [`OptionsForm::key_to_message`] — and
//! reports back to [`App`](super::app::App) with a [`FormAction`], which is
//! where the effect of applying (re-querying the ledger) is decided.
//!
//! There is no editing *mode*: the focused row takes every printable key, and
//! only the keys no text field wants (`Tab`, the arrows, `Enter`, `Esc`) move
//! between rows and leave. That is what a dialog does everywhere else, and it
//! keeps `j`/`k` — which are text here, not motions — from being stolen.

use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::keys::is_ctrl;

use super::options::{QueryOptions, format_date, parse_date};

/// The options the form edits, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldId {
    Exchange,
    Historical,
    Start,
    End,
    PriceDb,
}

impl FieldId {
    /// Every field, top to bottom. The conversion first (the option most worth
    /// changing mid-session), then the range, then the price DB — which is also
    /// the only one whose change costs a reload.
    const ALL: [FieldId; 5] = [
        FieldId::Exchange,
        FieldId::Historical,
        FieldId::Start,
        FieldId::End,
        FieldId::PriceDb,
    ];

    /// The command-line flag this row stands for. Named after the flag rather
    /// than described in prose: whoever reaches this form got here from a
    /// command line that has the same options on it.
    pub(super) fn label(self) -> &'static str {
        match self {
            FieldId::Exchange => "-X, --exchange",
            FieldId::Historical => "--historical",
            FieldId::Start => "--start",
            FieldId::End => "--end",
            FieldId::PriceDb => "--price-db",
        }
    }

    /// What an empty row shows: the shape the value takes, where there is one
    /// to teach, and otherwise that the option is simply not set.
    pub(super) fn placeholder(self) -> &'static str {
        match self {
            FieldId::Start | FieldId::End => "YYYY-MM-DD",
            FieldId::Exchange | FieldId::PriceDb => "(none)",
            // A flag is never empty.
            FieldId::Historical => "",
        }
    }
}

/// One row's value: free text, or a flag toggled with `space`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Text(String),
    Flag(bool),
}

/// One row of the form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Field {
    id: FieldId,
    value: Value,
}

impl Field {
    pub(super) fn label(&self) -> &'static str {
        self.id.label()
    }

    /// The value as drawn: the typed text, or the flag's state.
    pub(super) fn text(&self) -> &str {
        match &self.value {
            Value::Text(text) => text,
            Value::Flag(true) => "on",
            Value::Flag(false) => "off",
        }
    }

    /// The dim stand-in drawn when [`Self::text`] is empty.
    pub(super) fn placeholder(&self) -> &'static str {
        self.id.placeholder()
    }

    /// Whether this row is typed into (and so carries the cursor when focused).
    pub(super) fn is_text(&self) -> bool {
        matches!(self.value, Value::Text(_))
    }

    /// The trimmed text, or `None` when the option is left unset. Surrounding
    /// space is never meaningful in any of these values, and a stray one is
    /// easier to type than to see.
    fn stated(&self) -> Option<&str> {
        match &self.value {
            Value::Text(text) => Some(text.trim()).filter(|t| !t.is_empty()),
            Value::Flag(_) => None,
        }
    }

    fn flag(&self) -> bool {
        matches!(self.value, Value::Flag(true))
    }
}

/// Messages handled by the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMessage {
    /// Move the focus one row down (wrapping).
    FocusNext,
    /// Move the focus one row up (wrapping).
    FocusPrev,
    /// Append a character to the focused text row.
    Push(char),
    /// Delete the last character of the focused text row.
    Pop,
    /// Empty the focused row (`C-u`); a flag goes back to off.
    Clear,
    /// Flip the focused flag row (`space`).
    Toggle,
    /// Apply every row (`Enter`).
    Submit,
    /// Leave without applying anything (`Esc`).
    Cancel,
}

/// What the form asks [`App`](super::app::App) to do; everything else it
/// handles itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormAction {
    /// Close the form and run the report under these options.
    Apply(QueryOptions),
    /// Close the form, changing nothing.
    Cancel,
}

/// State of the `.` form: the rows, which one has the focus, and the complaint
/// from the last refused [`FormMessage::Submit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsForm {
    /// The options the form was opened on. Kept whole so applying carries the
    /// fields the form does not draw (`--today`) through unchanged.
    base: QueryOptions,
    fields: Vec<Field>,
    focus: usize,
    /// Why the last submit did not go through, shown under the rows until the
    /// next edit. The form stays open so it can be fixed where it was typed.
    error: Option<String>,
}

impl OptionsForm {
    /// Opens the form on the options currently in effect.
    pub(super) fn new(options: &QueryOptions) -> Self {
        let fields = FieldId::ALL
            .iter()
            .map(|&id| {
                let value = match id {
                    FieldId::Exchange => Value::Text(options.exchange.clone().unwrap_or_default()),
                    FieldId::Historical => Value::Flag(options.historical),
                    FieldId::Start => Value::Text(format_date(options.start)),
                    FieldId::End => Value::Text(format_date(options.end)),
                    FieldId::PriceDb => Value::Text(
                        options
                            .price_db
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                    ),
                };
                Field { id, value }
            })
            .collect();
        Self {
            base: options.clone(),
            fields,
            focus: 0,
            error: None,
        }
    }

    pub(super) fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub(super) fn focus(&self) -> usize {
        self.focus
    }

    pub(super) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn focused(&mut self) -> &mut Field {
        &mut self.fields[self.focus]
    }

    /// Applies a [`FormMessage`], returning a [`FormAction`] for the steps the
    /// form cannot take itself (running the query, closing itself).
    pub(super) fn update(&mut self, msg: FormMessage) -> Option<FormAction> {
        let len = self.fields.len();
        match msg {
            FormMessage::FocusNext => self.focus = (self.focus + 1) % len,
            FormMessage::FocusPrev => self.focus = (self.focus + len - 1) % len,
            FormMessage::Push(c) => {
                if let Value::Text(text) = &mut self.focused().value {
                    text.push(c);
                    self.error = None;
                }
            }
            FormMessage::Pop => {
                if let Value::Text(text) = &mut self.focused().value {
                    text.pop();
                    self.error = None;
                }
            }
            FormMessage::Clear => {
                // Emptying a flag is turning it off: that is its unset state.
                self.focused().value = match self.focused().value {
                    Value::Text(_) => Value::Text(String::new()),
                    Value::Flag(_) => Value::Flag(false),
                };
                self.error = None;
            }
            FormMessage::Toggle => {
                if let Value::Flag(on) = &mut self.focused().value {
                    *on = !*on;
                    self.error = None;
                }
            }
            FormMessage::Submit => match self.to_options() {
                Ok(options) => return Some(FormAction::Apply(options)),
                // Nothing is applied and nothing is lost: the offending row is
                // still there to fix, with the reason under it.
                Err(err) => self.error = Some(err),
            },
            FormMessage::Cancel => return Some(FormAction::Cancel),
        }
        None
    }

    /// Translates a key event into a [`FormMessage`]. Every printable key is
    /// text for the focused row (a flag row takes `space` as its toggle), so
    /// only the keys below reach anything else — `Ctrl-C` included, which
    /// [`super::event::key_to_message`] answers before the form is consulted.
    pub(super) fn key_to_message(&self, key: KeyEvent) -> Option<FormMessage> {
        let ctrl = is_ctrl(key.modifiers);
        match key.code {
            KeyCode::Esc => Some(FormMessage::Cancel),
            KeyCode::Enter => Some(FormMessage::Submit),
            KeyCode::Tab | KeyCode::Down => Some(FormMessage::FocusNext),
            KeyCode::Char('n') if ctrl => Some(FormMessage::FocusNext),
            KeyCode::BackTab | KeyCode::Up => Some(FormMessage::FocusPrev),
            KeyCode::Char('p') if ctrl => Some(FormMessage::FocusPrev),
            KeyCode::Backspace => Some(FormMessage::Pop),
            KeyCode::Char('u') if ctrl => Some(FormMessage::Clear),
            KeyCode::Char(' ') if !self.fields[self.focus].is_text() => Some(FormMessage::Toggle),
            KeyCode::Char(c) if !ctrl => Some(FormMessage::Push(c)),
            _ => None,
        }
    }

    /// Reads every row back into options, or reports the first row that does
    /// not parse. Rows the form does not draw are carried over from the options
    /// it was opened on.
    fn to_options(&self) -> Result<QueryOptions, String> {
        let mut options = self.base.clone();
        for field in &self.fields {
            match field.id {
                FieldId::Exchange => options.exchange = field.stated().map(str::to_owned),
                FieldId::Historical => options.historical = field.flag(),
                FieldId::Start => options.start = self.date_of(field)?,
                FieldId::End => options.end = self.date_of(field)?,
                FieldId::PriceDb => options.price_db = field.stated().map(Into::into),
            }
        }
        Ok(options)
    }

    fn date_of(&self, field: &Field) -> Result<Option<chrono::NaiveDate>, String> {
        field
            .stated()
            .map(|text| parse_date(text).map_err(|err| format!("{}: {err}", field.label())))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use crossterm::event::KeyModifiers;

    use super::super::testing::options;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Types `text` into the focused row.
    fn type_text(form: &mut OptionsForm, text: &str) {
        for c in text.chars() {
            form.update(FormMessage::Push(c));
        }
    }

    /// Moves the focus onto `id`.
    fn focus(form: &mut OptionsForm, id: FieldId) {
        while form.fields[form.focus].id != id {
            form.update(FormMessage::FocusNext);
        }
    }

    fn submit(form: &mut OptionsForm) -> Option<FormAction> {
        form.update(FormMessage::Submit)
    }

    #[test]
    fn opens_on_the_current_options() {
        let mut opts = options();
        opts.exchange = Some("CHF".to_owned());
        opts.historical = true;
        opts.start = Some(date(2024, 1, 1));
        opts.price_db = Some(PathBuf::from("prices.db"));
        let form = OptionsForm::new(&opts);
        let shown: Vec<&str> = form.fields().iter().map(Field::text).collect();
        assert_eq!(shown, ["CHF", "on", "2024-01-01", "", "prices.db"]);
    }

    /// Submitting an untouched form yields exactly what it was opened on —
    /// including `--today`, which it never draws.
    #[test]
    fn submitting_unchanged_is_the_identity() {
        let mut opts = options();
        opts.exchange = Some("CHF".to_owned());
        opts.end = Some(date(2025, 1, 1));
        let mut form = OptionsForm::new(&opts);
        assert_eq!(submit(&mut form), Some(FormAction::Apply(opts)));
    }

    #[test]
    fn typing_and_toggling_reach_the_applied_options() {
        let mut form = OptionsForm::new(&options());
        focus(&mut form, FieldId::Exchange);
        type_text(&mut form, "CHF");
        focus(&mut form, FieldId::Historical);
        form.update(FormMessage::Toggle);
        focus(&mut form, FieldId::Start);
        type_text(&mut form, "2024-01-01");
        focus(&mut form, FieldId::PriceDb);
        type_text(&mut form, "prices.db");

        assert_matches!(submit(&mut form), Some(FormAction::Apply(opts)) => {
            assert_eq!(opts.exchange.as_deref(), Some("CHF"));
            assert!(opts.historical);
            assert_eq!(opts.start, Some(date(2024, 1, 1)));
            assert_eq!(opts.end, None);
            assert_eq!(opts.price_db, Some(PathBuf::from("prices.db")));
            // Untouched, and untouchable from the form.
            assert_eq!(opts.today, date(2024, 6, 1));
        });
    }

    /// Emptying a row unsets the option, which is how a conversion or a range
    /// is taken back off.
    #[test]
    fn clearing_a_row_unsets_the_option() {
        let mut opts = options();
        opts.exchange = Some("CHF".to_owned());
        opts.historical = true;
        opts.start = Some(date(2024, 1, 1));
        let mut form = OptionsForm::new(&opts);

        focus(&mut form, FieldId::Exchange);
        form.update(FormMessage::Clear);
        focus(&mut form, FieldId::Historical);
        form.update(FormMessage::Clear);
        focus(&mut form, FieldId::Start);
        for _ in 0.."2024-01-01".len() {
            form.update(FormMessage::Pop);
        }

        assert_matches!(submit(&mut form), Some(FormAction::Apply(opts)) => {
            assert_eq!(opts.exchange, None);
            assert!(!opts.historical);
            assert_eq!(opts.start, None);
        });
    }

    #[test]
    fn whitespace_only_input_is_unset() {
        let mut form = OptionsForm::new(&options());
        focus(&mut form, FieldId::Exchange);
        type_text(&mut form, "  ");
        assert_matches!(submit(&mut form), Some(FormAction::Apply(opts)) => {
            assert_eq!(opts.exchange, None);
        });
    }

    /// A date that does not parse keeps the form open, says which row is wrong,
    /// and applies nothing.
    #[test]
    fn a_bad_date_is_refused_in_place() {
        let mut form = OptionsForm::new(&options());
        focus(&mut form, FieldId::End);
        type_text(&mut form, "2024/13/01");
        assert_eq!(submit(&mut form), None);
        let err = form.error().expect("the refused submit should say why");
        assert!(err.contains("--end"), "{err}");
        assert!(err.contains("YYYY-MM-DD"), "{err}");

        // The typed text is still there to be fixed, and the next edit takes
        // the complaint down.
        assert_eq!(form.fields()[form.focus()].text(), "2024/13/01");
        form.update(FormMessage::Pop);
        assert_eq!(form.error(), None);
    }

    #[test]
    fn focus_wraps_in_both_directions() {
        let mut form = OptionsForm::new(&options());
        assert_eq!(form.focus(), 0);
        form.update(FormMessage::FocusPrev);
        assert_eq!(form.focus(), FieldId::ALL.len() - 1);
        form.update(FormMessage::FocusNext);
        assert_eq!(form.focus(), 0);
    }

    #[test]
    fn cancel_asks_to_close_without_applying() {
        let mut form = OptionsForm::new(&options());
        focus(&mut form, FieldId::Exchange);
        type_text(&mut form, "CHF");
        assert_eq!(form.update(FormMessage::Cancel), Some(FormAction::Cancel));
    }

    /// The rows are typed into with no editing mode: `j`, `k` and `q` are text
    /// here, not the motions and the quit they are on the screens below.
    #[test]
    fn printable_keys_are_text_on_a_text_row() {
        let form = OptionsForm::new(&options());
        for c in ['j', 'k', 'q', 'r', '.', '/', '-', '5', ' '] {
            assert_eq!(
                form.key_to_message(key(KeyCode::Char(c))),
                Some(FormMessage::Push(c)),
                "{c:?}"
            );
        }
    }

    /// …except on the flag row, where `space` is the only way to change it.
    #[test]
    fn space_toggles_the_flag_row() {
        let mut form = OptionsForm::new(&options());
        focus(&mut form, FieldId::Historical);
        assert_eq!(
            form.key_to_message(key(KeyCode::Char(' '))),
            Some(FormMessage::Toggle)
        );
        // Text keys on a flag row are simply ignored rather than mapped to
        // something surprising.
        assert_eq!(
            form.key_to_message(key(KeyCode::Char('x'))),
            Some(FormMessage::Push('x'))
        );
        form.update(FormMessage::Push('x'));
        assert_eq!(form.fields()[form.focus()].text(), "off");
    }

    #[test]
    fn navigation_and_control_keys_map() {
        let form = OptionsForm::new(&options());
        for (k, msg) in [
            (key(KeyCode::Tab), FormMessage::FocusNext),
            (key(KeyCode::Down), FormMessage::FocusNext),
            (ctrl_key('n'), FormMessage::FocusNext),
            (key(KeyCode::BackTab), FormMessage::FocusPrev),
            (key(KeyCode::Up), FormMessage::FocusPrev),
            (ctrl_key('p'), FormMessage::FocusPrev),
            (key(KeyCode::Backspace), FormMessage::Pop),
            (ctrl_key('u'), FormMessage::Clear),
            (key(KeyCode::Enter), FormMessage::Submit),
            (key(KeyCode::Esc), FormMessage::Cancel),
        ] {
            assert_eq!(form.key_to_message(k), Some(msg), "{k:?}");
        }
    }

    #[test]
    fn unmapped_keys_are_ignored() {
        let form = OptionsForm::new(&options());
        assert_eq!(form.key_to_message(key(KeyCode::F(5))), None);
        assert_eq!(form.key_to_message(ctrl_key('z')), None);
    }
}
