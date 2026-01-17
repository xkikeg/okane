//! Defines a simple line-based reader.

use std::io::BufRead;

use crate::import::{ImportError, ImportErrorKind, IntoImportError};

/// Peekable reader to read input line by line.
pub struct LineReader<T> {
    reader: T,
    peek_buf: Option<String>,
    /// Line count, starting from 1.
    line_count: usize,
}

/// String with line information.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Line {
    /// actual line string.
    pub value: String,
    /// 1-based line count.
    pub line_count: usize,
}

impl<T> LineReader<T> {
    /// Creates a new instance.
    pub fn new(r: T) -> LineReader<T> {
        LineReader {
            reader: r,
            peek_buf: None,
            line_count: 0,
        }
    }

    /// Peek the next line. On either EOF or success, returns `Ok(())`.
    /// Use peek_buf() to check the content.
    pub fn peek(&mut self) -> Result<(), ImportError>
    where
        T: BufRead,
    {
        if self.peek_buf.is_some() {
            return Ok(());
        }
        self.peek_buf = self.read_next_line()?;
        Ok(())
    }

    /// Returns the peeked buffer.
    /// Note this could be merged into `peek()` once Polonius borrow checker becomes stable.
    pub fn peek_buf(&self) -> Option<&str> {
        self.peek_buf.as_deref()
    }

    fn take_peek_line(&mut self) -> Option<Line> {
        let buf = self.peek_buf.take()?;
        self.line_count += 1;
        Some(Line {
            value: buf,
            line_count: self.line_count,
        })
    }

    pub fn read_line(&mut self) -> Result<Option<Line>, ImportError>
    where
        T: BufRead,
    {
        if let Some(line) = self.take_peek_line() {
            return Ok(Some(line));
        }
        let Some(buf) = self.read_next_line()? else {
            // EOF
            return Ok(None);
        };
        self.line_count += 1;
        Ok(Some(Line {
            value: buf,
            line_count: self.line_count,
        }))
    }

    fn read_next_line(&mut self) -> Result<Option<String>, ImportError>
    where
        T: BufRead,
    {
        let mut buf = String::new();
        let read_bytes = self
            .reader
            .read_line(&mut buf)
            .into_import_err(ImportErrorKind::SourceFileReadFailed, || {
                format!("failed to peek the line @ {}", self.line_count + 1)
            })?;
        if read_bytes == 0 {
            return Ok(None);
        }
        Ok(Some(buf))
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
    fn line_reader_fails_to_peek_if_bufread_fails() {
        let r = ErrRead::new(
            b"",
            io::Error::new(io::ErrorKind::PermissionDenied, "expectde failure to read"),
        );
        let mut r = LineReader::new(io::BufReader::new(r));

        let got_err = r.peek().unwrap_err();
        assert_eq!(ImportErrorKind::SourceFileReadFailed, got_err.error_kind());
    }

    #[test]
    fn line_reader_fails_to_read_line_if_bufread_fails() {
        let r = ErrRead::new(
            b"this is a pen\n",
            io::Error::new(io::ErrorKind::PermissionDenied, "expectde failure to read"),
        );
        let mut r = LineReader::new(io::BufReader::new(r));

        r.peek().unwrap();
        assert_eq!(Some("this is a pen\n"), r.peek_buf());
        r.read_line().unwrap();
        let got_err = r.read_line().unwrap_err();
        assert_eq!(ImportErrorKind::SourceFileReadFailed, got_err.error_kind());
    }

    #[test]
    fn line_reader_peek_and_read_line() {
        let input = indoc! {"
            First line
            Second line
            Third line
        "};
        let mut r = LineReader::new(input.as_bytes());

        // first peek reads the next line.
        r.peek().unwrap();
        assert_eq!(Some("First line\n"), r.peek_buf());
        r.peek().unwrap();
        // second peek just returns the buffered value.
        assert_eq!(Some("First line\n"), r.peek_buf());
        assert_eq!(
            Some(Line {
                value: "First line\n".to_string(),
                line_count: 1,
            }),
            r.read_line().unwrap()
        );
        assert_eq!(
            Some(Line {
                value: "Second line\n".to_string(),
                line_count: 2,
            }),
            r.read_line().unwrap()
        );
        r.peek().unwrap();
        assert_eq!(Some("Third line\n"), r.peek_buf());
        assert_eq!(
            Some(Line {
                value: "Third line\n".to_string(),
                line_count: 3,
            }),
            r.read_line().unwrap()
        );
        r.peek().unwrap();
        assert_eq!(None, r.peek_buf());
        assert_eq!(None, r.read_line().unwrap());
    }
}
