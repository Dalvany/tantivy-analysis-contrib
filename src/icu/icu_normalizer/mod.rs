use rust_icu_unorm2::UNormalizer;

use super::Error;
pub use token_filter::ICUNormalizer2TokenFilter;
use token_stream::ICUNormalizer2TokenStream;
use wrapper::ICUNormalizer2FilterWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

/// Normalization algorithms (see [Wikipedia](https://en.wikipedia.org/wiki/Unicode_equivalence#Normalization)).
#[derive(Clone, Debug, Copy)]
pub enum Mode {
    /// Normalization Form Canonical Composition.
    NFC,
    /// Normalization Form Canonical Decomposition.
    NFD,
    /// Normalization Form Compatibility Composition.
    NFKC,
    /// Normalization Form Compatibility Decomposition.
    NFKD,
    /// Normalization Form Compatibility Composition with casefolding.
    NFKCCasefold,
}

impl TryFrom<Mode> for UNormalizer {
    type Error = Error;

    fn try_from(tp: Mode) -> Result<Self, Self::Error> {
        let normalizer = match tp {
            Mode::NFC => UNormalizer::new_nfc()?,
            Mode::NFD => UNormalizer::new_nfd()?,
            Mode::NFKC => UNormalizer::new_nfkc()?,
            Mode::NFKD => UNormalizer::new_nfkd()?,
            Mode::NFKCCasefold => UNormalizer::new_nfkc_casefold()?,
        };
        Ok(normalizer)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, tp: Mode) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(ICUNormalizer2TokenFilter::new(tp).unwrap())
            .build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    fn token_stream_helper_raw(text: &str, tp: Mode) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(RawTokenizer::default())
            .filter(ICUNormalizer2TokenFilter::new(tp).unwrap())
            .build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_default() {
        let tokens = token_stream_helper("This is a test", Mode::NFKCCasefold);
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

        let tokens = token_stream_helper("Ruß", Mode::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "russ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("ΜΆΪΟΣ", Mode::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "μάϊοσ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("Μάϊος", Mode::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "μάϊοσ".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("𐐖", Mode::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "𐐾".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("ﴳﴺﰧ", Mode::NFKCCasefold);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 9,
            position: 0,
            text: "طمطمطم".to_string(),
            position_length: 1,
        }];
        assert_eq!(expected, tokens);

        let tokens = token_stream_helper("क्‍ष", Mode::NFKCCasefold);
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
        let tokens = token_stream_helper(&v, Mode::NFD);

        let v1 = char::from_u32(0x0065).unwrap().to_string();
        let v2 = char::from_u32(0x0301).unwrap().to_string();
        let v = format!("{v1}{v2}");

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
        let tokens = token_stream_helper_raw("", Mode::NFKCCasefold);

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
