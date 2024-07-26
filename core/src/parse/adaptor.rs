//! Provides adaptor structs for winnow.

use smallvec::{Array, SmallVec};
use winnow::stream::Accumulate;

/// Adaptor for [SmallVec].
pub struct SmallVecAdaptor<T>
where
    T: Array,
{
    wrapped: SmallVec<T>,
}

impl<T> Accumulate<<T as Array>::Item> for SmallVecAdaptor<T>
where
    T: Array,
{
    fn initial(capacity: Option<usize>) -> Self {
        let wrapped = match capacity {
            None => SmallVec::new(),
            Some(capacity) => SmallVec::with_capacity(capacity),
        };
        Self { wrapped }
    }

    fn accumulate(&mut self, acc: <T as Array>::Item) {
        self.wrapped.push(acc)
    }
}

impl<T: Array> From<SmallVecAdaptor<T>> for SmallVec<T> {
    fn from(value: SmallVecAdaptor<T>) -> Self {
        value.wrapped
    }
}
