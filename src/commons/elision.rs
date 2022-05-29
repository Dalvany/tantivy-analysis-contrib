use std::collections::BTreeSet;

use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct ElisionTokenStream<'a> {
    tail: BoxTokenStream<'a>,
    // Use a BTreeSet as this set should be small otherwise use HashSet.
    elisions: BTreeSet<String>,
    ignore_case: bool,
}

impl<'a> TokenStream for ElisionTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        let token = &self.tail.token().text;
        let found: Option<(usize, char)> = token.char_indices().find(|(_, ch)| ch == &'\'');
        if let Some((index, _)) = found {
            let prefix = &self.tail.token().text[0..index];
            let contains = if self.ignore_case {
                self.elisions.contains(&prefix.to_lowercase())
            } else {
                self.elisions.contains(prefix)
            };
            if contains {
                self.tail.token_mut().text = token[index + 1..].to_string();
                self.tail.token_mut().offset_from = self.tail.token_mut().offset_from + index + 1;
            }
        }

        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

/// A token filter that removes elision from a token. For exemple the token `l'avion` will
/// become `avion`.
/// ```rust
/// use tantivy_analysis_contrib::commons::ElisionTokenFilter;
///
/// let filter = ElisionTokenFilter::from_iter_str(vec!["l", "m", "t", "qu", "n", "s", "j", "d", "c", "jusqu", "quoiqu", "lorsqu", "puisqu"], true);
/// ```
#[derive(Clone, Debug)]
pub struct ElisionTokenFilter {
    /// Set of elisions
    pub elisions: BTreeSet<String>,
    /// Indicates that elisions are case-insensitive
    pub ignore_case: bool,
}

impl ElisionTokenFilter {
    /// Construct a new [ElisionTokenFilter] from an iterator over [String] and a [bool].
    /// # Parameters :
    /// * elisions : list of elision to remove from tokens
    /// * ignore_case : indicate that elisions are ignore-case
    pub fn from_iter_string(elisions: impl IntoIterator<Item = String>, ignore_case: bool) -> Self {
        let elisions: BTreeSet<String> = elisions
            .into_iter()
            .map(|v| {
                if ignore_case {
                    v.to_lowercase()
                } else {
                    v
                }
            })
            .collect();
        Self {
            elisions,
            ignore_case,
        }
    }

    /// Construct a new [ElisionTokenFilter] from an iterator over [str] and a [bool].
    /// # Parameters :
    /// * elisions : list of elision to remove from tokens
    /// * ignore_case : indicate that elisions are ignore-case
    pub fn from_iter_str<'a>(
        elisions: impl IntoIterator<Item = &'a str>,
        ignore_case: bool,
    ) -> Self {
        let elisions: BTreeSet<String> = elisions
            .into_iter()
            .map(|v| {
                if ignore_case {
                    v.to_lowercase()
                } else {
                    v.to_string()
                }
            })
            .collect();
        Self {
            elisions,
            ignore_case,
        }
    }
}

impl TokenFilter for ElisionTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        BoxTokenStream::from(ElisionTokenStream {
            tail: token_stream,
            elisions: self.elisions.clone(),
            ignore_case: self.ignore_case,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

    fn tokenize_all(text: &str, elision: Vec<&str>, ignore_case: bool) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(ElisionTokenFilter::from_iter_str(elision, ignore_case))
            .token_stream(text);
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
