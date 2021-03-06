//! This module provides a tokenizer that use the same rules to break string into words.
//!
use rust_icu_ubrk::UBreakIterator;
use std::str::Chars;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

/// Default rules, copy from Lucene's binary rules
const DEFAULT_RULES: &str = include_str!("breaking_rules/Default.rbbi");

/// Myanmar rules, copy from Lucene's binary rules
/*const MYANMAR_SYLLABLE_RULES: &str = std::include_str!("breaking_rules/MyanmarSyllable.rbbi");*/

struct ICUBreakingWord<'a> {
    text: Chars<'a>,
    default_breaking_iterator: UBreakIterator,
}

impl<'a> From<&'a str> for ICUBreakingWord<'a> {
    fn from(text: &'a str) -> Self {
        ICUBreakingWord {
            text: text.chars(),
            default_breaking_iterator: UBreakIterator::try_new_rules(DEFAULT_RULES, text)
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

struct ICUTokenizerTokenStream<'a> {
    breaking_word: ICUBreakingWord<'a>,
    token: Token,
}

impl<'a> ICUTokenizerTokenStream<'a> {
    fn new(text: &'a str) -> Self {
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

/// ICU [Tokenizer]. It does not (yet ?) work as Lucene's counterpart.
/// Getting a tokenizer is simple :
/// ```rust
/// use tantivy_analysis_contrib::icu::ICUTokenizer;
///
/// let tokenizer = ICUTokenizer;
/// ```
///
/// # Example
///
/// Here is an example of a tokenization result
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::ICUTokenizer;
///
/// let mut token_stream = TextAnalyzer::from(ICUTokenizer).token_stream("?????????????????? ???????????? ??????????????? ");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "????????????".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "???????????????".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
#[derive(Clone, Copy, Debug)]
pub struct ICUTokenizer;

impl Tokenizer for ICUTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        BoxTokenStream::from(ICUTokenizerTokenStream::new(text))
    }
}

#[cfg(test)]
mod tests {
    /// Same tests as Lucene ICU tokenizer might be enough
    use super::*;

    impl<'a> Iterator for ICUTokenizerTokenStream<'a> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item> {
            if self.advance() {
                return Some(self.token.clone());
            }

            None
        }
    }

    #[test]
    fn test_huge_doc() {
        let mut huge_doc = " ".repeat(4094);
        huge_doc.push_str("testing 1234");
        let tokenizer = &mut ICUTokenizerTokenStream::new(huge_doc.as_str());
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 4094,
                offset_to: 4101,
                position: 0,
                text: "testing".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4102,
                offset_to: 4106,
                position: 1,
                text: "1234".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_armenian() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????????????????? 13 ???????????? ???????????????????? (4,600` ?????????????? ??????????????????????????) ?????????? ???? ?????????????????????? ???????????? ???? ?????????????? ?????????? ???????????????????? ?????????? ?? ???????????????? ???????????? ???????? ???? ?????????? ?? ?????????? ?????????????????????? ????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 11,
                position: 0,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 14,
                position: 1,
                text: "13".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 21,
                position: 2,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 32,
                position: 3,
                text: "????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 39,
                position: 4,
                text: "4,600".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 41,
                offset_to: 48,
                position: 5,
                text: "??????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 49,
                offset_to: 62,
                position: 6,
                text: "??????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 64,
                offset_to: 69,
                position: 7,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 70,
                offset_to: 72,
                position: 8,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 73,
                offset_to: 84,
                position: 9,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 85,
                offset_to: 91,
                position: 10,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 92,
                offset_to: 94,
                position: 11,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 95,
                offset_to: 102,
                position: 12,
                text: "??????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 103,
                offset_to: 108,
                position: 13,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 109,
                offset_to: 119,
                position: 14,
                text: "????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 120,
                offset_to: 125,
                position: 15,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 126,
                offset_to: 127,
                position: 16,
                text: "??".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 128,
                offset_to: 136,
                position: 17,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 137,
                offset_to: 143,
                position: 18,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 144,
                offset_to: 148,
                position: 19,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 149,
                offset_to: 151,
                position: 20,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 152,
                offset_to: 157,
                position: 21,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 158,
                offset_to: 159,
                position: 22,
                text: "??".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 160,
                offset_to: 165,
                position: 23,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 166,
                offset_to: 177,
                position: 24,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 178,
                offset_to: 183,
                position: 25,
                text: "??????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_amharic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "??????????????? ????????? ?????? ????????? ???????????? ?????????????????? ?????? ???????????? ???????????? (???????????????????????????) ????????? ???????????????",
        );
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "???????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 16,
                position: 3,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 4,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 28,
                position: 5,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 6,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 36,
                position: 7,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 41,
                position: 8,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 52,
                position: 9,
                text: "???????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 54,
                offset_to: 56,
                position: 10,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 63,
                position: 11,
                text: "???????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_arabic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????? ???????????????? ?????????? ???? ?????????????????? ???????? \"?????????????? ????????????????: ?????? ??????????????????\" (??????????????????????: Truth in Numbers: The Wikipedia Story)?? ???????? ???????????? ???? 2008.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 7,
                offset_to: 15,
                position: 1,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 21,
                position: 2,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 24,
                position: 3,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 34,
                position: 4,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 35,
                offset_to: 39,
                position: 5,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 41,
                offset_to: 48,
                position: 6,
                text: "??????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 49,
                offset_to: 57,
                position: 7,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 59,
                offset_to: 62,
                position: 8,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 63,
                offset_to: 72,
                position: 9,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 75,
                offset_to: 86,
                position: 10,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 88,
                offset_to: 93,
                position: 11,
                text: "Truth".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 94,
                offset_to: 96,
                position: 12,
                text: "in".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 97,
                offset_to: 104,
                position: 13,
                text: "Numbers".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 106,
                offset_to: 109,
                position: 14,
                text: "The".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 110,
                offset_to: 119,
                position: 15,
                text: "Wikipedia".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 120,
                offset_to: 125,
                position: 16,
                text: "Story".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 128,
                offset_to: 132,
                position: 17,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 133,
                offset_to: 139,
                position: 18,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 140,
                offset_to: 142,
                position: 19,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 143,
                offset_to: 147,
                position: 20,
                text: "2008".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_aramaic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????????? (????????????: Wikipedia) ???? ?????????????????????? ?????????? ?????????????? ???????????? ?????????????? ?????? ?????? ???? ?????????? ??\"????????\" ??\"??????????????????????\"??");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 16,
                position: 1,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 18,
                offset_to: 27,
                position: 2,
                text: "Wikipedia".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 3,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 43,
                position: 4,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 44,
                offset_to: 49,
                position: 5,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 50,
                offset_to: 57,
                position: 6,
                text: "??????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 64,
                position: 7,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 65,
                offset_to: 71,
                position: 8,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 73,
                offset_to: 76,
                position: 9,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 77,
                offset_to: 80,
                position: 10,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 83,
                position: 11,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 84,
                offset_to: 89,
                position: 12,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 90,
                offset_to: 91,
                position: 13,
                text: "??".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 92,
                offset_to: 96,
                position: 14,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 98,
                offset_to: 99,
                position: 15,
                text: "??".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 100,
                offset_to: 111,
                position: 16,
                text: "??????????????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bengali() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????? ???????????????????????? ???????????????????????? ????????? ????????????????????????????????? ??????????????????????????? (???????????? ????????????????????? ??????????????????)??? ???????????????????????????????????? ???????????? ?????? ???????????????????????????, ???????????? ??????????????? ????????? ????????????????????? ????????????????????? ???????????? ?????????????????? ????????????????????????????????? ?????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 11,
                position: 1,
                text: "????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 20,
                position: 2,
                text: "????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 21,
                offset_to: 24,
                position: 3,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 36,
                position: 4,
                text: "?????????????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 46,
                position: 5,
                text: "???????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 48,
                offset_to: 52,
                position: 6,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 60,
                position: 7,
                text: "?????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 61,
                offset_to: 67,
                position: 8,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 70,
                offset_to: 82,
                position: 9,
                text: "????????????????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 83,
                offset_to: 87,
                position: 10,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 88,
                offset_to: 90,
                position: 11,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 91,
                offset_to: 100,
                position: 12,
                text: "???????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 102,
                offset_to: 106,
                position: 13,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 107,
                offset_to: 111,
                position: 14,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 113,
                offset_to: 116,
                position: 15,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 117,
                offset_to: 124,
                position: 16,
                text: "?????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 125,
                offset_to: 132,
                position: 17,
                text: "?????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 133,
                offset_to: 137,
                position: 18,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 138,
                offset_to: 144,
                position: 19,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 145,
                offset_to: 156,
                position: 20,
                text: "?????????????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 157,
                offset_to: 163,
                position: 21,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_farsi() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????? ?????????? ?????????????? ???? ?????????? ???? ???? ???????? ???? ???????? ?????????? ???????? ?????????????????? ?????????? ???????????? ?????????? ????.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 10,
                position: 1,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 18,
                position: 2,
                text: "??????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 19,
                offset_to: 21,
                position: 3,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 27,
                position: 4,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 28,
                offset_to: 30,
                position: 5,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 31,
                offset_to: 33,
                position: 6,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 38,
                position: 7,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 39,
                offset_to: 41,
                position: 8,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 42,
                offset_to: 46,
                position: 9,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 47,
                offset_to: 52,
                position: 10,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 57,
                position: 11,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 67,
                position: 12,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 68,
                offset_to: 73,
                position: 13,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 74,
                offset_to: 80,
                position: 14,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 86,
                position: 15,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 87,
                offset_to: 89,
                position: 16,
                text: "????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_greek() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????????? ???? ???????????????????? ?????? ?????????????????? ???? ???? ?????????????????? wiki, ???????? ?????? ???????????????? ?????? ?????????? ???????????? ???? ???????????????????? ?? ???? ???????????????? ?????? ?????? ????????????.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 11,
                position: 1,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 22,
                position: 2,
                text: "????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 26,
                position: 3,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 27,
                offset_to: 36,
                position: 4,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 39,
                position: 5,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 42,
                position: 6,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 52,
                position: 7,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 57,
                position: 8,
                text: "wiki".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 59,
                offset_to: 63,
                position: 9,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 64,
                offset_to: 67,
                position: 10,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 68,
                offset_to: 76,
                position: 11,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 77,
                offset_to: 80,
                position: 12,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 86,
                position: 13,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 87,
                offset_to: 93,
                position: 14,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 94,
                offset_to: 96,
                position: 15,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 97,
                offset_to: 107,
                position: 16,
                text: "????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 108,
                offset_to: 109,
                position: 17,
                text: "??".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 110,
                offset_to: 112,
                position: 18,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 113,
                offset_to: 121,
                position: 19,
                text: "????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 122,
                offset_to: 125,
                position: 20,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 126,
                offset_to: 129,
                position: 21,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 130,
                offset_to: 136,
                position: 22,
                text: "????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_khmer() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????????????????????????????????????????????????????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 13,
                position: 1,
                text: "???????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 15,
                position: 2,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 18,
                position: 3,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 18,
                offset_to: 22,
                position: 4,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 25,
                position: 5,
                text: "?????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lao() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "?????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "?????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_myanmar() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????????????????????????????????????????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "??????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 15,
                position: 1,
                text: "???????????????????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 17,
                position: 2,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 3,
                text: "????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_thai() {
        let tokenizer =
            &mut ICUTokenizerTokenStream::new("???????????????????????????????????????????????????????????????????????????. ??????????????????????????????????????????? ????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 6,
                position: 1,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 2,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 13,
                position: 3,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 17,
                position: 4,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 20,
                position: 5,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 23,
                position: 6,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 25,
                position: 7,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 27,
                offset_to: 31,
                position: 8,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 31,
                offset_to: 34,
                position: 9,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 36,
                position: 10,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 36,
                offset_to: 38,
                position: 11,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 38,
                offset_to: 41,
                position: 12,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 47,
                position: 13,
                text: "????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tibetan() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "??????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????? ???",
        );
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 1,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 15,
                position: 3,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 20,
                position: 4,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 21,
                offset_to: 24,
                position: 5,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 28,
                position: 6,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 7,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 35,
                position: 8,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 36,
                offset_to: 39,
                position: 9,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 44,
                position: 10,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 45,
                offset_to: 47,
                position: 11,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 48,
                offset_to: 52,
                position: 12,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 55,
                position: 13,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 56,
                offset_to: 57,
                position: 14,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 60,
                position: 15,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 61,
                offset_to: 64,
                position: 16,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 65,
                offset_to: 68,
                position: 17,
                text: "?????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 69,
                offset_to: 73,
                position: 18,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 74,
                offset_to: 76,
                position: 19,
                text: "??????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chinese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????????????? ???????????? ??????????????? ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 2,
                position: 1,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 2,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 3,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 5,
                position: 4,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 7,
                offset_to: 11,
                position: 5,
                text: "????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 17,
                position: 6,
                text: "???????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hebrew() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????? ?????? ???? ??????\"??");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "??????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "??????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 18,
                position: 3,
                text: "??????\"??".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("???????? ???? ???? ????????'??");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 1,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 10,
                position: 2,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 17,
                position: 3,
                text: "????????'??".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty() {
        let expected: Vec<Token> = vec![];

        let tokenizer = &mut ICUTokenizerTokenStream::new("");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(".");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(" ");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lucene1545() {
        /*
         * Standard analyzer does not correctly tokenize combining character U+0364 COMBINING LATIN SMALL LETTRE E.
         * The word "mo??chte" is incorrectly tokenized into "mo" "chte", the combining character is lost.
         * Expected result is only on token "mo??chte".
         */
        let tokenizer = &mut ICUTokenizerTokenStream::new("mo??chte");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 7,
            position: 0,
            text: "mo??chte".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphanumeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("B2B");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "B2B".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("2B");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 2,
            position: 0,
            text: "2B".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_delimiters_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("some-dashed-phrase");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "some".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 11,
                position: 1,
                text: "dashed".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 18,
                position: 2,
                text: "phrase".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("dogs,chase,cats");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "dogs".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 10,
                position: 1,
                text: "chase".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 15,
                position: 2,
                text: "cats".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("ac/dc");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "ac".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "dc".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apostrophes_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 8,
            position: 0,
            text: "O'Reilly".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("you're");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 6,
            position: 0,
            text: "you're".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("she's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "she's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("Jim's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "Jim's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("don't");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "don't".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "O'Reilly's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("21.35");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "21.35".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("R2D2 C3PO");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "R2D2".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 1,
                text: "C3PO".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("216.239.63.104");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 14,
            position: 0,
            text: "216.239.63.104".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_text_with_numbers_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("David has 5000 bones");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "David".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "has".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 14,
                position: 2,
                text: "5000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 20,
                position: 3,
                text: "bones".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_various_text_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("C embedded developers wanted");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "C".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 10,
                position: 1,
                text: "embedded".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 21,
                position: 2,
                text: "developers".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 28,
                position: 3,
                text: "wanted".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo bar FOO BAR");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "foo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bar".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "FOO".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 15,
                position: 3,
                text: "BAR".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo      bar .  FOO <> BAR");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "foo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 12,
                position: 1,
                text: "bar".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 19,
                position: 2,
                text: "FOO".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 26,
                position: 3,
                text: "BAR".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("\"QUOTED\" word");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 1,
                offset_to: 7,
                position: 0,
                text: "QUOTED".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 13,
                position: 1,
                text: "word".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_korean_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("??????????????? ???????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "???????????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "???????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_offsets() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("David has 5000 bones");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "David".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "has".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 14,
                position: 2,
                text: "5000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 20,
                position: 3,
                text: "bones".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_korean() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "????????????".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_japanese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???????????? ????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 2,
                position: 1,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 2,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 3,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 4,
                text: "????????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("???? ????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 2,
                text: "????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 8,
            position: 0,
            text: "????????????????????".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence_with_modifier() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("?????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 7,
            position: 0,
            text: "?????????????????".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_regional_indicator() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "????????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 8,
                position: 1,
                text: "????????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_variation_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("#??????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "3??????".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tag_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("????????????????????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 14,
            position: 0,
            text: "????????????????????????????".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tokenization() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("poo????poo");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "poo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 8,
                position: 2,
                text: "poo".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("??????????????");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "????".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 1,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 2,
                text: "???".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 6,
                position: 3,
                text: "????".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }
}
