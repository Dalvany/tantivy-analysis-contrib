pub use token_filter::LengthTokenFilter;
use token_stream::LengthTokenStream;
use wrapper::LengthFilterWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{TextAnalyzer, Token, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, min: Option<usize>, max: Option<usize>) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LengthTokenFilter::new(min, max))
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
