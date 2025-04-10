//! Defines simplest [Decoration] [Ident], which does decorate nothing.
//! Also provides type aliases with [Ident].

use std::fmt::Debug;

use super::decoration::{AsUndecorated, Decoration};

/// Ident is a [Decoration] that attaches no extra information.
/// Good to use if you just want a syntax type.
pub struct Ident;

impl Decoration for Ident {
    type Decorated<T>
        = T
    where
        T: AsUndecorated<T> + Debug + PartialEq + Eq;

    fn decorate_parser<PIn, I, O, E>(parser: PIn) -> impl winnow::Parser<I, Self::Decorated<O>, E>
    where
        I: winnow::stream::Stream + winnow::stream::Location,
        O: AsUndecorated<O> + Debug + PartialEq + Eq,
        PIn: winnow::Parser<I, O, E>,
    {
        parser
    }
}

pub type LedgerEntry<'i> = super::LedgerEntry<'i, Ident>;
pub type Transaction<'i> = super::Transaction<'i, Ident>;
pub type Posting<'i> = super::Posting<'i, Ident>;
pub type PostingAmount<'i> = super::PostingAmount<'i, Ident>;
pub type Lot<'i> = super::Lot<'i, Ident>;
