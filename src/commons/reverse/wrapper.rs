//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use tantivy::tokenizer::Tokenizer;

use super::ReverseTokenStream;

#[derive(Clone, Debug)]
pub(crate) struct ReverseFilterWrapper<T> {
    inner: T,
}

impl<T: Tokenizer> Tokenizer for ReverseFilterWrapper<T> {
    type TokenStream<'a> = ReverseTokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        ReverseTokenStream {
            tail: self.inner.token_stream(text),
        }
    }
}
