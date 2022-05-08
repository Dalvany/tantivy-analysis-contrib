use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct LimitTokenCountStream<'a> {
    tail: BoxTokenStream<'a>,
    count: usize,
}

impl<'a> TokenStream for LimitTokenCountStream<'a> {
    fn advance(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }

        if !self.tail.advance() {
            return false;
        }

        self.count = self.count - 1;

        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

/// [TokenFilter] that limit the number of tokens
///
/// ```rust
/// use tantivy_analysis_contrib::commons::LimitTokenCountFilter;
///
/// let filter:LimitTokenCountFilter = LimitTokenCountFilter::new(5);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LimitTokenCountFilter {
    max_tokens: usize,
}

impl LimitTokenCountFilter {
    /// Create a new [LimitTokenCountFilter].
    ///
    /// # Parameters :
    /// * max_tokens : maximum number of tokens that will be index
    pub fn new(max_tokens: usize) -> Self {
        LimitTokenCountFilter { max_tokens }
    }
}

impl TokenFilter for LimitTokenCountFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(LimitTokenCountStream {
            tail: token_stream,
            count: self.max_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

    use super::*;

    fn token_stream_helper(text: &str, max_token: usize) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(LimitTokenCountFilter::new(max_token))
            .token_stream(text);
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
