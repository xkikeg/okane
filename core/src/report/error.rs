//! Defines error in report functions.

use std::{fmt::Display, path::PathBuf};

use annotate_snippets::{Level, Snippet};

use crate::{load, parse};

use super::book_keeping;

/// Error arised in report APIs.
#[derive(thiserror::Error, Debug)]
pub enum ReportError {
    Load(#[from] load::LoadError),
    BookKeep(book_keeping::BookKeepError, ErrorContext),
}

impl Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::Load(_) => write!(f, "failed to load the given file"),
            ReportError::BookKeep(err, ctx) => ctx.print(f, err),
        }
    }
}

/// Context of [ReportError], to carry the failure information.
#[derive(Debug)]
pub struct ErrorContext {
    renderer: annotate_snippets::Renderer,
    path: PathBuf,
    line_start: usize,
    text: String,
}

impl ErrorContext {
    fn print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        err: &book_keeping::BookKeepError,
    ) -> std::fmt::Result {
        let message = err.to_string();
        let path = self.path.to_string_lossy();
        let message = Level::Error.title(&message).snippet(
            Snippet::source(&self.text)
                .origin(&path)
                .line_start(self.line_start)
                .fold(true)
                .annotation(Level::Error.span(0..self.text.len())),
        );
        let rendered = self.renderer.render(message);
        rendered.fmt(f)
    }

    pub(super) fn new(
        renderer: annotate_snippets::Renderer,
        path: PathBuf,
        pctx: &parse::ParsedContext,
    ) -> Self {
        Self {
            renderer,
            path,
            line_start: pctx.compute_line_start(),
            text: pctx.as_str().to_owned(),
        }
    }
}
