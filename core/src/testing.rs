#![cfg(test)]

use std::{error::Error, fmt::Display};

pub fn recursive_print<E: Error>(e: E) -> impl Display {
    RecursivePrint(e)
}
/// recursively prints the error.
struct RecursivePrint<E>(E);

impl<E: Error> Display for RecursivePrint<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cur: Option<&dyn Error> = Some(&self.0);
        let mut index = 0;
        while let Some(e) = cur {
            write!(f, "error {}: {}", index, e)?;
            cur = e.source();
            index += 1;
        }
        Ok(())
    }
}
