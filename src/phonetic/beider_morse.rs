use rphonetic::{BeiderMorse, Encoder, LanguageSet};
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct BeiderMorseTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: BeiderMorse,
    pub codes: Vec<String>,
    pub languages: Option<LanguageSet>,
}

impl<'a> TokenStream for BeiderMorseTokenStream<'a> {
    fn advance(&mut self) -> bool {
        while self.codes.is_empty() {
            if !self.tail.advance() {
                return false;
            }

            let encoded = match &self.languages {
                None => self.encoder.encode(&self.tail.token().text),
                Some(languages) => self
                    .encoder
                    .encode_with_languages(&self.tail.token().text, languages),
            };
            let mut start_token = 0;
            let mut end_token = 0;
            let mut start = true;
            // "Simple" parsing of potential nested (...|...|...)-(...|...|...)
            for (index, ch) in encoded.char_indices() {
                if ch != '(' && ch != ')' && ch != '-' && ch != '|' {
                    if start {
                        start_token = index;
                        end_token = index;
                        start = false;
                    } else {
                        end_token += 1;
                    }
                } else if start_token < end_token {
                    self.codes
                        .push(encoded[start_token..=end_token].to_string());
                    start_token = end_token;
                    start = true;
                }
            }
        }

        // Here self.codes can't be empty because of "early" return
        // when self.tail.advance() is false.
        self.tail.token_mut().text = self.codes.pop().unwrap();
        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
