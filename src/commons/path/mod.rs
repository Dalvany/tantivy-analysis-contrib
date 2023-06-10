use token_stream::PathTokenStream;
pub use tokenizer::*;

mod token_stream;
mod tokenizer;

const DEFAULT_SEPARATOR: char = '/';

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

    // Same tests as Lucene except for random string which are not tested here.
    use super::*;

    fn tokenize_all(text: &str, mut tokenizer: PathTokenizer) -> Vec<Token> {
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
