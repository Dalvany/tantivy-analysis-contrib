use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct TrimTokenStream<'a> {
    tail: BoxTokenStream<'a>,
    token: Token,
}

impl<'a> TokenStream for TrimTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }

        let text = self.tail.token().text.clone();

        let start_index = text.chars().position(|c| !c.is_whitespace());
        let end_index = text
            .chars()
            .rev()
            .position(|c| !c.is_whitespace())
            .map(|v| text.len() - v);
        let text = text.trim().to_string();

        if start_index.is_none() || end_index.is_none() {}
        match (start_index, end_index) {
            (None, None) | (None, Some(_)) | (Some(_), None) => {
                self.token = Token {
                    offset_from: self.tail.token().offset_from,
                    offset_to: self.tail.token().offset_from,
                    position: self.tail.token().position,
                    text,
                    position_length: self.tail.token().position_length,
                }
            }
            (Some(start), Some(end)) => {
                self.token = Token {
                    offset_from: self.tail.token().offset_from + start,
                    offset_to: self.tail.token().offset_to - end,
                    position: self.tail.token().position,
                    text,
                    position_length: self.tail.token().position_length,
                }
            }
        }

        true
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

/// [TokenFilter] that trims leading and trailing space.
/// ```rust
/// use tantivy_analysis_contrib::commons::TrimTokenFilter;
///
/// let length_token_filter = TrimTokenFilter;
/// ```
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token, RawTokenizer};
/// use tantivy_analysis_contrib::commons::TrimTokenFilter;
///
/// let mut token_stream = TextAnalyzer::from(RawTokenizer)
///             .filter(TrimTokenFilter)
///             .token_stream("\t\n token    \n\n\t\t");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "token".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TrimTokenFilter;

impl TokenFilter for TrimTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(TrimTokenStream {
            tail: token_stream,
            token: Token::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer};

    use super::*;

    fn token_stream_helper(text: &str) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(RawTokenizer)
            .filter(TrimTokenFilter)
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_trim() {
        let result = token_stream_helper(" \ttest\t\t \n  ");
        let expected: Vec<Token> = vec![Token {
            offset_from: 2,
            offset_to: 6,
            position: 0,
            text: "test".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_whitespace() {
        let result = token_stream_helper(" \t\t\t \n  ");
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 0,
            position: 0,
            text: "".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_empty() {
        let result = token_stream_helper("");
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
