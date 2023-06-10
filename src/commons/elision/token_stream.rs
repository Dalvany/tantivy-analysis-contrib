//! Module that contains the [TokenStream] implementation. It's this that
//! do the real job.

use std::sync::Arc;

use rustc_hash::FxHashSet;
use tantivy::tokenizer::{Token, TokenStream};

#[derive(Clone, Debug)]
pub(crate) struct ElisionTokenStream<T> {
    tail: T,
    // Use a BTreeSet as this set should be small otherwise use HashSet.
    elisions: Arc<FxHashSet<String>>,
    ignore_case: bool,
}

impl<T: TokenStream> TokenStream for ElisionTokenStream<T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        let token = &self.tail.token().text;
        let found: Option<(usize, char)> = token.char_indices().find(|(_, ch)| ch == &'\'');
        if let Some((index, _)) = found {
            let prefix = &self.tail.token().text[0..index];
            let contains = if self.ignore_case {
                self.elisions.contains(&prefix.to_lowercase())
            } else {
                self.elisions.contains(prefix)
            };
            if contains {
                self.tail.token_mut().text = token[index + 1..].to_string();
                self.tail.token_mut().offset_from = self.tail.token_mut().offset_from + index + 1;
            }
        }

        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
