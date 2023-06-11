use super::LimitTokenCountFilterWrapper;
use tantivy::tokenizer::{TokenFilter, Tokenizer};

/// [TokenFilter] that limit the number of tokens
///
/// ```rust
/// use tantivy_analysis_contrib::commons::LimitTokenCountFilter;
///
/// let filter:LimitTokenCountFilter = LimitTokenCountFilter::new(5);
/// ```
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::LimitTokenCountFilter;
///
/// let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
///             .filter(LimitTokenCountFilter::from(3))
///             .token_stream("There will be 3 tokens in the end");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "There".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "will".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "be".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LimitTokenCountFilter {
    max_tokens: usize,
}

impl LimitTokenCountFilter {
    /// Create a new [LimitTokenCountFilter].
    ///
    /// # Parameters :
    /// * max_tokens : maximum number of tokens that will be indexed
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }
}

impl From<usize> for LimitTokenCountFilter {
    fn from(max_tokens: usize) -> Self {
        Self { max_tokens }
    }
}

impl TokenFilter for LimitTokenCountFilter {
    type Tokenizer<T: Tokenizer> = LimitTokenCountFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        LimitTokenCountFilterWrapper::new(token_stream, self.max_tokens)
    }
}
