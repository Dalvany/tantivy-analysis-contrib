use std::iter::{Rev, Skip};
use std::str::Split;

use either::Either;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

const DEFAULT_SEPARATOR: char = '/';

#[derive(Debug)]
struct PathTokenStream<'a> {
    text: Skip<Either<Split<'a, char>, Rev<Split<'a, char>>>>,
    buffer: String,
    token: Token,
    separator: char,
    offset: usize,
    starts_with: bool,
    reverse: bool,
}

impl<'a> TokenStream for PathTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if let Some(part) = self.text.next() {
            if !self.starts_with {
                // Do not add the separator (or replacement) if doesn't start (or end) with separator
                self.starts_with = true;
            } else if self.reverse {
                self.buffer.insert(0, self.separator);
            } else {
                self.buffer.push(self.separator);
            }

            if self.reverse {
                self.buffer.insert_str(0, part);
            } else {
                self.buffer.push_str(part);
            }

            let offset_from = if self.reverse {
                self.offset - self.buffer.len()
            } else {
                self.offset
            };

            let offset_to = if self.reverse {
                self.offset
            } else {
                self.offset + self.buffer.len()
            };

            self.token = Token {
                offset_from,
                offset_to,
                position: 0,
                text: self.buffer.clone(),
                position_length: 1,
            };
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

/// Path tokenizer. It will tokenize this :
/// ```norust
/// /part1/part2/part3
/// ```
/// into
/// ```norust
/// /part1
/// /part1/part2
/// /part1/part2/part3
/// ```
///
/// Enabling `reverse` will make this tokenizer to behave like Lucene's
/// [ReversePathHierarchyTokenizer](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/path/ReversePathHierarchyTokenizer.html)
///
/// # Warning
/// To construct a new [PathTokenizer] you should use the [PathTokenizerBuilder] or the [Default] implementation as
/// [From] trait will probably be removed.
#[derive(Clone, Copy, Debug, Builder)]
#[builder(setter(into), default)]
pub struct PathTokenizer {
    /// Do the tokenization backward.
    /// ```norust
    /// mail.google.com
    /// ```
    /// into
    /// ```norust
    /// com
    /// google.com
    /// mail.google.com
    /// ```
    #[builder(default = "false")]
    pub reverse: bool,
    /// Number of part to skip.
    #[builder(default = "0")]
    pub skip: usize,
    /// Delimiter of path parts
    /// In the following exemple, delimiter is the `/` character :
    /// ```norust
    /// /part1/part2/part3
    /// ```
    #[builder(default = "DEFAULT_SEPARATOR")]
    pub delimiter: char,
    /// Character that replace delimiter for generated parts.
    /// If [None] then the same char as delimiter will be used.
    /// For exemple if delimiter is `/` and replacement is `|`
    /// ```norust
    /// /part1/part2/part3
    /// ```
    /// will generate
    /// ```norust
    /// |part1
    /// |part1|part2
    /// |part1|part2|part3
    /// ```
    pub replacement: Option<char>,
}

impl Default for PathTokenizer {
    /// Construct a [PathTokenizer] with no skip and
    /// `/` as delimiter and replacement.
    fn default() -> Self {
        PathTokenizer {
            reverse: false,
            skip: 0,
            delimiter: DEFAULT_SEPARATOR,
            replacement: None,
        }
    }
}

// TODO remove
impl From<char> for PathTokenizer {
    fn from(delimiter: char) -> Self {
        PathTokenizer {
            reverse: false,
            skip: 0,
            delimiter,
            replacement: None,
        }
    }
}

// TODO remove
impl From<usize> for PathTokenizer {
    fn from(skip: usize) -> Self {
        PathTokenizer {
            reverse: false,
            skip,
            delimiter: DEFAULT_SEPARATOR,
            replacement: None,
        }
    }
}

// TODO remove
impl From<(char, char)> for PathTokenizer {
    fn from((delimiter, replacement): (char, char)) -> Self {
        PathTokenizer {
            reverse: false,
            skip: 0,
            delimiter,
            replacement: Some(replacement),
        }
    }
}

// TODO remove
impl From<(usize, char, char)> for PathTokenizer {
    fn from((skip, delimiter, replacement): (usize, char, char)) -> Self {
        PathTokenizer {
            reverse: false,
            skip,
            delimiter,
            replacement: Some(replacement),
        }
    }
}

// TODO remove
impl From<(usize, char)> for PathTokenizer {
    fn from((skip, delimiter): (usize, char)) -> Self {
        PathTokenizer {
            reverse: false,
            skip,
            delimiter,
            replacement: None,
        }
    }
}

