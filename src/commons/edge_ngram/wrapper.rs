//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use std::num::NonZeroUsize;

use tantivy::tokenizer::Tokenizer;

use super::EdgeNgramFilterStream;

#[derive(Clone, Debug)]
pub(crate) struct EdgeNgramFilterWrapper<T> {
    min: NonZeroUsize,
    max: Option<NonZeroUsize>,
    keep_original_token: bool,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for EdgeNgramFilterWrapper<T> {
    type TokenStream<'a> = EdgeNgramFilterStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        EdgeNgramFilterStream  {
            tail: self.inner.token_stream(text),
            token: Default::default(),
            min: self.min.get(),
            max: self.max.map(|v| v.get()),
            count: self.min.get(),
            keep_original_token: self.keep_original_token,
            current_len: 0,
            stop_length: 0,
        }
    }
}