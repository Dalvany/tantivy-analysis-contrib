//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use tantivy::tokenizer::{TokenStream, Token};

#[derive(Debug, Clone)]
pub(crate) struct LimitTokenCountStream<T> {
    tail: T,
    count: usize,
}

impl<T: TokenStream> TokenStream for LimitTokenCountStream<T> {
    fn advance(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }

        if !self.tail.advance() {
            return false;
        }

        self.count -= 1;

        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}