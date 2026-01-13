//! Provides macro to create one_based instance easily.

macro_rules! one_based_32 {
    ( $x:expr ) => {
        ::one_based::OneBasedU32::from_one_based($x).unwrap()
    };
}

macro_rules! zero_based_usize {
    ( $x:expr ) => {
        ::one_based::OneBasedUsize::from_zero_based($x).unwrap()
    };
}

pub(crate) use one_based_32;
pub(crate) use zero_based_usize;
