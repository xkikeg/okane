use std::{
    fmt::Display,
    ops::{Deref, Range},
};

use annotate_snippets::{Level, Renderer, Snippet};
use winnow::{
    error::{ContextError, ErrMode, StrContext},
    stream::{Offset, Stream},
    Located,
};

#[derive(Debug)]
pub struct ParseError {
    renderer: Renderer,
    error_span: Range<usize>,
    input: String,
    line_start: usize,
    winnow_error: ContextError,
}

impl ParseError {
    /// Create a new instance of ParseError.
    pub(super) fn new<'i>(
        renderer: Renderer,
        initial: &'i str,
        mut input: Located<&'i str>,
        start: <Located<&'i str> as Stream>::Checkpoint,
        error: ErrMode<ContextError<StrContext>>,
    ) -> Self {
        let offset = input.offset_from(&start);
        input.reset(&start);
        let line_start = compute_line_number(initial, input.as_ref());
        let error = error.into_inner().expect("partial input can't be used");
        // Assume the error span is only for the first `char`.
        // When we'll implement
        let end = (offset + 1..)
            .find(|e| input.is_char_boundary(*e))
            .unwrap_or(offset);
        Self {
            renderer,
            error_span: offset..end,
            input: input.deref().to_string(),
            line_start,
            winnow_error: error,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.winnow_error.to_string();
        let message = Level::Error.title(&message).snippet(
            Snippet::source(&self.input)
                .line_start(self.line_start)
                .fold(true)
                .annotation(Level::Error.span(self.error_span.clone())),
        );
        let rendered = self.renderer.render(message);
        rendered.fmt(f)
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.winnow_error
            .cause()
            .map(|x| x as &(dyn std::error::Error + 'static))
    }
}

fn compute_line_number(initial: &str, current: &str) -> usize {
    let current = current.as_ptr() as usize;
    1 + initial
        .lines()
        .take_while(|x| {
            let linehead = x.as_ptr();
            (linehead as usize) < current
        })
        .count()
}
