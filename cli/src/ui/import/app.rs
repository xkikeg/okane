//! Application state for the import review session.
//!
//! Follows The Elm Architecture: state lives in [`ReviewApp`], all
//! transitions go through [`ReviewApp::update`] driven by a [`Message`].
//! Decisions that must mutate the imported transactions are returned as
//! [`Command`]s and fulfilled by [`super::event`], which reports back via
//! [`ReviewApp::apply_decision`].

use chrono::NaiveDate;

use crate::import::single_entry::ReviewKind;
use crate::ui::table::TableNav;

/// Decision state of one reviewed transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Not decided yet.
    Todo,
    /// Confirmed (possibly after an account assignment).
    Accepted,
    /// Deliberately left as-is; still written on session completion.
    Skipped,
}

/// One row of the review queue, mirroring the imported transaction list by index.
#[derive(Debug)]
pub struct ReviewItem {
    pub kind: ReviewKind,
    pub status: Status,
    /// Rendered Ledger text of the (possibly edited) transaction.
    pub preview: String,
    pub date: NaiveDate,
    pub payee: String,
    pub amount: String,
}

impl ReviewItem {
    pub fn new(
        kind: ReviewKind,
        preview: String,
        date: NaiveDate,
        payee: String,
        amount: String,
    ) -> Self {
        // Rule-matched transactions need no attention.
        let status = match kind {
            ReviewKind::Auto => Status::Accepted,
            ReviewKind::Pending | ReviewKind::Unknown => Status::Todo,
        };
        Self {
            kind,
            status,
            preview,
            date,
            payee,
            amount,
        }
    }
}

/// How the session ended; tells the caller whether to write the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionOutcome {
    /// Append all transactions to the output file.
    Write,
    /// Discard everything; the output file is untouched.
    Abort,
}

/// Modal overlay drawn on top of the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// Confirm writing while `unreviewed` items are still [`Status::Todo`].
    WriteConfirm { unreviewed: usize },
    /// Confirm aborting the session without writing.
    AbortConfirm,
}

/// Account-input prompt with autocomplete over the ledger's accounts.
#[derive(Debug)]
pub struct AccountPrompt {
    /// Index of the item the chosen account applies to.
    pub item: usize,
    /// Raw typed input.
    pub input: String,
    /// Indices into `ReviewApp::accounts` matching `input`, ascending.
    pub matches: Vec<usize>,
    /// Cursor into `matches`; meaningful only when `matches` is non-empty.
    pub selected: usize,
}

impl AccountPrompt {
    fn new(item: usize, accounts: &[String]) -> Self {
        let mut prompt = Self {
            item,
            input: String::new(),
            matches: Vec::new(),
            selected: 0,
        };
        prompt.refilter(accounts);
        prompt
    }

    fn refilter(&mut self, accounts: &[String]) {
        self.matches = filter_accounts(accounts, &self.input);
        self.selected = 0;
    }

    fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if !self.matches.is_empty() {
            self.selected = std::cmp::min(self.selected + 1, self.matches.len() - 1);
        }
    }
}

/// Case-insensitive substring filter; empty input matches everything.
fn filter_accounts(accounts: &[String], input: &str) -> Vec<usize> {
    let needle = input.to_lowercase();
    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| account.to_lowercase().contains(&needle))
        .map(|(i, _)| i)
        .collect()
}

/// Messages that drive state transitions (Elm-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    SelectFirst,
    SelectLast,
    /// Accept the selected item as-is (clears the pending flag).
    Accept,
    /// Open the account prompt for the selected item.
    OpenPrompt,
    /// Leave the selected item untouched and move on.
    Skip,
    /// Finish the session and write — confirmed first if items remain Todo.
    RequestWrite,
    /// Abort the session — always confirmed.
    RequestAbort,
    /// Confirm the current overlay.
    ConfirmOverlay,
    /// Dismiss the current overlay.
    DismissOverlay,
    /// Unconditional abort (Ctrl-C).
    AbortImmediate,
    /// A printable character typed into the account prompt.
    PromptInput(char),
    PromptBackspace,
    /// Move the candidate cursor up.
    PromptPrev,
    /// Move the candidate cursor down.
    PromptNext,
    /// Complete the input to the highlighted candidate (Tab).
    PromptComplete,
    /// Submit the highlighted candidate, or the raw input when nothing
    /// matches (new accounts are allowed).
    PromptSubmit,
    PromptCancel,
}

