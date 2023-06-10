use rust_icu_utrans as utrans;
use tantivy::tokenizer::Tokenizer;

use super::{Direction, ICUTransformTokenStream};

#[derive(Debug, Clone)]
pub(crate) struct ICUTransformFilterWrapper<T> {
    compound_id: String,
    rules: Option<String>,
    direction: Direction,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for ICUTransformFilterWrapper<T> {
    type TokenStream<'a> = ICUTransformTokenStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        ICUTransformTokenStream {
            // unwrap work, we checked in token filter's new method.
            transform: utrans::UTransliterator::new(
                self.compound_id.as_str(),
                self.rules.as_deref(),
                self.direction.into(),
            )
            .expect("Can't create transliterator"),
            tail: self.inner.token_stream(text),
            temp: String::with_capacity(100),
        }
    }
}
