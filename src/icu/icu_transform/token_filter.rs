use super::super::Error;
use super::{Direction, ICUTransformFilterWrapper};
use rust_icu_utrans as utrans;
use tantivy::tokenizer::{TokenFilter, Tokenizer};

/// This [TokenFilter] allow to transform text into another,
/// for example, to performe transliteration.
/// See [ICU documentation](https://unicode-org.github.io/icu/userguide/transforms/general/)
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};
///
/// let token_filter = ICUTransformTokenFilter::new(
///     "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
///     None,
///     Direction::Forward
/// )?;
/// #     Ok(())
/// # }
/// ```
///
/// # Example
///
/// Here is an example of transform that converts greek letters into latin letters
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};
///
/// let mut tmp = TextAnalyzer::builder(RawTokenizer::default())
///    .filter(ICUTransformTokenFilter::new(
///       "Greek-Latin".to_string(),
///       None,
///       Direction::Forward
///    )?)
///    .build();
/// let mut token_stream = tmp.token_stream("Αλφαβητικός Κατάλογος");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Alphabētikós Katálogos".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
#[derive(Clone, Debug)]
pub struct ICUTransformTokenFilter {
    /// [Compound transform](https://unicode-org.github.io/icu/userguide/transforms/general/#compound-ids)
    compound_id: String,
    /// Custom transform [rules](https://unicode-org.github.io/icu/userguide/transforms/general/rules.html)
    rules: Option<String>,
    /// Direction
    direction: Direction,
}

impl ICUTransformTokenFilter {
    /// Construct a new transform filter.
    ///
    /// # Parameters :
    ///
    /// * `compound_id` : [Compound transform](https://unicode-org.github.io/icu/userguide/transforms/general/#compound-ids)
    /// * `rules` : Custom transform [rules](https://unicode-org.github.io/icu/userguide/transforms/general/rules.html)
    /// * `direction` : Direction
    pub fn new(
        compound_id: String,
        rules: Option<String>,
        direction: Direction,
    ) -> Result<Self, Error> {
        let _ =
            utrans::UTransliterator::new(compound_id.as_str(), rules.as_deref(), direction.into())?;

        Ok(Self {
            compound_id,
            rules,
            direction,
        })
    }
}

impl TokenFilter for ICUTransformTokenFilter {
    type Tokenizer<T: Tokenizer> = ICUTransformFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        ICUTransformFilterWrapper::new(token_stream, self.compound_id, self.rules, self.direction)
    }
}