/// Effect requested by [`ReviewApp::update`] that requires the mutable
/// transaction list the pure state machine does not own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Clear the pending flag of the transaction at `index`.
    AcceptPending { index: usize },
    /// Set the destination account of the transaction at `index`.
    SetAccount { index: usize, account: String },
}

/// Application state for the review session.
#[derive(Debug)]
pub struct ReviewApp {
    pub source_display: String,
    pub output_display: String,
    pub items: Vec<ReviewItem>,
    pub nav: TableNav,
    /// Sorted account names from the user's ledger, for autocomplete.
    pub accounts: Vec<String>,
    pub prompt: Option<AccountPrompt>,
    pub overlay: Option<Overlay>,
    /// Set once the session is over; the event loop exits on it.
    pub outcome: Option<SessionOutcome>,
}

impl ReviewApp {
    pub fn new(
        source_display: String,
        output_display: String,
        items: Vec<ReviewItem>,
        accounts: Vec<String>,
    ) -> Self {
        let mut nav = TableNav::new(items.len());
        // Start on the first item needing attention.
        if let Some(first) = items.iter().position(|item| item.status == Status::Todo) {
            nav.table_state.select(Some(first));
        }
        Self {
            source_display,
            output_display,
            items,
            nav,
            accounts,
            prompt: None,
            overlay: None,
            outcome: None,
        }
    }

    /// Index of the currently-selected queue item, if any.
    pub fn selected_index(&self) -> Option<usize> {
        let idx = self.nav.table_state.selected()?;
        (idx < self.items.len()).then_some(idx)
    }

