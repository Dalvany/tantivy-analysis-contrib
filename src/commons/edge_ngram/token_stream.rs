//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use tantivy::tokenizer::{Token, TokenStream};

#[derive(Clone, Debug)]
pub struct EdgeNgramFilterStream<T> {
    pub(crate) tail: T,
    /// Current token to emit
    pub(crate) token: Token,
    /// Minimum ngram must be greater than 0
    pub(crate) min: usize,
    /// Maximum ngram, None means no limit
    pub(crate) max: Option<usize>,
    /// Which ngram we should emit
    pub(crate) count: usize,
    /// Do we have to keep the original token?
    pub(crate) keep_original_token: bool,
    /// Avoid doing multiple time self.tail.token().chars().count()
    pub(crate) current_len: usize,
    /// Stop at
    pub(crate) stop_length: usize,
}

impl<T: TokenStream> TokenStream for EdgeNgramFilterStream<T> {
    fn advance(&mut self) -> bool {
        loop {
            // if count = min then we begin a new token...
            if self.count == self.min {
                if !self.tail.advance() {
                    return false;
                }

                self.token = self.tail.token().clone();
                // Reset everything with new token
                self.current_len = self.tail.token().text.chars().count();

                // If we have to keep the original token but its length
                // is lower than min, then we force output it
                // otherwise it won't be emitted.
                if self.keep_original_token && self.current_len < self.min {
                    return true;
                }

                // We stop if we reach the end of the token or max (if present).
                self.stop_length =
                    std::cmp::min(self.max.unwrap_or(self.current_len), self.current_len);
            }

            if self.count <= self.stop_length {
                let token_string: String =
                    self.tail.token().text.chars().take(self.count).collect();
                self.token.text = token_string;

                // We have reached the end of token, so we reset the count to min
                if self.count == self.stop_length {
                    if self.stop_length == self.current_len
                        || (self.max.is_some() && !self.keep_original_token)
                    {
                        // If we reach the end of token then reset
                        // Or
                        // If we have a max, we have reached it if we
                        // do not have to keep the original token then reset
                        self.count = self.min;
                    } else {
                        self.count += 1;
                    }
                } else {
                    self.count += 1;
                }

                return true;
            } else {
                self.count = self.min;
                if self.keep_original_token {
                    self.token.text = self.tail.token().text.clone();
                    return true;
                }
            }
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}
