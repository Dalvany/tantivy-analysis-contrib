//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use std::sync::Arc;

use rustc_hash::FxHashSet;
use tantivy::tokenizer::Tokenizer;

use super::ElisionTokenStream;

#[derive(Clone, Debug)]
pub(crate) struct ElisionFilterWrapper<T> {
    elisions: Arc<FxHashSet<String>>,
    ignore_case: bool,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for ElisionFilterWrapper<T> {
    type TokenStream<'a> = ElisionTokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        ElisionTokenStream {
            tail: self.inner.token_stream(text),
            elisions: self.elisions.clone(),
            ignore_case: self.ignore_case,
        }
    }
}