impl Tokenizer for PathTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        let mut offset = 0;
        let mut starts_with = if self.reverse {
            text.ends_with(self.delimiter)
        } else {
            text.starts_with(self.delimiter)
        };
        let split = text.split(self.delimiter);
        let split: Either<Split<char>, Rev<Split<char>>> = if self.reverse {
            Either::Right(split.rev())
        } else {
            Either::Left(split)
        };

        let mut split = if starts_with {
            split.skip(1)
        } else {
            split.skip(0)
        };
        let mut i = self.skip;
        while i > 0 {
            if let Some(token) = split.next() {
                if starts_with {
                    offset += 1;
                } else {
                    starts_with = true;
                }
                offset += token.len();
            }
            i -= 1;
        }

        if self.reverse {
            offset = text.len() - offset;
        }

        BoxTokenStream::from(PathTokenStream {
            text: split,
            buffer: String::with_capacity(text.len()),
            token: Default::default(),
            separator: self.replacement.unwrap_or(self.delimiter),
            offset,
            starts_with,
            reverse: self.reverse,
        })
    }
}

#[cfg(test)]
mod tests {
    // Same tests as Lucene except for random string which are not tested here.
    use super::*;

    fn tokenize_all(text: &str, tokenizer: PathTokenizer) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();

        let mut tokenizer = tokenizer.token_stream(text);
        while tokenizer.advance() {
            result.push(tokenizer.token().clone());
        }

        result
    }

    #[test]
    fn test_basic() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("/a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "/a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "/a/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "/a/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_end_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "/a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "/a/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "/a/b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 7,
                position: 0,
                text: "/a/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "a/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "a/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "a/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "a/b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "a/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("/", tokenizer);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 1,
            position: 0,
            text: "/".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiters() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("//", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "//".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace() {
        let tokenizer = PathTokenizerBuilder::default()
            .replacement('\\')
            .build()
            .unwrap();
        let result = tokenize_all("/a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "\\a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "\\a\\b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "\\a\\b\\c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_windows_path() {
        let tokenizer = PathTokenizerBuilder::default()
            .delimiter('\\')
            .build()
            .unwrap();
        let result = tokenize_all("c:\\a\\b\\c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "c:".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "c:\\a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "c:\\a\\b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "c:\\a\\b\\c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_normalize_win_delim_to_linux_delim() {
        let tokenizer = PathTokenizerBuilder::default()
            .replacement('/')
            .delimiter('\\')
            .build()
            .unwrap();
        let result = tokenize_all("c:\\a\\b\\c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "c:".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "c:/a".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "c:/a/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "c:/a/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_basic_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 2,
                offset_to: 4,
                position: 0,
                text: "/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 6,
                position: 0,
                text: "/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_end_of_delimiter_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 2,
                offset_to: 4,
                position: 0,
                text: "/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 6,
                position: 0,
                text: "/b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 7,
                position: 0,
                text: "/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 1,
                offset_to: 3,
                position: 0,
                text: "/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 5,
                position: 0,
                text: "/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 1,
                offset_to: 3,
                position: 0,
                text: "/b".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 5,
                position: 0,
                text: "/b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 6,
                position: 0,
                text: "/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiter_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/", tokenizer);
        let expected: Vec<Token> = Vec::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_basic_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 5,
                offset_to: 6,
                position: 0,
                text: "c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 6,
                position: 0,
                text: "b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 6,
                position: 0,
                text: "a/b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "/a/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_end_of_delimiter_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 0,
                text: "c/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 7,
                position: 0,
                text: "b/c/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 7,
                position: 0,
                text: "a/b/c/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 7,
                position: 0,
                text: "/a/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 4,
                offset_to: 5,
                position: 0,
                text: "c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 5,
                position: 0,
                text: "b/c".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "a/b/c".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 4,
                offset_to: 6,
                position: 0,
                text: "c/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 6,
                position: 0,
                text: "b/c/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "a/b/c/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiter_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("/", tokenizer);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 1,
            position: 0,
            text: "/".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiters_reverse() {
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .build()
            .unwrap();

        let result = tokenize_all("//", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 1,
                offset_to: 2,
                position: 0,
                text: "/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "//".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_end_of_delimiter_reverse_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 0,
                text: "b/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 5,
                position: 0,
                text: "a/b/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "/a/b/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_reverse_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 2,
                offset_to: 4,
                position: 0,
                text: "b/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "a/b/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter_reverse_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 2,
                offset_to: 4,
                position: 0,
                text: "b/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "a/b/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiter_reverse_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/", tokenizer);
        let expected: Vec<Token> = vec![];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_delimiters_reverse_skip() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(1_usize)
            .build()
            .unwrap();

        let result = tokenize_all("//", tokenizer);
        let expected: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 1,
            position: 0,
            text: "/".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_reverse_skip2() {
        #[allow(trivial_numeric_casts)]
        let tokenizer = PathTokenizerBuilder::default()
            .reverse(true)
            .skip(2_usize)
            .build()
            .unwrap();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expected: Vec<Token> = vec![
            Token {
                offset_from: 1,
                offset_to: 3,
                position: 0,
                text: "a/".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "/a/".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);
    }
}
