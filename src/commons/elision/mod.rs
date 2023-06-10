use token_stream::ElisionTokenStream;
use wrapper::ElisionFilterWrapper;
pub use token_filter::ElisionTokenFilter;

mod token_stream;
mod wrapper;
mod token_filter;

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer, Token};

    fn tokenize_all(text: &str, elision: Vec<&str>, ignore_case: bool) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(ElisionTokenFilter::from_iter_str(elision, ignore_case)).build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_elision_ignore_case() {
        let result = tokenize_all(
            "Plop, juste pour voir l'embrouille avec O'brian. m'enfin.",
            vec!["L", "M"],
            true,
        );
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "Plop,".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "juste".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 16,
                position: 2,
                text: "pour".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 3,
                text: "voir".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 24,
                offset_to: 34,
                position: 4,
                text: "embrouille".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 35,
                offset_to: 39,
                position: 5,
                text: "avec".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 48,
                position: 6,
                text: "O'brian.".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 51,
                offset_to: 57,
                position: 7,
                text: "enfin.".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_elision() {
        let result = tokenize_all(
            "Plop, juste pour voir l'embrouille avec O'brian. M'enfin.",
            vec!["l", "M"],
            false,
        );
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "Plop,".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "juste".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 16,
                position: 2,
                text: "pour".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 3,
                text: "voir".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 24,
                offset_to: 34,
                position: 4,
                text: "embrouille".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 35,
                offset_to: 39,
                position: 5,
                text: "avec".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 48,
                position: 6,
                text: "O'brian.".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 51,
                offset_to: 57,
                position: 7,
                text: "enfin.".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }
}
