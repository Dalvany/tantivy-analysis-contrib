use std::sync::Arc;

use rustc_hash::FxHashSet;
use tantivy::tokenizer::{TokenFilter, Tokenizer};

use super::ElisionFilterWrapper;

/// A token filter that removes elision from a token.
/// For example, the token `l'avion` will
/// become `avion`.
/// ```rust
/// use tantivy_analysis_contrib::commons::ElisionTokenFilter;
///
/// let filter = ElisionTokenFilter::from_iter_str(vec!["l", "m", "t", "qu", "n", "s", "j", "d", "c", "jusqu", "quoiqu", "lorsqu", "puisqu"], true);
/// ```
///
/// # Example
///
/// This example shows produced token by [ElisionTokenFilter].
///
/// All starting `l'` and `m'` are removed from tokens whatever the case is.
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::ElisionTokenFilter;
///
/// let mut tmp = TextAnalyzer::builder(WhitespaceTokenizer::default())
///    .filter(ElisionTokenFilter::from_iter_str(vec!["L", "M"], true))
///    .build();
/// let mut token_stream = tmp.token_stream("Plop, juste pour voir l'embrouille avec O'brian. m'enfin.");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Plop,".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "juste".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "pour".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "voir".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "embrouille".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "avec".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "O'brian.".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "enfin.".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElisionTokenFilter {
    /// Set of elisions
    pub elisions: FxHashSet<String>,
    /// Indicates that elisions are case-insensitive
    pub ignore_case: bool,
}

impl ElisionTokenFilter {
    /// Construct a new [ElisionTokenFilter] from an iterator over [String] and a [bool].
    /// # Parameters :
    /// * `elisions`: list of elision to remove from tokens
    /// * `ignore_case`: indicate that elisions are case-insensitive
    pub fn from_iter_string(elisions: impl IntoIterator<Item = String>, ignore_case: bool) -> Self {
        let elisions: FxHashSet<String> = elisions
            .into_iter()
            .map(|v| if ignore_case { v.to_lowercase() } else { v })
            .collect();
        Self {
            elisions,
            ignore_case,
        }
    }

    /// Construct a new [ElisionTokenFilter] from an iterator over [str] and a [bool].
    /// # Parameters :
    /// * `elisions`: list of elision to remove from tokens
    /// * `ignore_case`: indicate that elisions are case-insensitive
    pub fn from_iter_str<'a>(
        elisions: impl IntoIterator<Item = &'a str>,
        ignore_case: bool,
    ) -> Self {
        let elisions: FxHashSet<String> = elisions
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
    type Tokenizer<T: Tokenizer> = ElisionFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        ElisionFilterWrapper::new(token_stream, Arc::new(self.elisions), self.ignore_case)
    }
}
