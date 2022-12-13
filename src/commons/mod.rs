//! Sets of commons [TokenFilter](tantivy::tokenizer::TokenFilter) and [Tokenizer](tantivy::tokenizer::Tokenizer).
//!
//! Currently :
//! * [LengthTokenFilter] : keep tokens that match length criteria.
//! * [TrimTokenFilter] : trim leading and trailing whitespace.
//! * [LimitTokenCountFilter] : limit the number of token.
//! * [PathTokenizer] : tokenize a path hierarchy.
//! * [ReverseTokenFilter] : a filter that reverse the string.
//! * [ElisionTokenFilter] : a filter that remove elisions.
//! * [EdgeNgramTokenFilter] : a token filter that produce 'edge-ngram'.
pub use crate::commons::edge_ngram::{EdgeNgramError, EdgeNgramTokenFilter};
pub use crate::commons::elision::ElisionTokenFilter;
pub use crate::commons::length::LengthTokenFilter;
pub use crate::commons::limit::LimitTokenCountFilter;
pub use crate::commons::path::{PathTokenizer, PathTokenizerBuilder};
pub use crate::commons::reverse::ReverseTokenFilter;
#[deprecated(
    since = "0.6.0",
    note = "Tantivy already provide one. Users should use it instead"
)]
pub use crate::commons::trim::TrimTokenFilter;
pub use fst::Set;

mod edge_ngram;
mod elision;
mod length;
mod limit;
mod path;
mod reverse;
mod trim;
