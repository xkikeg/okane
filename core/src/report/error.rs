//! Defines error in report functions.

use std::{fmt::Display, path::PathBuf};

use annotate_snippets::{Annotation, Level, Snippet};
use bumpalo::Bump;

use crate::{
    load,
    parse::{self, ParsedSpan},
};

use super::{
    book_keeping::{self, BookKeepError},
    price_db,
};

/// Error arised in report APIs.
#[derive(thiserror::Error, Debug)]
pub enum ReportError {
    Load(#[from] load::LoadError),
    PriceDB(#[from] price_db::LoadError),
    BookKeep(book_keeping::BookKeepError, Box<ErrorContext>),
}

impl Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::Load(_) => write!(f, "failed to load the given file"),
            ReportError::PriceDB(_) => write!(f, "failed to load the Price DB"),
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
    parsed_span: ParsedSpan,
}

impl ErrorContext {
    fn print(&self, f: &mut std::fmt::Formatter<'_>, err: &BookKeepError) -> std::fmt::Result {
        let message = err.to_string();
        let path = self.path.to_string_lossy();
        let bump = Bump::new();
        let annotations: Vec<Annotation> = err.annotations(&bump, &self.parsed_span, &self.text);
        let message = Level::ERROR.primary_title(&message).element(
            Snippet::source(&self.text)
                .path(&path)
                .line_start(self.line_start)
                .fold(false)
                .annotations(annotations),
        );
        let rendered = self.renderer.render(&[message]);
        rendered.fmt(f)
    }

    pub(super) fn new(
        renderer: annotate_snippets::Renderer,
        path: PathBuf,
        pctx: &parse::ParsedContext,
    ) -> Box<Self> {
        Box::new(Self {
            renderer,
            path,
            line_start: pctx.compute_line_start(),
            text: pctx.as_str().to_owned(),
            parsed_span: pctx.span(),
        })
    }
}
