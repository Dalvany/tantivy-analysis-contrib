use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

/// Edge ngram errors
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum EdgeNgramError {
    /// Error raised when minimum is set to 0.
    InvalidMinimum,
    /// Error raised when maximum is not [None](Option::None) and
    /// strictly lower than minimum.
    MaximumLowerThanMinimum {
        /// Minimum ngram.
        min: NonZeroUsize,
        /// Maximum ngram.
        max: NonZeroUsize,
    },
}

impl Display for EdgeNgramError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMinimum => write!(f, "Minimum must be greater than 0"),
            Self::MaximumLowerThanMinimum { min, max } => write!(
                f,
                "Maximum '{}' must be greater or equals to minimum '{}' or should be 'None'",
                max, min
            ),
        }
    }
}

impl std::error::Error for EdgeNgramError {}

struct EdgeNgramTokenStreamFilter<'a> {
    tail: BoxTokenStream<'a>,
    /// Current token to emit
    token: Token,
    /// Minimum ngram, must be greater than 0
    min: usize,
    /// Maximum ngram, None means no limit
    max: Option<usize>,
    /// Which ngram we should emit
    count: usize,
    /// Do we have to keep original token
    keep_original_token: bool,
    /// Avoid doing multiple time self.tail.token().chars().count()
    current_len: usize,
    /// Stop at
    stop_length: usize,
}

