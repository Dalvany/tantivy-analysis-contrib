use std::iter::Skip;
use std::str::Split;

use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

const DEFAULT_SEPARATOR: char = '/';

#[derive(Debug)]
struct PathTokenStream<'a> {
    text: Skip<Split<'a, char>>,
    buffer: String,
    token: Token,
    separator: char,
    offset: usize,
    starts_with: bool,
}

impl<'a> TokenStream for PathTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if let Some(part) = self.text.next() {
            if self.starts_with {
                self.buffer.push(self.separator);
            } else {
                self.starts_with = true;
            }
            self.buffer.push_str(part);
            self.token = Token {
                offset_from: self.offset,
                offset_to: self.offset + self.buffer.len(),
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
#[derive(Clone, Copy, Debug)]
pub struct PathTokenizer {
    /// Number of part to skip.
    skip: usize,
    /// Delimiter of path parts
    /// In the following exemple, delimiter is the `/` character :
    /// ```norust
    /// /part1/part2/part3
    /// ```
    delimiter: char,
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
    replacement: Option<char>,
}

impl Default for PathTokenizer {
    /// Construct a [PathTokenizer] with no skip and
    /// `/` as delimiter and replacement.
    fn default() -> Self {
        PathTokenizer {
            skip: 0,
            delimiter: DEFAULT_SEPARATOR,
            replacement: None,
        }
    }
}

impl From<char> for PathTokenizer {
    fn from(delimiter: char) -> Self {
        PathTokenizer {
            skip: 0,
            delimiter,
            replacement: None,
        }
    }
}

impl From<usize> for PathTokenizer {
    fn from(skip: usize) -> Self {
        PathTokenizer {
            skip,
            delimiter: DEFAULT_SEPARATOR,
            replacement: None,
        }
    }
}

impl From<(char, char)> for PathTokenizer {
    fn from((delimiter, replacement): (char, char)) -> Self {
        PathTokenizer {
            skip: 0,
            delimiter,
            replacement: Some(replacement),
        }
    }
}

impl From<(usize, char, char)> for PathTokenizer {
    fn from((skip, delimiter, replacement): (usize, char, char)) -> Self {
        PathTokenizer {
            skip,
            delimiter,
            replacement: Some(replacement),
        }
    }
}

impl From<(usize, char)> for PathTokenizer {
    fn from((skip, delimiter): (usize, char)) -> Self {
        PathTokenizer {
            skip,
            delimiter,
            replacement: None,
        }
    }
}

impl Tokenizer for PathTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        let mut offset = 0;
        let mut starts_with = text.starts_with(self.delimiter);
        let split = text.split(self.delimiter);
        let mut split = if starts_with {
            split.skip(1)
        } else {
            split.skip(0)
        };
        let mut i = self.skip;
        while i > 0 {
            if let Some(token) = split.next() {
                if starts_with {
                    offset = offset + 1;
                } else {
                    starts_with = true;
                }
                offset = offset + token.len();
            }
            i = i - 1;
        }

        BoxTokenStream::from(PathTokenStream {
            text: split,
            buffer: String::with_capacity(text.len()),
            token: Default::default(),
            separator: self.replacement.unwrap_or(self.delimiter),
            offset,
            starts_with,
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
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_end_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_start_of_char() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("a/b/c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("a/b/c/", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_only_delimiter() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("/", tokenizer);
        let expect: Vec<Token> = vec![Token {
            offset_from: 0,
            offset_to: 1,
            position: 0,
            text: "/".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expect);
    }

    #[test]
    fn test_only_delimiters() {
        let tokenizer = PathTokenizer::default();

        let result = tokenize_all("//", tokenizer);
        let expect: Vec<Token> = vec![
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
        assert_eq!(result, expect);
    }

    #[test]
    fn test_replace() {
        let tokenizer = PathTokenizer::from(('/', '\\'));
        let result = tokenize_all("/a/b/c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_windows_path() {
        let tokenizer = PathTokenizer::from('\\');
        let result = tokenize_all("c:\\a\\b\\c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_normalize_win_delim_to_linux_delim() {
        let tokenizer = PathTokenizer::from(('\\', '/'));
        let result = tokenize_all("c:\\a\\b\\c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_basic_skip() {
        let tokenizer = PathTokenizer::from(1);

        let result = tokenize_all("/a/b/c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_end_of_delimiter_skip() {
        let tokenizer = PathTokenizer::from(1);

        let result = tokenize_all("/a/b/c/", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_start_of_char_skip() {
        let tokenizer = PathTokenizer::from(1);

        let result = tokenize_all("a/b/c", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_start_of_char_end_of_delimiter_skip() {
        let tokenizer = PathTokenizer::from(1);

        let result = tokenize_all("a/b/c/", tokenizer);
        let expect: Vec<Token> = vec![
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

        assert_eq!(result, expect);
    }

    #[test]
    fn test_only_delimiter_skip() {
        let tokenizer = PathTokenizer::from(1);

        let result = tokenize_all("/", tokenizer);
        let expect: Vec<Token> = Vec::new();

        assert_eq!(result, expect);
    }
}
