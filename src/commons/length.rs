use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct LengthTokenStream<'a> {
    tail: BoxTokenStream<'a>,
    min: Option<usize>,
    max: Option<usize>,
}

impl<'a> TokenStream for LengthTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let mut result = true;
        let mut length_ok = false;
        while result && !length_ok {
            result = self.tail.advance();
            if result {
                let size = self.tail.token().text.len();
                length_ok =
                    self.min.map_or(true, |v| v <= size) && self.max.map_or(true, |v| size <= v);
            }
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

/// This [TokenFilter] filters tokens that doesn't match a min or a max length.
/// ```rust
/// use tantivy_analysis_contrib::commons::LengthTokenFilter;
///
/// let length_token_filter = LengthTokenFilter::new(Some(4), Some(10));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LengthTokenFilter {
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthTokenFilter {
    /// Get a new token filter.
    /// # Parameters :
    /// * min : minimum length a token should have (inclusive)
    /// * max : maximum length a token should have (inclusive)
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        LengthTokenFilter { min, max }
    }
}

impl TokenFilter for LengthTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(LengthTokenStream {
            tail: token_stream,
            min: self.min,
            max: self.max,
        })
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, min: Option<usize>, max: Option<usize>) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(LengthTokenFilter::new(min, max))
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_min_length_token_greater() {
        let result = token_stream_helper("token", Some(4), None);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_min_length_token_equals() {
        let result = token_stream_helper("token", Some(5), None);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_min_length_token_less() {
        let result = token_stream_helper("token", Some(6), None);
        let expected: Vec<Token> = vec![];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_min_none() {
        let result = token_stream_helper("token", None, None);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_length_token_greater() {
        let result = token_stream_helper("token", None, Some(4));
        let expected: Vec<Token> = vec![];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_length_token_equals() {
        let result = token_stream_helper("token", None, Some(5));
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_length_token_less() {
        let result = token_stream_helper("token", None, Some(6));
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_none() {
        let result = token_stream_helper("token", None, None);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "token".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_length() {
        let result = token_stream_helper(
            "Current text should have 4 tokens right ?",
            Some(4),
            Some(5),
        );
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 8,
                offset_to: 12,
                position: 1,
                text: "text".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 24,
                position: 3,
                text: "have".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 39,
                position: 6,
                text: "right".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }
}
