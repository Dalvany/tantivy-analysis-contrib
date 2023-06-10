use tantivy::tokenizer::Tokenizer;

use super::{ICUNormalizer2TokenStream, Mode};

#[derive(Debug, Clone)]
pub(crate) struct ICUNormalizer2FilterWrapper<T> {
    mode: Mode,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for ICUNormalizer2FilterWrapper<T> {
    type TokenStream<'a> = ICUNormalizer2TokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        // It's safe to unwrap here, we check that its work in token filter's new method
        ICUNormalizer2TokenStream {
            normalizer: self.mode.try_into().expect("Can't convert into normalizer"),
            tail: self.inner.token_stream(text),
            temp: String::with_capacity(100),
        }
    }
}
