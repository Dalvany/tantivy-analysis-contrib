//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use std::mem;

use rust_icu_unorm2::UNormalizer;
use tantivy::tokenizer::{Token, TokenStream};

#[derive(Debug)]
pub struct ICUNormalizer2TokenStream<T> {
    normalizer: UNormalizer,
    tail: T,
    temp: String,
}

impl<T> ICUNormalizer2TokenStream<T> {
    pub(crate) fn new(tail: T, normalizer: UNormalizer) -> Self {
        Self {
            normalizer,
            tail,
            temp: String::with_capacity(100),
        }
    }
}

impl<T: TokenStream> TokenStream for ICUNormalizer2TokenStream<T> {
    fn advance(&mut self) -> bool {
        let result = self.tail.advance();
        if !result {
            return false;
        }

        if let Ok(t) = self.normalizer.normalize(&self.tail.token().text) {
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
