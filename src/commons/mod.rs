//! Sets of commons [TokenFilter].
//!
//! Currently :
//! * [LengthTokenFilter] : keep tokens that match length criteria.
//! * [TrimTokenFilter] : trim leading and trailing whitespace.
mod length;
mod trim;

pub use crate::commons::length::LengthTokenFilter;
pub use crate::commons::trim::TrimTokenFilter;
