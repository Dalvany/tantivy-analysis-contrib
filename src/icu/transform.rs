use std::error::Error;
use std::mem;

use rust_icu::sys;
use rust_icu::trans as utrans;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Forward,
    Reverse,
}

impl Into<sys::UTransDirection> for Direction {
    fn into(self) -> sys::UTransDirection {
        match self {
            Direction::Forward => sys::UTransDirection::UTRANS_FORWARD,
            Direction::Reverse => sys::UTransDirection::UTRANS_REVERSE,
        }
    }
}

struct ICUTransformTokenStream<'a> {
    transform: utrans::UTransliterator,
    tail: BoxTokenStream<'a>,
    temp: String,
}

impl<'a> TokenStream for ICUTransformTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let result = self.tail.advance();
        if !result {
            return false;
        }
        if let Ok(t) = self.transform.transliterate(&self.tail.token().text) {
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

#[derive(Clone, Debug)]
pub struct ICUTransformTokenFilter {
    pub compound_id: String,
    pub rules: Option<String>,
    pub direction: sys::UTransDirection,
}

impl ICUTransformTokenFilter {
    pub fn new(
        compound_id: String,
        rules: Option<String>,
        direction: Direction,
    ) -> Result<Self, Box<dyn Error>> {
        let _ = utrans::UTransliterator::new(
            compound_id.as_str(),
            rules.as_ref().map(|x| x.as_str()),
            direction.into(),
        )?;
        Ok(ICUTransformTokenFilter {
            compound_id,
            rules,
            direction: direction.into(),
        })
    }
}

impl TokenFilter for ICUTransformTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(ICUTransformTokenStream {
            // unwrap work, we checked in new method.
            transform: utrans::UTransliterator::new(
                self.compound_id.as_str(),
                self.rules.as_ref().map(|x| x.as_str()),
                self.direction,
            )
            .unwrap(),
            tail: token_stream,
            temp: String::with_capacity(100),
        })
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer};

    use super::*;

    fn token_stream_helper(
        text: &str,
        compound_id: &str,
        rules: Option<String>,
        direction: Direction,
    ) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(RawTokenizer)
            .filter(
                ICUTransformTokenFilter::new(String::from(compound_id), rules, direction).unwrap(),
            )
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_basic_functionality() {
        let tokens =
            token_stream_helper("簡化字", "Traditional-Simplified", None, Direction::Forward);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 9,
            position: 0,
            text: "简化字".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);

        let tokens = token_stream_helper("ヒラガナ", "Katakana-Hiragana", None, Direction::Forward);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 12,
            position: 0,
            text: "ひらがな".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);

        let tokens = token_stream_helper(
            "アルアノリウ",
            "Fullwidth-Halfwidth",
            None,
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 18,
            position: 0,
            text: "ｱﾙｱﾉﾘｳ".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);

        let tokens = token_stream_helper(
            "Αλφαβητικός Κατάλογος",
            "Any-Latin",
            None,
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 41,
            position: 0,
            text: "Alphabētikós Katálogos".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);

        let tokens = token_stream_helper(
            "Alphabētikós Katálogos",
            "NFD; [:Nonspacing Mark:] Remove",
            None,
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 25,
            position: 0,
            text: "Alphabetikos Katalogos".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);

        let tokens = token_stream_helper("中国", "Han-Latin", None, Direction::Forward);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 6,
            position: 0,
            text: "zhōng guó".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);
    }

    #[test]
    pub fn test_custom_functionality() {
        let tokens = token_stream_helper(
            "abacadaba",
            "test",
            Some("a > b; b > c;".to_string()),
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 9,
            position: 0,
            text: "bcbcbdbcb".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);
    }

    #[test]
    pub fn test_custom_functionality_2() {
        let tokens = token_stream_helper(
            "caa",
            "test",
            Some("c { a > b; a > d;".to_string()),
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "cbd".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);
    }

    #[test]
    pub fn test_empty() {
        let tokens = token_stream_helper("", "Any-Latin", None, Direction::Forward);

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
