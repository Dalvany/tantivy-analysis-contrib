use std::iter::{Rev, Skip};
use std::str::Split;

use either::Either;
use tantivy::tokenizer::{Token, TokenStream};

#[derive(Debug, Clone)]
pub struct PathTokenStream<'a> {
    pub(crate) text: Skip<Either<Split<'a, char>, Rev<Split<'a, char>>>>,
    pub(crate) buffer: String,
    pub(crate) token: Token,
    pub(crate) separator: char,
    pub(crate) offset: usize,
    pub(crate) starts_with: bool,
    pub(crate) reverse: bool,
}

impl<'a> TokenStream for PathTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if let Some(part) = self.text.next() {
            if !self.starts_with {
                // Do not add the separator (or replacement) if it doesn't start (or end) with the separator
                self.starts_with = true;
            } else if self.reverse {
                self.buffer.insert(0, self.separator);
            } else {
                self.buffer.push(self.separator);
            }

            if self.reverse {
                self.buffer.insert_str(0, part);
            } else {
                self.buffer.push_str(part);
            }

            let offset_from = if self.reverse {
                self.offset - self.buffer.len()
            } else {
                self.offset
            };

            let offset_to = if self.reverse {
                self.offset
            } else {
                self.offset + self.buffer.len()
            };

            self.token = Token {
                offset_from,
                offset_to,
                position: 0,
                text: self.buffer.clone(),
                position_length: 1,
            };
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}
