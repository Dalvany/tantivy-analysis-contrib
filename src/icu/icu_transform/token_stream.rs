//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use std::mem;

use rust_icu_utrans as utrans;
use tantivy::tokenizer::{Token, TokenStream};

#[derive(Debug)]
pub struct ICUTransformTokenStream<T> {
    transform: utrans::UTransliterator,
    tail: T,
    temp: String,
}

impl<T> ICUTransformTokenStream<T> {
    pub(crate) fn new(tail: T, transform: utrans::UTransliterator) -> Self {
        Self {
            transform,
            tail,
            temp: String::with_capacity(100),
        }
    }
}

impl<T: TokenStream> TokenStream for ICUTransformTokenStream<T> {
    fn advance(&mut self) -> bool {
        let result = self.tail.advance();
        if !result {
            return false;
        }
        if let Ok(t) = self.transform.transliterate(&self.tail.token().text) {
            self.temp = t;
            mem::swap(&mut self.tail.token_mut().text, &mut self.temp);
        }
        result
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
