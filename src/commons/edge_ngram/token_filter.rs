use std::num::NonZeroUsize;

use tantivy::tokenizer::{TokenFilter, Tokenizer};

use super::{EdgeNgramError, EdgeNgramFilterWrapper};

/// Token filter that produce [ngram](https://docs.rs/tantivy/0.18.1/tantivy/tokenizer/struct.NgramTokenizer.html)
/// from the start of the token.
/// For example, `Quick` will generate
/// `Q`, `Qu`, `Qui`, `Quic`, ...etc.
///
/// It is configure with two parameters:
/// * min edge-ngram: the number of maximum characters (e.g. with min=3, `Quick`
/// will generate `Qui`, `Quic` and `Quick`).
/// It must be greater than 0.
/// * max edge-ngram: the number of maximum characters (e.g. with max=3, `Quick`
/// will generate `Q`, `Qu` and `Qui`.
/// It is optional, and there is no maximum then
/// it will generate up to the end of the token.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::num::NonZeroUsize;
/// use tantivy::tokenizer::{WhitespaceTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::EdgeNgramTokenFilter;
///
/// let mut tmp = TextAnalyzer::builder(WhitespaceTokenizer::default())
///    .filter(EdgeNgramTokenFilter::new(NonZeroUsize::new(2).unwrap(), NonZeroUsize::new(4), false)?)
///    .build();
/// let mut token_stream = tmp.token_stream("Quick");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Qu".to_string());
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Qui".to_string());
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Quic".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
///
/// This token filter is useful to do a "starts with" therefor a "search as you type".
///
/// It is also easy to have an efficient "ends with" by adding the [ReverseTokenFilter](crate::commons::reverse::ReverseTokenFilter)
/// before the edge ngram filter.
///
/// # How to use it
///
/// To use it, you should have another pipeline at search time that does not include
/// the edge-ngram filter.
/// Otherwise, you'll get irrelevant results.
/// Please see the [example](https://github.com/Dalvany/tantivy-analysis-contrib/tree/main/examples/edge_ngram.rs)
/// in source repository for a way to do it.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct EdgeNgramTokenFilter {
    min: NonZeroUsize,
    max: Option<NonZeroUsize>,
    keep_original_token: bool,
}

impl EdgeNgramTokenFilter {
    /// Create a new `EdgeNgramTokenFilter` with the min and max ngram
    /// provided.
    ///
    /// # Parameters
    ///
    /// * `min` : minimum edge-ngram.
    /// * `max` : maximum edge-ngram. It must be greater or equals to `min`.
    /// Provide [None](None) for unlimited.
    /// * `keep_original_token`: the complete token will also be output if
    /// the length is greater than `max`.
    pub fn new(
        min: NonZeroUsize,
        max: Option<NonZeroUsize>,
        keep_original_token: bool,
    ) -> Result<Self, EdgeNgramError> {
        // Check max
        if let Some(m) = max {
            if m < min {
                return Err(EdgeNgramError::MaximumLowerThanMinimum { min, max: m });
            }
        }

        Ok(EdgeNgramTokenFilter {
            min,
            max,
            keep_original_token,
        })
    }
}

impl From<NonZeroUsize> for EdgeNgramTokenFilter {
    fn from(ngram: NonZeroUsize) -> Self {
        // This is safe to unwrap since minGram != 0 and maxGram = minGram.
        Self::new(ngram, Some(ngram), false).unwrap()
    }
}

impl TokenFilter for EdgeNgramTokenFilter {
    type Tokenizer<T: Tokenizer> = EdgeNgramFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        EdgeNgramFilterWrapper::new(tokenizer, self.min, self.max, self.keep_original_token)
    }
}
