use std::mem;

use rust_icu_sys as sys;
use rust_icu_utrans as utrans;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

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

/// This [TokenFilter] allow to transform text into another
/// for example to performe transliteration.
/// See [ICU documentation](https://unicode-org.github.io/icu/userguide/transforms/general/)
/// ```rust
/// use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};
/// let token_filter = ICUTransformTokenFilter {
///     compound_id: "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
///     rules: None,
///     direction: Direction::Forward
/// };
/// ```
///
/// # Example
///
/// Here is an example of transform that convert into greek letters into latin letters
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};
///
/// let mut token_stream = TextAnalyzer::from(RawTokenizer)
///             .filter(ICUTransformTokenFilter {
///                 compound_id: "Greek-Latin".to_string(),
///                 rules: None,
///                 direction: Direction::Forward
///             })
///             .token_stream("Αλφαβητικός Κατάλογος");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Alphabētikós Katálogos".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
#[derive(Clone, Debug)]
pub struct ICUTransformTokenFilter {
    /// [Compound transform](https://unicode-org.github.io/icu/userguide/transforms/general/#compound-ids)
    pub compound_id: String,
    /// Custom transform [rules](https://unicode-org.github.io/icu/userguide/transforms/general/rules.html)
    pub rules: Option<String>,
    /// Direction
    pub direction: Direction,
}

impl TokenFilter for ICUTransformTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(ICUTransformTokenStream {
            // unwrap work, we checked in new method.
            transform: utrans::UTransliterator::new(
                self.compound_id.as_str(),
                self.rules.as_deref(),
                self.direction.into(),
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
            .filter(ICUTransformTokenFilter {
                compound_id: compound_id.to_string(),
                rules,
                direction,
            })
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
