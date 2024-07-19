// Copyright 2018-2023 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sequence-related functionality
//!
//! This module provides:
//!
//! *   [`IndexedRandom`] for sampling slices and other indexable lists
//! *   [`IndexedMutRandom`] for sampling slices and other mutably indexable lists
//! *   [`SliceRandom`] for mutating slices
//! *   [`IteratorRandom`] for sampling iterators
//! *   [`index::sample`] low-level API to choose multiple indices from
//!     `0..length`
//!
//! Also see:
//!
//! *   [`crate::distr::WeightedIndex`] distribution which provides
//!     weighted index sampling.
//!
//! In order to make results reproducible across 32-64 bit architectures, all
//! `usize` indices are sampled as a `u32` where possible (also providing a
//! small performance boost in some cases).

mod coin_flipper;
mod increasing_uniform;
mod iterator;
mod slice;

#[cfg(feature = "alloc")]
#[path = "index.rs"]
mod index_;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use crate::distr::WeightError;
pub use iterator::IteratorRandom;
#[cfg(feature = "alloc")]
pub use slice::SliceChooseIter;
pub use slice::{IndexedMutRandom, IndexedRandom, SliceRandom};

use crate::Rng;

// Sample a number uniformly between 0 and `ubound`. Uses 32-bit sampling where
// possible, primarily in order to produce the same output on 32-bit and 64-bit
// platforms.
#[inline]
fn gen_index<R: Rng + ?Sized>(rng: &mut R, ubound: usize) -> usize {
    if ubound <= (u32::MAX as usize) {
        rng.gen_range(0..ubound as u32) as usize
    } else {
        #[cfg(target_pointer_width = "32")]
        unreachable!();

        #[cfg(target_pointer_width = "64")]
        return rng.gen_range(0..ubound as u64) as usize;
    }
}

/// Low-level API for sampling indices
pub mod index {
    use super::gen_index;
    use crate::Rng;

    #[cfg(feature = "alloc")]
    #[doc(inline)]
    pub use super::index_::*;

    /// Randomly sample exactly `N` distinct indices from `0..len`, and
    /// return them in random order (fully shuffled).
    ///
    /// This is implemented via Floyd's algorithm. Time complexity is `O(N^2)`
    /// and memory complexity is `O(N)`.
    ///
    /// Returns `None` if (and only if) `N > len`.
    pub fn sample_array<R, const N: usize>(rng: &mut R, len: usize) -> Option<[usize; N]>
    where
        R: Rng + ?Sized,
    {
        if N > len {
            return None;
        }

        // Floyd's algorithm
        let mut indices = [0; N];
        for (i, j) in (len - N..len).enumerate() {
            let t = gen_index(rng, j + 1);
            if let Some(pos) = indices[0..i].iter().position(|&x| x == t) {
                indices[pos] = j;
            }
            indices[i] = t;
        }
        Some(indices)
    }
}