    /// Number of items already decided (not [`Status::Todo`]).
    pub fn reviewed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.status != Status::Todo)
            .count()
    }

    /// Applies a message; optionally returns a [`Command`] for the event
    /// loop to execute (the only impure step in this flow).
    pub fn update(&mut self, msg: Message) -> Option<Command> {
        // AbortImmediate is honored regardless of overlay/prompt.
        if matches!(msg, Message::AbortImmediate) {
            self.outcome = Some(SessionOutcome::Abort);
            return None;
        }

        if let Some(overlay) = self.overlay {
            match msg {
                Message::ConfirmOverlay => {
                    self.overlay = None;
                    self.outcome = Some(match overlay {
                        Overlay::WriteConfirm { .. } => SessionOutcome::Write,
                        Overlay::AbortConfirm => SessionOutcome::Abort,
                    });
                }
                Message::DismissOverlay => self.overlay = None,
                // Ignore other input while a modal is up.
                _ => {}
            }
            return None;
        }

        if self.prompt.is_some() {
            return self.update_prompt(msg);
        }

        match msg {
            Message::MoveUp => self.nav.move_selection(-1),
            Message::MoveDown => self.nav.move_selection(1),
            Message::PageUp => {
                let delta = -(self.nav.page_size() as isize);
                self.nav.move_selection(delta);
            }
            Message::PageDown => {
                let delta = self.nav.page_size() as isize;
                self.nav.move_selection(delta);
            }
            Message::SelectFirst => self.nav.select_first(),
            Message::SelectLast => self.nav.select_last(),
            Message::Accept => {
                if let Some(index) = self.selected_index() {
                    match self.items[index].kind {
                        ReviewKind::Pending => {
                            return Some(Command::AcceptPending { index });
                        }
                        // Re-accepting (e.g. after a skip) needs no mutation.
                        ReviewKind::Auto => {
                            self.items[index].status = Status::Accepted;
                            self.nav.move_selection(1);
                        }
                        // An unknown transaction needs an account first.
                        ReviewKind::Unknown => {}
                    }
                }
            }
            Message::OpenPrompt => {
                if let Some(index) = self.selected_index() {
                    self.prompt = Some(AccountPrompt::new(index, &self.accounts));
                }
            }
            Message::Skip => {
                if let Some(index) = self.selected_index() {
                    self.items[index].status = Status::Skipped;
                    self.nav.move_selection(1);
                }
            }
            Message::RequestWrite => {
                let unreviewed = self.items.len() - self.reviewed_count();
                if unreviewed > 0 {
                    self.overlay = Some(Overlay::WriteConfirm { unreviewed });
                } else {
                    self.outcome = Some(SessionOutcome::Write);
                }
            }
            Message::RequestAbort => self.overlay = Some(Overlay::AbortConfirm),
            // Handled above or prompt-only.
            Message::AbortImmediate
            | Message::ConfirmOverlay
            | Message::DismissOverlay
            | Message::PromptInput(_)
            | Message::PromptBackspace
            | Message::PromptPrev
            | Message::PromptNext
            | Message::PromptComplete
            | Message::PromptSubmit
            | Message::PromptCancel => {}
        }
        None
    }

    fn update_prompt(&mut self, msg: Message) -> Option<Command> {
        let prompt = self.prompt.as_mut().expect("prompt must be active");
        match msg {
            Message::PromptInput(c) => {
                prompt.input.push(c);
                prompt.refilter(&self.accounts);
            }
            Message::PromptBackspace => {
                prompt.input.pop();
                prompt.refilter(&self.accounts);
            }
            Message::PromptPrev => prompt.select_prev(),
            Message::PromptNext => prompt.select_next(),
            Message::PromptComplete => {
                if let Some(&i) = prompt.matches.get(prompt.selected) {
                    self.accounts[i].clone_into(&mut prompt.input);
                    prompt.refilter(&self.accounts);
                }
            }
            Message::PromptSubmit => {
                let account = prompt
                    .matches
                    .get(prompt.selected)
                    .map(|&i| self.accounts[i].clone())
                    .unwrap_or_else(|| prompt.input.trim().to_string());
                if account.is_empty() {
                    return None;
                }
                let index = prompt.item;
                self.prompt = None;
                return Some(Command::SetAccount { index, account });
            }
            Message::PromptCancel => self.prompt = None,
            // Queue-level messages are not available while typing.
            _ => {}
        }
        None
    }

    /// Called by the event loop once a [`Command`] has been fulfilled:
    /// records the re-rendered preview and advances to the next item.
    pub fn apply_decision(&mut self, index: usize, preview: String) {
        let item = &mut self.items[index];
        item.status = Status::Accepted;
        item.preview = preview;
        if self.nav.table_state.selected() == Some(index) {
            self.nav.move_selection(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn item(kind: ReviewKind) -> ReviewItem {
        ReviewItem::new(
            kind,
            "preview\n".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            "payee".to_string(),
            "10 USD".to_string(),
        )
    }

    fn app(kinds: &[ReviewKind]) -> ReviewApp {
        ReviewApp::new(
            "source.csv".to_string(),
            "out.ledger".to_string(),
            kinds.iter().copied().map(item).collect(),
            vec![
                "Assets:Bank".to_string(),
                "Expenses:Grocery".to_string(),
                "Expenses:Sweets".to_string(),
                "Income:Salary".to_string(),
            ],
        )
    }

    #[test]
    fn auto_items_start_accepted_others_todo() {
        let app = app(&[ReviewKind::Auto, ReviewKind::Pending, ReviewKind::Unknown]);
        assert_eq!(app.items[0].status, Status::Accepted);
        assert_eq!(app.items[1].status, Status::Todo);
        assert_eq!(app.items[2].status, Status::Todo);
    }

    #[test]
    fn initial_selection_is_first_todo_item() {
        let app = app(&[ReviewKind::Auto, ReviewKind::Auto, ReviewKind::Unknown]);
        assert_eq!(app.nav.table_state.selected(), Some(2));
    }

    #[test]
    fn initial_selection_defaults_to_first_when_all_accepted() {
        let app = app(&[ReviewKind::Auto, ReviewKind::Auto]);
        assert_eq!(app.nav.table_state.selected(), Some(0));
    }

    #[test]
    fn accept_pending_returns_command() {
        let mut app = app(&[ReviewKind::Pending]);
        assert_eq!(
            app.update(Message::Accept),
            Some(Command::AcceptPending { index: 0 })
        );
        // Status flips only once the command is fulfilled.
        assert_eq!(app.items[0].status, Status::Todo);
    }

    #[test]
    fn accept_unknown_is_noop() {
        let mut app = app(&[ReviewKind::Unknown]);
        assert_eq!(app.update(Message::Accept), None);
        assert_eq!(app.items[0].status, Status::Todo);
    }

    #[test]
    fn accept_auto_re_accepts_after_skip() {
        let mut app = app(&[ReviewKind::Auto, ReviewKind::Auto]);
        app.update(Message::Skip);
        assert_eq!(app.items[0].status, Status::Skipped);
        app.update(Message::SelectFirst);
        app.update(Message::Accept);
        assert_eq!(app.items[0].status, Status::Accepted);
        // Acting advances to the next item.
        assert_eq!(app.nav.table_state.selected(), Some(1));
    }

    #[test]
    fn skip_marks_and_advances() {
        let mut app = app(&[ReviewKind::Pending, ReviewKind::Unknown]);
        app.update(Message::Skip);
        assert_eq!(app.items[0].status, Status::Skipped);
        assert_eq!(app.nav.table_state.selected(), Some(1));
    }

    #[test]
    fn apply_decision_accepts_and_advances() {
        let mut app = app(&[ReviewKind::Pending, ReviewKind::Unknown]);
        app.apply_decision(0, "new preview\n".to_string());
        assert_eq!(app.items[0].status, Status::Accepted);
        assert_eq!(app.items[0].preview, "new preview\n");
        assert_eq!(app.nav.table_state.selected(), Some(1));
    }

    #[test]
    fn apply_decision_elsewhere_keeps_selection() {
        let mut app = app(&[ReviewKind::Unknown, ReviewKind::Pending]);
        app.update(Message::SelectLast);
        app.apply_decision(0, "new preview\n".to_string());
        assert_eq!(app.nav.table_state.selected(), Some(1));
    }

    #[test]
    fn write_with_todo_items_asks_confirmation() {
        let mut app = app(&[ReviewKind::Pending, ReviewKind::Unknown, ReviewKind::Auto]);
        app.update(Message::RequestWrite);
        assert_eq!(app.overlay, Some(Overlay::WriteConfirm { unreviewed: 2 }));
        assert_eq!(app.outcome, None);

        app.update(Message::ConfirmOverlay);
        assert_eq!(app.overlay, None);
        assert_eq!(app.outcome, Some(SessionOutcome::Write));
    }

    #[test]
    fn write_with_everything_reviewed_finishes_directly() {
        let mut app = app(&[ReviewKind::Auto]);
        app.update(Message::RequestWrite);
        assert_eq!(app.overlay, None);
        assert_eq!(app.outcome, Some(SessionOutcome::Write));
    }

    #[test]
    fn dismissing_write_confirm_continues_session() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::RequestWrite);
        app.update(Message::DismissOverlay);
        assert_eq!(app.overlay, None);
        assert_eq!(app.outcome, None);
    }

    #[test]
    fn abort_is_always_confirmed() {
        let mut app = app(&[ReviewKind::Auto]);
        app.update(Message::RequestAbort);
        assert_eq!(app.overlay, Some(Overlay::AbortConfirm));
        app.update(Message::ConfirmOverlay);
        assert_eq!(app.outcome, Some(SessionOutcome::Abort));
    }

    #[test]
    fn abort_immediate_overrides_everything() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        assert!(app.prompt.is_some());
        app.update(Message::AbortImmediate);
        assert_eq!(app.outcome, Some(SessionOutcome::Abort));
    }

    #[test]
    fn nav_messages_ignored_while_overlay_visible() {
        let mut app = app(&[ReviewKind::Unknown, ReviewKind::Unknown]);
        app.update(Message::RequestAbort);
        app.update(Message::MoveDown);
        assert_eq!(app.nav.table_state.selected(), Some(0));
    }

    #[test]
    fn prompt_filters_candidates_incrementally() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        // Empty input matches everything.
        assert_eq!(app.prompt.as_ref().unwrap().matches, vec![0, 1, 2, 3]);

        for c in "exp".chars() {
            app.update(Message::PromptInput(c));
        }
        // Case-insensitive substring match.
        assert_eq!(app.prompt.as_ref().unwrap().matches, vec![1, 2]);

        app.update(Message::PromptBackspace);
        app.update(Message::PromptBackspace);
        app.update(Message::PromptBackspace);
        assert_eq!(app.prompt.as_ref().unwrap().matches, vec![0, 1, 2, 3]);
    }

    #[test]
    fn prompt_submit_picks_highlighted_candidate() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        for c in "exp".chars() {
            app.update(Message::PromptInput(c));
        }
        app.update(Message::PromptNext);
        let cmd = app.update(Message::PromptSubmit);
        assert_eq!(
            cmd,
            Some(Command::SetAccount {
                index: 0,
                account: "Expenses:Sweets".to_string(),
            })
        );
        assert!(app.prompt.is_none());
    }

    #[test]
    fn prompt_submit_uses_raw_input_when_nothing_matches() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        for c in "Expenses:Books".chars() {
            app.update(Message::PromptInput(c));
        }
        let cmd = app.update(Message::PromptSubmit);
        assert_eq!(
            cmd,
            Some(Command::SetAccount {
                index: 0,
                account: "Expenses:Books".to_string(),
            })
        );
    }

    #[test]
    fn prompt_submit_empty_is_noop() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.accounts.clear();
        app.update(Message::OpenPrompt);
        assert_eq!(app.update(Message::PromptSubmit), None);
        assert!(app.prompt.is_some());
    }

    #[test]
    fn prompt_tab_completes_to_highlighted_candidate() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        for c in "groc".chars() {
            app.update(Message::PromptInput(c));
        }
        app.update(Message::PromptComplete);
        let prompt = app.prompt.as_ref().unwrap();
        assert_eq!(prompt.input, "Expenses:Grocery");
        assert_eq!(prompt.matches, vec![1]);
    }

    #[test]
    fn prompt_cancel_keeps_item_untouched() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        app.update(Message::PromptInput('x'));
        app.update(Message::PromptCancel);
        assert!(app.prompt.is_none());
        assert_eq!(app.items[0].status, Status::Todo);
    }

    #[test]
    fn prompt_candidate_cursor_clamps() {
        let mut app = app(&[ReviewKind::Unknown]);
        app.update(Message::OpenPrompt);
        app.update(Message::PromptPrev);
        assert_eq!(app.prompt.as_ref().unwrap().selected, 0);
        for _ in 0..10 {
            app.update(Message::PromptNext);
        }
        assert_eq!(app.prompt.as_ref().unwrap().selected, 3);
    }

    #[test]
    fn filter_accounts_empty_input_matches_all() {
        let accounts = vec!["A".to_string(), "B".to_string()];
        assert_eq!(filter_accounts(&accounts, ""), vec![0, 1]);
    }

    #[test]
    fn filter_accounts_is_case_insensitive_substring() {
        let accounts = vec![
            "Expenses:Grocery".to_string(),
            "Income:Salary".to_string(),
            "Expenses:Salt".to_string(),
        ];
        assert_eq!(filter_accounts(&accounts, "sal"), vec![1, 2]);
        assert_eq!(filter_accounts(&accounts, "GROCERY"), vec![0]);
        assert_eq!(filter_accounts(&accounts, "nomatch"), Vec::<usize>::new());
    }

    #[test]
    fn reviewed_count_tracks_decisions() {
        let mut app = app(&[ReviewKind::Auto, ReviewKind::Pending, ReviewKind::Unknown]);
        assert_eq!(app.reviewed_count(), 1);
        app.update(Message::Skip);
        assert_eq!(app.reviewed_count(), 2);
    }
}
