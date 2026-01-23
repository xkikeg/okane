//! Defines a simple line-based reader.

use std::io::BufRead;

/// Peekable reader to read input line by line.
/// Actually this doesn't read file line-by-line,
/// but rather read everything into `Vec` for simplicity.
/// Expected input is quite small and it should be ok.
#[derive(Debug)]
pub struct LineReader {
    lines: Vec<String>,
    current: usize,
}

/// str with line information.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Line<'a> {
    /// actual line string.
    pub value: &'a str,
    /// 1-based line count.
    pub line_count: usize,
}

impl LineReader {
    /// Creates a new instance.
    pub fn new<T: BufRead>(r: T) -> std::io::Result<LineReader> {
        let lines: Result<Vec<String>, _> = r.lines().collect();
        Ok(LineReader {
            lines: lines?,
            current: 0,
        })
    }

    /// Returns last line count, useful for missing line error.
    pub fn last_line_count(&self) -> usize {
        self.current
    }

    /// Returns the next line without moving the current position.
    pub fn peek_line(&self) -> Option<Line<'_>> {
        self.nth(self.current)
    }

    /// Returns the next line, and moves the current position.
    pub fn read_line(&mut self) -> Option<Line<'_>> {
        let i = self.current;
        self.current += 1;
        self.nth(i)
    }

    fn nth(&self, n: usize) -> Option<Line<'_>> {
        self.lines.get(n).map(|value| Line {
            value,
            line_count: n + 1,
        })
    }
}

#[cfg(test)]
pub use testing::ErrRead;

#[cfg(test)]
mod testing {
    use std::io;

    pub struct ErrRead<'a> {
        initial: &'a [u8],
        error: io::Error,
    }

    impl<'a> ErrRead<'a> {
        pub fn new(initial: &'a [u8], error: io::Error) -> Self {
            Self { initial, error }
        }
    }

    impl io::Read for ErrRead<'_> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let read = self.initial.read(buf)?;
            if read != 0 {
                return Ok(read);
            }
            let mut err = io::Error::new(
                io::ErrorKind::Unsupported,
                "called read more than once on error case",
            );
            std::mem::swap(&mut err, &mut self.error);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn line_reader_fails_to_new() {
        let r = ErrRead::new(
            b"first line\n",
            io::Error::new(io::ErrorKind::PermissionDenied, "expectde failure to read"),
        );
        let got_err = LineReader::new(io::BufReader::new(r)).unwrap_err();

        assert_eq!(io::ErrorKind::PermissionDenied, got_err.kind());
    }

    #[test]
    fn line_reader_peek_line_and_read_line() {
        let input = indoc! {"
            First line
            Second line
            Third line
        "};
        let mut r = LineReader::new(input.as_bytes()).unwrap();

        // first peek reads the next line.
        assert_eq!(
            Some(Line {
                value: "First line",
                line_count: 1
            }),
            r.peek_line()
        );
        // second peek just returns the buffered value.
        assert_eq!(
            Some(Line {
                value: "First line",
                line_count: 1
            }),
            r.peek_line()
        );
        assert_eq!(
            Some(Line {
                value: "First line",
                line_count: 1,
            }),
            r.read_line()
        );
        assert_eq!(
            Some(Line {
                value: "Second line",
                line_count: 2,
            }),
            r.read_line()
        );
        assert_eq!(
            Some(Line {
                value: "Third line",
                line_count: 3
            }),
            r.peek_line()
        );
        assert_eq!(
            Some(Line {
                value: "Third line",
                line_count: 3,
            }),
            r.read_line()
        );
        assert_eq!(None, r.peek_line());
        assert_eq!(None, r.read_line());
    }
}
