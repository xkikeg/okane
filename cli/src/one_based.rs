//! Provides OneBasedIndex, which wraps usize as 1-based index.

use std::{fmt::Display, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Serialize};

/// Represents 1-based index.
///
/// From human point of view, often 1-based index is easier than 0-based.
/// But program solely needs 0-based. `OneBasedIndex` provides
/// safety and easiness to use.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OneBasedIndex(usize);

impl Display for OneBasedIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_one_based())
    }
}

impl FromStr for OneBasedIndex {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: usize = s.parse()?;
        Self::from_one_based(v).map_err(ParseError::InvalidInput)
    }
}

impl Serialize for OneBasedIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.as_one_based() as u64)
    }
}

impl<'de> Deserialize<'de> for OneBasedIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: u64 = Deserialize::deserialize(deserializer)?;
        use serde::de::{self, Error};
        Self::from_one_based(v as usize).map_err(|_| {
            D::Error::invalid_value(de::Unexpected::Unsigned(v), &"a positive integer")
        })
    }
}

impl OneBasedIndex {
    /// Creates OneBasedIndex from 1-based index.
    pub fn from_one_based(v: usize) -> Result<Self, Error> {
        if v == 0 {
            Err(Error::InvalidIndex)
        } else {
            Ok(OneBasedIndex(v - 1))
        }
    }

    /// Creates OneBasedIndex from 0-based index.
    pub fn from_zero_based(v: usize) -> Self {
        Self(v)
    }

    /// Returns regular 0-based index.
    pub fn as_zero_based(&self) -> usize {
        self.0
    }

    /// Returns 1-based index.
    pub fn as_one_based(&self) -> usize {
        self.as_zero_based() + 1
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! one_based {
    ( $x:expr ) => {
        one_based::OneBasedIndex::from_one_based($x).unwrap()
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("1-based origin must not be greater than zero")]
    InvalidIndex,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid input: {0:?}")]
    InvalidInput(#[from] Error),
    #[error("failed to parse as integer: {0:?}")]
    ParseFailure(#[from] ParseIntError),
}
