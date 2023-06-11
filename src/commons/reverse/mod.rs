pub use token_filter::ReverseTokenFilter;
use token_stream::ReverseTokenStream;
use wrapper::ReverseFilterWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper_whitespace(text: &str) -> Vec<Token> {
        let filter = ReverseTokenFilter;
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(filter)
            .build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    fn token_stream_helper_raw(text: &str) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(RawTokenizer::default())
            .filter(ReverseTokenFilter)
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
    fn test_filter() {
        let result = token_stream_helper_whitespace("Do have a nice day");
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "oD".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 7,
                position: 1,
                text: "evah".to_string(),
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
                text: "ecin".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 18,
                position: 4,
                text: "yad".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_reverse_string() {
        let result = token_stream_helper_raw("A");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 1,
            position: 0,
            text: "A".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let result = token_stream_helper_raw("AB");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 2,
            position: 0,
            text: "BA".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let result = token_stream_helper_raw("ABC");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "CBA".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_reverse_supplementary() {
        let result = token_stream_helper_raw("瀛愯䇹鍟艱𩬅");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 19,
            position: 0,
            text: "𩬅艱鍟䇹愯瀛".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let result = token_stream_helper_raw("瀛愯䇹鍟艱𩬅a");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 20,
            position: 0,
            text: "a𩬅艱鍟䇹愯瀛".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let result = token_stream_helper_raw("𩬅abcdef");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "fedcba𩬅".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_term() {
        let result = token_stream_helper_raw("");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 0,
            position: 0,
            text: "".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }
}
