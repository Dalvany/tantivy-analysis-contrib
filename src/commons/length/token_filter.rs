use tantivy_tokenizer_api::{TokenFilter, Tokenizer};

use super::LengthFilterWrapper;

/// This [TokenFilter] filters tokens that don't match a min or a max length (inclusive).
/// ```rust
/// use tantivy_analysis_contrib::commons::LengthTokenFilter;
///
/// let length_token_filter = LengthTokenFilter::new(Some(4), Some(10));
/// ```
///
/// # Example
///
/// In this example, tokens `There`, `1` and `token` are filtered out because they are too short or
/// too long.
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::LengthTokenFilter;
///
/// let mut tmp = TextAnalyzer::builder(WhitespaceTokenizer::default())
///    .filter(LengthTokenFilter::new(Some(2), Some(4)))
///    .build();
/// let mut token_stream = tmp.token_stream("There is 1 token");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "is".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LengthTokenFilter {
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthTokenFilter {
    /// Get a new token filter.
    /// # Parameters :
    /// * min : minimum length a token should have (inclusive)
    /// * max : maximum length a token should have (inclusive)
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        LengthTokenFilter { min, max }
    }
}

impl TokenFilter for LengthTokenFilter {
    type Tokenizer<T: Tokenizer> = LengthFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        LengthFilterWrapper::new(token_stream, self.min, self.max)
    }
}
