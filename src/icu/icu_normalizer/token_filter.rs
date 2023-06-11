use rust_icu_unorm2::UNormalizer;
use tantivy::tokenizer::{TokenFilter, Tokenizer};

use super::super::Error;
use super::{ICUNormalizer2FilterWrapper, Mode};

/// [TokenFilter] that converts text into a normal form.
/// It supports all [Google's unicode normalization](https://docs.rs/rust_icu_unorm2/2.0.0/rust_icu_unorm2/struct.UNormalizer.html) using [Mode]:
/// * NFC
/// * NFD
/// * NFKC
/// * NFKD
/// * NFKC casefold
///
/// See Wikipedia's [unicode normalization](https://en.wikipedia.org/wiki/Unicode_equivalence#Normalization) or
/// [Unicode documentation](https://www.unicode.org/reports/tr15/) for more information.
///
/// Building an [ICUNormalizer2TokenFilter] is straightforward :
/// ```rust
/// use tantivy_analysis_contrib::icu::ICUNormalizer2TokenFilter;
/// use tantivy_analysis_contrib::icu::Mode;
///
/// let normalizer = ICUNormalizer2TokenFilter {
///     mode: Mode::NFD,
/// };
/// ```
///
/// # Example
///
/// Here is an example showing which tokens are produce
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::{ICUNormalizer2TokenFilter, Mode};
///
/// let mut token_stream = TextAnalyzer::from(RawTokenizer)
///             .filter(ICUNormalizer2TokenFilter::new(Mode::NFKCCasefold))
///             .token_stream("Ruß");
///
/// let token = token_stream.next().expect("A token should be present.");
///
/// assert_eq!(token.text, "russ".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ICUNormalizer2TokenFilter {
    mode: Mode,
}

impl ICUNormalizer2TokenFilter {
    /// Construct a new normalizer 2 token filter.
    ///
    /// # Parameters :
    ///
    /// * `mode` : Normalization algorithm.
    pub fn new(mode: Mode) -> Result<Self, Error> {
        let _ = UNormalizer::try_from(mode)?;
        Ok(mode.into())
    }
}

impl From<Mode> for ICUNormalizer2TokenFilter {
    fn from(mode: Mode) -> Self {
        ICUNormalizer2TokenFilter { mode }
    }
}

impl TokenFilter for ICUNormalizer2TokenFilter {
    type Tokenizer<T: Tokenizer> = ICUNormalizer2FilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        ICUNormalizer2FilterWrapper::new(token_stream, self.mode)
    }
}
