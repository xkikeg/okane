//! Module decoration defines the trait,
//! which describes the extra information attached to [crate::syntax] data.

use std::fmt::Debug;

/// `AsUndecorated<T>` is equivalent to [AsRef], with specific meaning.
/// AsRef can be too generic and not suitable for use case.
pub trait AsUndecorated<T: ?Sized> {
    fn as_undecorated(&self) -> &T;
}

/// Decoration associates the extra information attached to
/// [Transaction][super::Transaction] or any other [crate::syntax] data.
/// See [super::plain] or [super::tracked] for implementations.
pub trait Decoration: 'static {
    type Decorated<T>: AsUndecorated<T> + Debug + PartialEq + Eq
    where
        T: AsUndecorated<T> + Debug + PartialEq + Eq;

    fn decorate_parser<PIn, I, O, E>(parser: PIn) -> impl winnow::Parser<I, Self::Decorated<O>, E>
    where
        I: winnow::stream::Stream + winnow::stream::Location,
        O: AsUndecorated<O> + Debug + PartialEq + Eq,
        PIn: winnow::Parser<I, O, E>;
}

macro_rules! define_as_undecorated {
    ([$($impl_generics:tt)*],
     $type_name:ty) => {
        impl<$($impl_generics)*> AsUndecorated<$type_name> for $type_name {
            fn as_undecorated(&self) -> &$type_name {
                self
            }
        }
    };
}

define_as_undecorated!(['i, Deco: Decoration], super::LedgerEntry<'i, Deco>);
define_as_undecorated!(['i, Deco: Decoration], super::Transaction<'i, Deco>);
define_as_undecorated!(['i, Deco: Decoration], super::Posting<'i, Deco>);
define_as_undecorated!(['i, Deco: Decoration], super::PostingAmount<'i, Deco>);
define_as_undecorated!(['i], super::Exchange<'i>);
define_as_undecorated!(['i], super::expr::ValueExpr<'i>);
