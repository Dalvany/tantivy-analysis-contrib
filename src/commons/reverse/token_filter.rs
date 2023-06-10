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
/// let mut token_stream = TextAnalyzer::from(RawTokenizer)
///             .filter(ReverseTokenFilter)
///             .token_stream("ReverseTokenFilter");
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

    fn transform<T: Tokenizer>(self, token_stream: T) -> ReverseFilterWrapper<T> {
        ReverseFilterWrapper {
            inner: token_stream,
        }
    }
}
