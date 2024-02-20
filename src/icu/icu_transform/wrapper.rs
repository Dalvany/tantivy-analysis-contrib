//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use rust_icu_utrans as utrans;
use tantivy_tokenizer_api::Tokenizer;

use super::{Direction, ICUTransformTokenStream};

#[derive(Debug, Clone)]
pub struct ICUTransformFilterWrapper<T> {
    compound_id: String,
    rules: Option<String>,
    direction: Direction,
    inner: T,
}

impl<T> ICUTransformFilterWrapper<T> {
    pub(crate) fn new(
        inner: T,
        compound_id: String,
        rules: Option<String>,
        direction: Direction,
    ) -> Self {
        Self {
            compound_id,
            rules,
            direction,
            inner,
        }
    }
}

impl<T: Tokenizer> Tokenizer for ICUTransformFilterWrapper<T> {
    type TokenStream<'a> = ICUTransformTokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        // unwrap work, we checked in token filter's new method.
        let transform = utrans::UTransliterator::new(
            self.compound_id.as_str(),
            self.rules.as_deref(),
            self.direction.into(),
        )
        .expect("Can't create transliterator");

        ICUTransformTokenStream::new(self.inner.token_stream(text), transform)
    }
}
