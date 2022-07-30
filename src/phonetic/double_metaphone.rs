use rphonetic::DoubleMetaphone;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct DoubleMetaphoneTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: DoubleMetaphone,
    pub codes: Vec<String>,
    pub inject: bool,
}

impl<'a> TokenStream for DoubleMetaphoneTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.codes.is_empty() {
            let mut result = false;
            while !result {
                result = self.tail.advance();
                if !result {
                    return false;
                }
                if self.tail.token().text.is_empty() {
                    return true;
                }

                let encoded = self.encoder.double_metaphone(&self.tail.token().text);
                let primary = encoded.primary();
                let alternate = encoded.alternate();
                if primary.is_empty() && alternate.is_empty() && self.inject {
                    return true;
                }
                if !primary.is_empty() && !primary.is_empty() {
                    // None of the primary and the alternate are empty, then we set the whole thing.
                    if self.inject {
                        self.codes.push(primary);
                    } else {
                        self.tail.token_mut().text = primary;
                    }
                    self.codes.push(alternate);
                    result = true;
                } else if !primary.is_empty() {
                    // Primary is not empty, but alternate is empty otherwise, it
                    // runs the first if.
                    if self.inject {
                        self.codes.push(primary);
                    } else {
                        self.tail.token_mut().text = primary;
                    }
                    result = true;
                } else if !alternate.is_empty() {
                    // Alternate is not empty, but primary is empty otherwise, it
                    // runs the first if or the first else-if.
                    if self.inject {
                        self.codes.push(alternate);
                    } else {
                        self.tail.token_mut().text = alternate;
                    }
                    result = true;
                }
            }
            result
        } else {
            self.tail.token_mut().text = self.codes.pop().unwrap();
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
