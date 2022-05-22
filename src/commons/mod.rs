//! Sets of commons [TokenFilter](tantivy::tokenizer::TokenFilter) and [Tokenizer](tantivy::tokenizer::Tokenizer).
//!
//! Currently :
//! * [LengthTokenFilter] : keep tokens that match length criteria.
//! * [TrimTokenFilter] : trim leading and trailing whitespace.
//! * [LimitTokenCountFilter] : limit the number of token.
//! * [PathTokenizer] : tokenize a path hierarchy.
//! * [ReverseTokenFilter] : a filter that reverse the string.
//! * [ElisionTokenFilter] : a filter that remove elisions.
pub use crate::commons::elision::ElisionTokenFilter;
pub use crate::commons::length::LengthTokenFilter;
pub use crate::commons::limit::LimitTokenCountFilter;
pub use crate::commons::path::{PathTokenizer, PathTokenizerBuilder};
pub use crate::commons::reverse::ReverseTokenFilter;
pub use crate::commons::trim::TrimTokenFilter;

mod elision;
mod length;
mod limit;
mod path;
mod reverse;
mod trim;
