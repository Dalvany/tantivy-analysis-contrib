//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use tantivy::tokenizer::Tokenizer;

use super::{ICUNormalizer2TokenStream, Mode};

#[derive(Debug, Clone)]
pub struct ICUNormalizer2FilterWrapper<T> {
    mode: Mode,
    inner: T,
}

impl<T> ICUNormalizer2FilterWrapper<T> {
    pub(crate) fn new(inner: T, mode: Mode) -> Self {
        Self { mode, inner }
    }
}

impl<T: Tokenizer> Tokenizer for ICUNormalizer2FilterWrapper<T> {
    type TokenStream<'a> = ICUNormalizer2TokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        // It's safe to unwrap here, we check that its work in token filter's new method
        ICUNormalizer2TokenStream::new(
            self.inner.token_stream(text),
            self.mode.try_into().expect("Can't convert into normalizer"),
        )
    }
}
