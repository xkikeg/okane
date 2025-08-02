use std::mem::MaybeUninit;

/// FixedVec is similar to `Vec`, but can't be resized beyond the compile-time spcecified size.
pub struct FixedVec<const N: usize, T> {
    size: usize,
    buf: [MaybeUninit<T>; N],
}

impl<const N: usize, T> Default for FixedVec<N, T> {
    fn default() -> Self {
        Self {
            size: 0,
            buf: [const { MaybeUninit::uninit() }; N],
        }
    }
}

impl<const N: usize, T> PartialEq for FixedVec<N, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        todo!("missing")
    }
}

impl<const N: usize, T> FixedVec<N, T> {}