impl<'a> TokenStream for EdgeNgramTokenStreamFilter<'a> {
    fn advance(&mut self) -> bool {
        loop {
            // if count = min then we begin a new token...
            if self.count == self.min {
                if !self.tail.advance() {
                    return false;
                }

                self.token = self.tail.token().clone();
                // Reset everything with new token
                self.current_len = self.tail.token().text.chars().count();

                // If we have to keep original token but its length
                // is lower than min then we force output it
                // otherwise it won't be emitted.
                if self.keep_original_token && self.current_len < self.min {
                    return true;
                }

                // We stop if we reach the end of the token or max (if present).
                self.stop_length =
                    std::cmp::min(self.max.unwrap_or(self.current_len), self.current_len);
            }

            if self.count <= self.stop_length {
                let token_string: String =
                    self.tail.token().text.chars().take(self.count).collect();
                self.token.text = token_string;

                // We have reach end of token, so we reset count to min
                if self.count == self.stop_length {
                    if self.stop_length == self.current_len
                        || (self.max.is_some() && !self.keep_original_token)
                    {
                        // If we reach the end of token then reset
                        // Or
                        // If we have a max, we have reached it, if we
                        // do not have to keep original token then reset
                        self.count = self.min;
                    } else {
                        self.count += 1;
                    }
                } else {
                    self.count += 1;
                }

                return true;
            } else {
                self.count = self.min;
                if self.keep_original_token {
                    self.token.text = self.tail.token().text.clone();
                    return true;
                }
            }
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

/// Token filter that produce [ngram](https://docs.rs/tantivy/0.18.1/tantivy/tokenizer/struct.NgramTokenizer.html)
/// from the start of the token. For example `Quick` will generate
/// `Q`, `Qu`, `Qui`, `Quic`, ...etc.
///
/// It is configure with two parameters :
/// * min edge-ngram : the number of maximum characters (e.g. with min=3, `Quick`
/// will generate `Qui`, `Quic` and `Quick`). It must be greater than 0.
/// * max edge-ngram : the number of maximum characters (e.g. with max=3, `Quick`
/// will generate `Q`, `Qu` and `Qui`. It is optional and there is no maximum then
/// it will generate up to the end of the token.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::num::NonZeroUsize;
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::EdgeNgramTokenFilter;
///
/// let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
///             .filter(EdgeNgramTokenFilter::new(NonZeroUsize::new(2).unwrap(), NonZeroUsize::new(4), false)?)
///             .token_stream("Quick");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Qu".to_string());
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Qui".to_string());
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Quic".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
///
/// This token filter is useful to do a "starts with" therefor a "search as you type".
///
/// It is also easy to have an efficient "ends with" by adding the [ReverseTokenFilter](tantivy_analysis_contrib::commons::ReverseTokenFilter)
/// before the edge ngram filter.
///
/// # How to use it
///
/// To use it you should have another pipeline at search time that does not include
/// the edge-ngram filter. Otherwise, you'll might get irrelevant results.
/// Please see the [example](https://github.com/Dalvany/tantivy-analysis-contrib/tree/main/examples/edge_ngram.rs)
/// in source repository for a way to do it.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct EdgeNgramTokenFilter {
    min: NonZeroUsize,
    max: Option<NonZeroUsize>,
    keep_original_token: bool,
}

impl EdgeNgramTokenFilter {
    /// Create a new `EdgeNgramTokenFilter` with the min and max ngram
    /// provided.
    ///
    /// # Parameters
    ///
    /// * `min` : minimum edge-ngram.
    /// * `max` : maximum edge-ngram. It must be greater or equals to `min`.
    /// Provide [None](Option::None) for unlimited.
    /// * `keep_original_token` : the complete token will also be output if
    /// the length is greater than `max`.
    pub fn new(
        min: NonZeroUsize,
        max: Option<NonZeroUsize>,
        keep_original_token: bool,
    ) -> Result<Self, EdgeNgramError> {
        // Check max
        if let Some(m) = max {
            if m < min {
                return Err(EdgeNgramError::MaximumLowerThanMinimum { min, max: m });
            }
        }

        Ok(EdgeNgramTokenFilter {
            min,
            max,
            keep_original_token,
        })
    }
}

impl From<NonZeroUsize> for EdgeNgramTokenFilter {
    fn from(ngram: NonZeroUsize) -> Self {
        // This is safe to unwrap since minGram != 0 and maxGram = minGram.
        Self::new(ngram, Some(ngram), false).unwrap()
    }
}

impl TokenFilter for EdgeNgramTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(EdgeNgramTokenStreamFilter {
            tail: token_stream,
            token: Default::default(),
            min: self.min.get(),
            max: self.max.map(|v| v.get()),
            count: self.min.get(),
            keep_original_token: self.keep_original_token,
            current_len: 0,
            stop_length: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

    fn token_stream_helper(
        text: &str,
        min: NonZeroUsize,
        max: Option<NonZeroUsize>,
        keep_original: bool,
    ) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(EdgeNgramTokenFilter::new(min, max, keep_original).unwrap())
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_invalid_input_2() {
        let result =
            EdgeNgramTokenFilter::new(NonZeroUsize::new(2).unwrap(), NonZeroUsize::new(1), false);

        let expected = EdgeNgramError::MaximumLowerThanMinimum {
            min: NonZeroUsize::new(2).unwrap(),
            max: NonZeroUsize::new(1).unwrap(),
        };

        assert_eq!(result, Err(expected));
    }

    #[test]
    fn test_front_unigram() {
        let result = token_stream_helper(
            "abcde",
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(1),
            false,
        );

        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "a".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_oversized_ngram() {
        let result = token_stream_helper(
            "abcde",
            NonZeroUsize::new(6).unwrap(),
            NonZeroUsize::new(6),
            false,
        );

        let expected = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_oversized_ngrams_preserve_original() {
        let result = token_stream_helper(
            "abcde",
            NonZeroUsize::new(6).unwrap(),
            NonZeroUsize::new(6),
            true,
        );

        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "abcde".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preserve_original() {
        // Without preserve
        let result = token_stream_helper(
            "a bcd efghi jk",
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(3),
            false,
        );

        let expected = vec![
            Token {
                offset_from: 2,
                offset_to: 5,
                position: 1,
                text: "bc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 5,
                position: 1,
                text: "bcd".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 2,
                text: "ef".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 2,
                text: "efg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 14,
                position: 3,
                text: "jk".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        // With preserve
        let result = token_stream_helper(
            "a bcd efghi jk",
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(3),
            true,
        );

        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 5,
                position: 1,
                text: "bc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 5,
                position: 1,
                text: "bcd".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 2,
                text: "ef".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 2,
                text: "efg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 2,
                text: "efghi".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 14,
                position: 3,
                text: "jk".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_front_range_of_ngrams() {
        let result = token_stream_helper(
            "abcde",
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(3),
            false,
        );

        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "ab".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abc".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_positions() {
        let result = token_stream_helper(
            "abcde vwxyz",
            NonZeroUsize::new(1).unwrap(),
            NonZeroUsize::new(3),
            false,
        );

        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "ab".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "v".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "vw".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "vwx".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_small_token_in_stream() {
        let result = token_stream_helper(
            "abc de fgh",
            NonZeroUsize::new(3).unwrap(),
            NonZeroUsize::new(3),
            false,
        );

        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "abc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 7,
                offset_to: 10,
                position: 2,
                text: "fgh".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }
}
