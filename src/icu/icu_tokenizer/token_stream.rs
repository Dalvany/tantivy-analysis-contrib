use std::str::Chars;

use rust_icu_ubrk::UBreakIterator;
use tantivy_tokenizer_api::{Token, TokenStream};

struct ICUBreakingWord<'a> {
    text: Chars<'a>,
    default_breaking_iterator: UBreakIterator,
}

impl<'a> std::fmt::Debug for ICUBreakingWord<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ICUBreakingWord")
            .field("text", &self.text)
            .finish()
    }
}

impl<'a> From<&'a str> for ICUBreakingWord<'a> {
    fn from(text: &'a str) -> Self {
        ICUBreakingWord {
            text: text.chars(),
            default_breaking_iterator: UBreakIterator::try_new_rules(super::DEFAULT_RULES, text)
                .expect("Can't read default rules."),
        }
    }
}

impl<'a> Iterator for ICUBreakingWord<'a> {
    type Item = (String, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // It is a port in Rust of Lucene algorithm
        let mut cont = true;
        let mut start = self.default_breaking_iterator.current();
        let mut end = self.default_breaking_iterator.next();
        while cont && end.is_some() {
            if end.is_some() && self.default_breaking_iterator.get_rule_status() == 0 {
                start = end.unwrap();
                end = self.default_breaking_iterator.next();
            }
            if let Some(index) = end {
                cont = !self
                    .text
                    .clone()
                    .take(index as usize)
                    .skip(start as usize)
                    .any(char::is_alphanumeric);
            }
        }

        match end {
            None => None,
            Some(index) => {
                let substring: String = self
                    .text
                    .clone()
                    .take(index as usize)
                    .skip(start as usize)
                    .collect();
                Some((substring, start as usize, index as usize))
            }
        }
    }
}

#[derive(Debug)]
pub struct ICUTokenizerTokenStream<'a> {
    breaking_word: ICUBreakingWord<'a>,
    token: Token,
}

impl<'a> ICUTokenizerTokenStream<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        ICUTokenizerTokenStream {
            breaking_word: ICUBreakingWord::from(text),
            token: Token::default(),
        }
    }
}

impl<'a> TokenStream for ICUTokenizerTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let token = self.breaking_word.next();
        match token {
            None => false,
            Some(token) => {
                self.token.text.clear();
                self.token.position = self.token.position.wrapping_add(1);
                self.token.offset_from = token.1;
                self.token.offset_to = token.2;
                self.token.text.push_str(&token.0);
                true
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
