use tantivy::tokenizer::{TokenFilter, Tokenizer};

use super::ReverseFilterWrapper;

/// This is a [TokenFilter] that reverse a string.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::ReverseTokenFilter;
///
/// let mut tmp = TextAnalyzer::builder(RawTokenizer::default())
///    .filter(ReverseTokenFilter)
///    .build();
/// let mut token_stream = tmp.token_stream("ReverseTokenFilter");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "retliFnekoTesreveR".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ReverseTokenFilter;

impl TokenFilter for ReverseTokenFilter {
    type Tokenizer<T: Tokenizer> = ReverseFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        ReverseFilterWrapper::new(token_stream)
    }
}
