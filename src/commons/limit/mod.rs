pub use token_filter::LimitTokenCountFilter;
use token_stream::LimitTokenCountStream;
use wrapper::LimitTokenCountWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{TextAnalyzer, Token, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, max_token: usize) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LimitTokenCountFilter::new(max_token))
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
    fn test_lower() {
        let result = token_stream_helper("This is a text", 5);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "This".to_string(),
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
                text: "text".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_equals() {
        let result = token_stream_helper("This is a text", 4);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "This".to_string(),
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
                text: "text".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_greater() {
        let result = token_stream_helper("This is a text", 3);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "This".to_string(),
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
        ];

        assert_eq!(result, expected);
    }
}
