pub(crate) use format::FormatError;
use okane_core::format;

use std::io::{Read, Write};

/// Converts given string into formatted string.
pub fn format<R, W>(r: &mut R, w: &mut W) -> Result<(), format::FormatError>
where
    R: Read,
    W: Write,
{
    format::FormatOptions::new().recursive(false).format(r, w)
}
