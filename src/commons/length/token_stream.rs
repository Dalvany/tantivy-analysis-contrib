//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use tantivy::tokenizer::{Token, TokenStream};

#[derive(Clone, Debug)]
pub struct LengthTokenStream<T> {
    tail: T,
    min: Option<usize>,
    max: Option<usize>,
}

impl<T> LengthTokenStream<T> {
    pub(crate) fn new(tail: T, min: Option<usize>, max: Option<usize>) -> Self {
        Self { tail, min, max }
    }
}

impl<T: TokenStream> TokenStream for LengthTokenStream<T> {
    fn advance(&mut self) -> bool {
        let mut result = true;
        let mut length_ok = false;
        while result && !length_ok {
            result = self.tail.advance();
            if result {
                let size = self.tail.token().text.len();
                length_ok =
                    self.min.map_or(true, |v| v <= size) && self.max.map_or(true, |v| size <= v);
            }
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
