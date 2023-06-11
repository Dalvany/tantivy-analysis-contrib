//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use tantivy::tokenizer::Tokenizer;

use super::LimitTokenCountStream;

#[derive(Clone, Debug)]
pub struct LimitTokenCountFilterWrapper<T> {
    count: usize,
    inner: T,
}

impl<T> LimitTokenCountFilterWrapper<T> {
    pub(crate) fn new(inner: T, count: usize) -> Self {
        Self { count, inner }
    }
}

impl<T: Tokenizer> Tokenizer for LimitTokenCountFilterWrapper<T> {
    type TokenStream<'a> = LimitTokenCountStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        LimitTokenCountStream::new(self.inner.token_stream(text), self.count)
    }
}
