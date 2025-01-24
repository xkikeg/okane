//! Defines error in report functions.

use std::{fmt::Display, path::PathBuf};

use annotate_snippets::{Annotation, Level, Snippet};

use crate::{
    load,
    parse::{self, ParsedSpan},
};

use super::book_keeping::{self, BookKeepError};

/// Error arised in report APIs.
#[derive(thiserror::Error, Debug)]
pub enum ReportError {
    Load(#[from] load::LoadError),
    BookKeep(book_keeping::BookKeepError, Box<ErrorContext>),
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
    parsed_span: ParsedSpan,
}

impl ErrorContext {
    fn print(&self, f: &mut std::fmt::Formatter<'_>, err: &BookKeepError) -> std::fmt::Result {
        let message = err.to_string();
        let path = self.path.to_string_lossy();
        let annotations: Vec<Annotation> = match err {
            BookKeepError::UndeduciblePostingAmount(first, second) => vec![
                Level::Warning
                    .span(self.parsed_span.resolve(&first.span()))
                    .label("first posting without constraints"),
                Level::Error
                    .span(self.parsed_span.resolve(&second.span()))
                    .label("cannot deduce this posting"),
            ],
            BookKeepError::ZeroAmountWithExchange(exchange) => vec![Level::Error
                .span(self.parsed_span.resolve(exchange))
                .label("absolute zero posting should not have exchange")],
            BookKeepError::ZeroExchangeRate(exchange) => vec![Level::Error
                .span(self.parsed_span.resolve(exchange))
                .label("exchange with zero amount")],
            BookKeepError::ExchangeWithAmountCommodity {
                posting_amount,
                exchange,
            } => vec![
                Level::Info
                    .span(self.parsed_span.resolve(posting_amount))
                    .label("posting amount"),
                Level::Error
                    .span(self.parsed_span.resolve(exchange))
                    .label("exchange cannot have the same commodity with posting"),
            ],
            _ => {
                // TODO: Add more detailed error into this.
                // Also, put these logic into BookKeepError.
                vec![Level::Error.span(0..self.text.len()).label("error occured")]
            }
        };
        let message = Level::Error.title(&message).snippet(
            Snippet::source(&self.text)
                .origin(&path)
                .line_start(self.line_start)
                .annotations(annotations),
        );
        let rendered = self.renderer.render(message);
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
