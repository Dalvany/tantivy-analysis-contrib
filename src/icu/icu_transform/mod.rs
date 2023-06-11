use rust_icu_sys as sys;
pub use token_filter::ICUTransformTokenFilter;
use token_stream::ICUTransformTokenStream;
use wrapper::ICUTransformFilterWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

/// Direction
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    /// Forward
    Forward,
    /// Reverse
    Reverse,
}

impl From<Direction> for sys::UTransDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Forward => sys::UTransDirection::UTRANS_FORWARD,
            Direction::Reverse => sys::UTransDirection::UTRANS_REVERSE,
        }
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};

    use super::*;

    fn token_stream_helper(
        text: &str,
        compound_id: &str,
        rules: Option<String>,
        direction: Direction,
    ) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(RawTokenizer::default())
            .filter(
                ICUTransformTokenFilter::new(compound_id.to_string(), rules, direction).unwrap(),
            )
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
    fn test_custom_functionality() {
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
    fn test_custom_functionality_2() {
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
    fn test_empty() {
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

    #[test]
    fn test_example_from_doc() {
        let tokens = token_stream_helper(
            "中国",
            "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC",
            None,
            Direction::Forward,
        );
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 6,
            position: 0,
            text: "zhong guo".to_string(),
            position_length: 1,
        }];
        assert_eq!(tokens, expected);
    }
}
