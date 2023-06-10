//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use tantivy::tokenizer::Tokenizer;

use super::LimitTokenCountStream;

#[derive(Clone, Debug)]
pub(crate) struct LimitTokenCountWrapper<T> {
    count: usize,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for LimitTokenCountWrapper<T> {
    type TokenStream<'a> = LimitTokenCountStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        LimitTokenCountStream {
            count: self.count,
            tail: self.inner.token_stream(text),
        }
    }
}