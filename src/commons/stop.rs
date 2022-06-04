use fst::Set;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct StopTokenStream<'a> {
    tail: BoxTokenStream<'a>,
    stop_words: Set<Vec<u8>>,
}

impl<'a> TokenStream for StopTokenStream<'a> {
    fn advance(&mut self) -> bool {
        while self.tail.advance() {
            if !self.stop_words.contains(&self.tail.token().text) {
                return true;
            }
        }

        false
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

/// This [TokenFilter] remove token. This is useful to filter out
/// articles. It use [fst](https://crates.io/crates/fst) crate.
/// ```rust
/// use fst::Set;
/// use tantivy_analysis_contrib::commons::StopTokenFilter;
///
/// let mut stop_words = vec!["a", "the", "is", "are"];
/// stop_words.sort();
/// let stop_words = Set::from_iter(stop_words).unwrap();
/// let filter = StopTokenFilter {
///     stop_words
/// };
/// ```
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use fst::Set;
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::StopTokenFilter;
///
/// let mut stop_words = vec!["is", "are", "the", "of", "a", "an"];
/// stop_words.sort();
///
/// let stop_token_filter:StopTokenFilter = Set::from_iter(stop_words)?.into();
///
/// let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
///             .filter(stop_token_filter)
///             .token_stream("This is a good test of the english stop analyzer");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "This".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "good".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "test".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "english".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "stop".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "analyzer".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct StopTokenFilter {
    /// List of stop words.
    pub stop_words: Set<Vec<u8>>,
}

impl From<Set<Vec<u8>>> for StopTokenFilter {
    fn from(stop_words: Set<Vec<u8>>) -> Self {
        Self { stop_words }
    }
}

impl TokenFilter for StopTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(StopTokenStream {
            tail: token_stream,
            stop_words: self.stop_words.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

    fn token_stream_helper(text: &str, stop_words: Vec<&str>) -> Vec<Token> {
        let stop_words = Set::from_iter(stop_words).unwrap();
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(StopTokenFilter { stop_words })
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_stop() {
        let mut stop_words = vec!["good", "test", "analyzer"];
        stop_words.sort();
        let result = token_stream_helper(
            "This is a good test of the english stop analyzer",
            stop_words,
        );
        let expected = vec![
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
                offset_from: 20,
                offset_to: 22,
                position: 5,
                text: "of".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 26,
                position: 6,
                text: "the".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 27,
                offset_to: 34,
                position: 7,
                text: "english".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 35,
                offset_to: 39,
                position: 8,
                text: "stop".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(expected, result);
    }
}
