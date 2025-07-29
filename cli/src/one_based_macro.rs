//! Provides macro to create one_based instance easily.

#![cfg(test)]

macro_rules! one_based_32 {
    ( $x:expr ) => {
        ::one_based::OneBasedU32::from_one_based($x).unwrap()
    };
}

pub(crate) use one_based_32;
