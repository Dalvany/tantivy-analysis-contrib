//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

// TODO Allow marker ?

use std::mem;

use tantivy::tokenizer::{Token, TokenStream};

#[derive(Debug, Clone)]
pub struct ReverseTokenStream<T> {
    tail: T,
}

impl<T> ReverseTokenStream<T> {
    pub(crate) fn new(tail: T) -> Self {
        Self { tail }
    }
}

impl<T: TokenStream> TokenStream for ReverseTokenStream<T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        let mut buffer = self.tail.token().text.clone().chars().rev().collect();
        mem::swap(&mut self.tail.token_mut().text, &mut buffer);

        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
