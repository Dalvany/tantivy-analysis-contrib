use rphonetic::DoubleMetaphone;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct DoubleMetaphoneTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: DoubleMetaphone,
    pub alternate: Option<String>,
}

impl<'a> TokenStream for DoubleMetaphoneTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.alternate.is_none() {
            let mut result = false;
            while !result {
                result = self.tail.advance();
                if !result {
                    return false;
                }

                let encoded = self.encoder.double_metaphone(&self.tail.token().text);
                let primary = encoded.primary();
                let alternate = encoded.alternate();
                if !primary.is_empty() && !primary.is_empty() {
                    // None of the primary and the alternate are empty, then we set the whole thing.
                    self.tail.token_mut().text = primary;
                    self.alternate = Some(alternate);
                    result = true;
                } else if !primary.is_empty() {
                    // Primary is not empty, but alternate is empty otherwise, it
                    // runs the first if.
                    self.tail.token_mut().text = primary;
                    self.alternate = None;
                    result = true;
                } else if !alternate.is_empty() {
                    // Alternate is not empty, but primary is empty otherwise, it
                    // runs the first if or the first else-if.
                    self.tail.token_mut().text = alternate;
                    self.alternate = None;
                    result = true;
                }
            }
            result
        } else {
            self.tail.token_mut().text = self.alternate.as_ref().unwrap().to_string();
            self.alternate = None;
            true
        }
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
