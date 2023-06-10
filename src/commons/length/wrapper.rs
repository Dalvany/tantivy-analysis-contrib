//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use tantivy::tokenizer::Tokenizer;

use super::LengthTokenStream;

#[derive(Clone, Debug)]
pub(crate) struct LengthFilterWrapper<T> {
    min: Option<usize>,
    max: Option<usize>,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for LengthFilterWrapper<T> {
    type TokenStream<'a> = LengthTokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        LengthTokenStream {
            tail: self.inner.token_stream(text),
            min: self.min,
            max: self.max,
        }
    }
}
