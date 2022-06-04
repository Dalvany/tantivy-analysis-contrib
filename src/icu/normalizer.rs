use std::mem;

use rust_icu_unorm2::UNormalizer;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

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

impl From<Mode> for UNormalizer {
    fn from(tp: Mode) -> Self {
        match tp {
            Mode::NFC => UNormalizer::new_nfc().expect("Can't create NFC normalizer"),
            Mode::NFD => UNormalizer::new_nfd().expect("Can't create NFD normalizer"),
            Mode::NFKC => UNormalizer::new_nfkc().expect("Can't create NFKC normalizer"),
            Mode::NFKD => UNormalizer::new_nfkd().expect("Can't create NFKD normalizer"),
            Mode::NFKCCasefold => {
                UNormalizer::new_nfkc_casefold().expect("Can't create NFKC casefold normalizer")
            }
        }
    }
}

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

/// [TokenFilter] that converts text into normal form.
/// It supports all [Google's unicode normalization](https://docs.rs/rust_icu_unorm2/2.0.0/rust_icu_unorm2/struct.UNormalizer.html) using [Mode]:
/// * NFC
/// * NFD
/// * NFKC
/// * NFKD
/// * NFKC casefold
///
/// See Wikipedia's [unicode normalization](https://en.wikipedia.org/wiki/Unicode_equivalence#Normalization) or
/// [Unicode documentation](https://www.unicode.org/reports/tr15/) for more information.
///
/// Building an [ICUNormalizer2TokenFilter] is straightforward :
/// ```rust
/// use tantivy_analysis_contrib::icu::ICUNormalizer2TokenFilter;
/// use tantivy_analysis_contrib::icu::Mode;
///
/// let normalizer = ICUNormalizer2TokenFilter {
///     mode: Mode::NFD,
/// };
/// ```
///
/// # Example
///
/// Here is an example showing which tokens are produce
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::{ICUNormalizer2TokenFilter, Mode};
///
/// let mut token_stream = TextAnalyzer::from(RawTokenizer)
///             .filter(ICUNormalizer2TokenFilter { mode: Mode::NFKCCasefold })
///             .token_stream("Ruß");
///
/// let token = token_stream.next().expect("A token should be present.");
///
/// assert_eq!(token.text, "russ".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ICUNormalizer2TokenFilter {
    /// Normalization algorithm.
    pub mode: Mode,
}

impl TokenFilter for ICUNormalizer2TokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(ICUNormalizer2TokenStream {
            normalizer: UNormalizer::from(self.mode),
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

    fn token_stream_helper(text: &str, tp: Mode) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(ICUNormalizer2TokenFilter { mode: tp })
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    fn token_stream_helper_raw(text: &str, tp: Mode) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(RawTokenizer)
            .filter(ICUNormalizer2TokenFilter { mode: tp })
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
