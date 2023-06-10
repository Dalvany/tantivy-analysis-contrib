use std::num::NonZeroUsize;
use thiserror::Error;
pub use token_filter::EdgeNgramTokenFilter;
use token_stream::EdgeNgramFilterStream;
use wrapper::EdgeNgramFilterWrapper;

mod token_filter;
mod token_stream;
mod wrapper;

/// Edge ngram errors
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Error)]
pub enum EdgeNgramError {
    /// Error raised when the maximum is not [None](None) and
    /// strictly lower than the minimum.
    #[error("Maximum '{max}' must be greater or equals to minimum '{min}' or should be 'None'")]
    MaximumLowerThanMinimum {
        /// Minimum ngram.
        min: NonZeroUsize,
        /// Maximum ngram.
        max: NonZeroUsize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::{TextAnalyzer, Token, WhitespaceTokenizer};

    fn token_stream_helper(
        text: &str,
        min: NonZeroUsize,
        max: Option<NonZeroUsize>,
        keep_original: bool,
    ) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(EdgeNgramTokenFilter::new(min, max, keep_original).unwrap())
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

    #[test]
    fn test_no_limit_keep() {
        let result = token_stream_helper("abcde", NonZeroUsize::new(1).unwrap(), None, true);

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
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abcd".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abcde".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_limit_no_keep() {
        let result = token_stream_helper("abcde", NonZeroUsize::new(1).unwrap(), None, false);

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
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abcd".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "abcde".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }
}
