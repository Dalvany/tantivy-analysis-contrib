use std::mem;

use rust_icu::norm::UNormalizer;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

impl From<Type> for UNormalizer {
    fn from(tp: Type) -> Self {
        match tp {
            Type::NFC => UNormalizer::new_nfc().expect("Can't create NFC normalizer"),
            Type::NFD => UNormalizer::new_nfd().expect("Can't create NFD normalizer"),
            Type::NFKC => UNormalizer::new_nfkc().expect("Can't create NFKC normalizer"),
            Type::NFKD => UNormalizer::new_nfkd().expect("Can't create NFKD normalizer"),
            Type::NFKCCasefold => {
                UNormalizer::new_nfkc_casefold().expect("Can't create NFKC casefold normalizer")
            }
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Type {
    NFC,
    NFD,
    NFKC,
    NFKD,
    NFKCCasefold,
}

struct ICUNormalizer2TokenStream<'a> {
    normalizer: UNormalizer,
    tail: BoxTokenStream<'a>,
    temp: String,
}

impl<'a> TokenStream for ICUNormalizer2TokenStream<'a> {
    fn advance(&mut self) -> bool {
        let result = self.tail.advance();
        if !result {
            return false;
        }

        if let Ok(t) = self.normalizer.normalize(&self.tail.token().text) {
            self.temp = t;
            mem::swap(&mut self.tail.token_mut().text, &mut self.temp);
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

#[derive(Clone, Copy, Debug)]
pub struct ICUNormalizer2TokenFilter {
    pub tp: Type,
}

impl TokenFilter for ICUNormalizer2TokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(ICUNormalizer2TokenStream {
            normalizer: UNormalizer::from(self.tp),
            tail: token_stream,
            temp: String::with_capacity(100),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, tp: Type) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(ICUNormalizer2TokenFilter { tp })
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    fn token_stream_helper_raw(text: &str, tp: Type) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(RawTokenizer)
            .filter(ICUNormalizer2TokenFilter { tp })
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_default() {
        let tokens = token_stream_helper("This is a test", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "this".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 1,
                text: "is".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 9,
                position: 2,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 14,
                position: 3,
                text: "test".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("Ruß", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "russ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("ΜΆΪΟΣ", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "μάϊοσ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("Μάϊος", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "μάϊοσ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("𐐖", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "𐐾".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("ﴳﴺﰧ", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 9,
            position: 0,
            text: "طمطمطم".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("क्‍ष", Type::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 12,
            position: 0,
            text: "क्ष".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn test_alternate() -> Result<(), Box<dyn Error>> {
        let v = char::from_u32(0x00E9).unwrap().to_string();
        let tokens = token_stream_helper(&v, Type::NFD);

        let v1 = char::from_u32(0x0065).unwrap().to_string();
        let v2 = char::from_u32(0x0301).unwrap().to_string();
        let v = format!("{}{}", v1, v2);

        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 2,
            position: 0,
            text: v,
            position_length: 1,
        }];

        assert_eq!(expected, tokens);

        Ok(())
    }
    #[test]
    pub fn test_empty() {
        let tokens = token_stream_helper_raw("", Type::NFKCCasefold);

        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 0,
            position: 0,
            text: "".to_string(),
            position_length: 1,
        }];

        assert_eq!(expected, tokens);
    }
}
