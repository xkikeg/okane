use std::{
    fmt::Display,
    ops::{Deref, Range},
};

use annotate_snippets::{Level, Renderer, Snippet};
use winnow::{
    error::{ContextError, ErrMode, StrContext},
    stream::{Location, Offset, Stream},
    LocatingSlice,
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
        mut input: LocatingSlice<&'i str>,
        start: <LocatingSlice<&'i str> as Stream>::Checkpoint,
        error: ErrMode<ContextError<StrContext>>,
    ) -> Self {
        let offset = input.offset_from(&start);
        input.reset(&start);
        let line_start = compute_line_number(initial, input.location());
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

/// Computes the line number at the `pos` position of `s`.
/// If `pos` is outside of `s` or not a UTF-8 boundary, it panics.
pub(super) fn compute_line_number(s: &str, pos: usize) -> usize {
    assert!(
        pos <= s.len(),
        "cannot compute line_number for out-of-range position"
    );
    let (s, _) = s.as_bytes().split_at(pos);
    1 + s.iter().filter(|x| **x == b'\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_line_number_valid_inputs() {
        assert_eq!(compute_line_number("This\nis\npen", 0), 1);
        assert_eq!(compute_line_number("This\nis\npen", 1), 1);
        assert_eq!(compute_line_number("This\nis\npen", 4), 1);
        assert_eq!(compute_line_number("This\nis\npen", 5), 2);
        assert_eq!(compute_line_number("This\nis\npen", 7), 2);
        assert_eq!(compute_line_number("This\nis\npen", 8), 3);
    }

    #[test]
    fn compute_line_number_works_on_invalid_utf8_boundary() {
        assert_eq!(compute_line_number("日本語だよ", 1), 1);
    }

    #[test]
    #[should_panic(expected = "cannot compute line_number for")]
    fn compute_line_number_panics_on_out_of_range_pos() {
        compute_line_number("hello world", 12);
    }
}
