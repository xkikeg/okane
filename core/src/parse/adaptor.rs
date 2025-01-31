//! Provides utilities to use winnow parser easily with [`ParseError`].

use std::{fmt::Debug, marker::PhantomData, ops::Range};

use winnow::{error::ContextError, LocatingSlice, Parser};

use crate::syntax;

use super::error::{self, ParseError};

/// Options to control parse behavior.
#[derive(Debug, Clone)]
pub struct ParseOptions {
    error_style: annotate_snippets::Renderer,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            error_style: annotate_snippets::Renderer::plain(),
        }
    }
}

impl ParseOptions {
    /// Sets the given [`annotate_snippets::Renderer`] to `self`.
    pub fn with_error_style(mut self, error_style: annotate_snippets::Renderer) -> Self {
        self.error_style = error_style;
        self
    }

    pub(super) fn parse_single<'i, Out, P>(
        &self,
        parser: P,
        input: &'i str,
    ) -> Result<(ParsedContext<'i>, Out), ParseError>
    where
        Out: 'i,
        P: Parser<LocatingSlice<&'i str>, Out, ContextError> + 'i,
    {
        use winnow::stream::Stream as _;
        let initial = input;
        let input = LocatingSlice::new(input);
        let start = input.checkpoint();
        match parser.with_span().parse(input) {
            Err(e) => Err(ParseError::new(
                self.error_style.clone(),
                initial,
                input,
                start,
                e.into_inner(),
            )),
            Ok((entry, span)) => Ok((ParsedContext { initial, span }, entry)),
        }
    }

    /// Parses the given `parser` object, separated with `separator`.
    pub(super) fn parse_repeated<'i, Out, Sep, P, Q, E>(
        &self,
        parser: P,
        separator: Q,
        input: &'i str,
    ) -> impl Iterator<Item = Result<(ParsedContext<'i>, Out), ParseError>> + 'i
    where
        Out: 'i,
        Sep: 'i,
        P: Parser<LocatingSlice<&'i str>, Out, E> + 'i,
        Q: Parser<LocatingSlice<&'i str>, Sep, E> + 'i,
        E: winnow::error::ParserError<&'i str, Inner = ContextError> + Debug + 'i,
    {
        ParsedIter {
            parser,
            separator,
            initial: input,
            input: LocatingSlice::new(input),
            renderer: self.error_style.clone(),
            _phantom: PhantomData,
        }
    }
}

/// Context information carrying the metadata of the entry.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedContext<'i> {
    pub(super) initial: &'i str,
    pub(super) span: Range<usize>,
}

impl ParsedContext<'_> {
    /// Computes the starting line number from this context.
    /// Note this function is O(N), not a cheap function.
    pub fn compute_line_start(&self) -> usize {
        error::compute_line_number(self.initial, self.span.start)
    }

    /// Returns the [`str`] slice corresponding to this context.
    pub fn as_str(&self) -> &str {
        self.initial
            .get(self.span.clone())
            .expect("ParsedContext::span must be a valid UTF-8 boundary")
    }

    /// Returns the position of the parsed string within the original `&str`,
    /// which can be used to find the position of the [`Tracked`][syntax::tracked::Tracked] item.
    pub fn span(&self) -> ParsedSpan {
        ParsedSpan(self.span.clone())
    }
}

/// Range parsed with the given parser within the original input `&str`.
#[derive(Debug)]
pub struct ParsedSpan(Range<usize>);

impl ParsedSpan {
    /// Returns the span of the given span relative to this span.
    pub fn resolve(&self, span: &syntax::tracked::TrackedSpan) -> Range<usize> {
        let target = span.as_range();
        clip(self.0.clone(), target)
    }
}

fn clip(parent: Range<usize>, child: Range<usize>) -> Range<usize> {
    let start = std::cmp::max(parent.start, child.start) - parent.start;
    let end = std::cmp::min(parent.end, child.end) - parent.start;
    start..end
}

/// Iterator to return parsed ledger entry one-by-one.
struct ParsedIter<'i, Out, Sep, P, Q, E> {
    parser: P,
    separator: Q,
    initial: &'i str,
    input: LocatingSlice<&'i str>,
    renderer: annotate_snippets::Renderer,
    _phantom: PhantomData<(Out, Sep, E)>,
}

impl<'i, Out, Sep, P, Q, E> Iterator for ParsedIter<'i, Out, Sep, P, Q, E>
where
    P: Parser<LocatingSlice<&'i str>, Out, E>,
    Q: Parser<LocatingSlice<&'i str>, Sep, E>,
    E: winnow::error::ParserError<&'i str, Inner = ContextError> + Debug + 'i,
{
    type Item = Result<(ParsedContext<'i>, Out), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        use winnow::stream::Stream as _;
        let start = self.input.checkpoint();
        self.next_impl()
            .map_err(|e| {
                ParseError::new(
                    self.renderer.clone(),
                    self.initial,
                    self.input,
                    start,
                    e.into_inner()
                        .expect("ParseIter doesn't work with streaming parse yet"),
                )
            })
            .transpose()
    }
}

impl<'i, Out, Sep, P, Q, E> ParsedIter<'i, Out, Sep, P, Q, E>
where
    P: Parser<LocatingSlice<&'i str>, Out, E>,
    Q: Parser<LocatingSlice<&'i str>, Sep, E>,
    E: winnow::error::ParserError<&'i str> + 'i,
{
    fn next_impl(&mut self) -> Result<Option<(ParsedContext<'i>, Out)>, E> {
        self.separator.parse_next(&mut self.input)?;
        if self.input.is_empty() {
            return Ok(None);
        }
        let (entry, span) = self
            .parser
            .by_ref()
            .with_span()
            .parse_next(&mut self.input)?;
        Ok(Some((
            ParsedContext {
                initial: self.initial,
                span,
            },
            entry,
        )))
    }
}
