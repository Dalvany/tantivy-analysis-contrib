//! Sets of commons [TokenFilter].
//!
//! Currently :
//! * [LengthTokenFilter] : keep tokens that match length criteria.
//! * [TrimTokenFilter] : trim leading and trailing whitespace.
//! * [LimitTokenCountFilter] : limit the number of token.
//! * [PathTokenizer] : tokenize a path hierarchy.
pub use crate::commons::length::LengthTokenFilter;
pub use crate::commons::limit::LimitTokenCountFilter;
pub use crate::commons::path::PathTokenizer;
pub use crate::commons::trim::TrimTokenFilter;

mod length;
mod limit;
mod path;
mod trim;
