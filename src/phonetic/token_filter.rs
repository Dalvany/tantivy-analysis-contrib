use tantivy_tokenizer_api::{TokenFilter, Tokenizer};

use super::{EncoderAlgorithm, Error, PhoneticAlgorithm, PhoneticFilterWrapper};

/// This the phonetic token filter.
/// It generates a token according
/// to the algorithm provided.
///
/// You should use [PhoneticAlgorithm] to construct a new [PhoneticTokenFilter].
///
/// ```rust
/// # fn main() -> Result<(), tantivy_analysis_contrib::phonetic::Error> {
/// use tantivy_analysis_contrib::phonetic::{Alternate, MaxCodeLength, PhoneticAlgorithm, PhoneticTokenFilter, Strict};
///
/// // Example with Double Metaphone.
/// let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(None), Alternate(false));
/// let token_filter = PhoneticTokenFilter::try_from(algorithm)?;
///
/// // Another example with Nysiis
/// let algorithm = PhoneticAlgorithm::Nysiis(Strict(None));
/// let token_filter = PhoneticTokenFilter::try_from(algorithm)?;
///
/// #    Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct PhoneticTokenFilter {
    algorithm: EncoderAlgorithm,
    inject: bool,
}

impl TokenFilter for PhoneticTokenFilter {
    type Tokenizer<T: Tokenizer> = PhoneticFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, token_stream: T) -> Self::Tokenizer<T> {
        PhoneticFilterWrapper::new(token_stream, self.algorithm, self.inject)
    }
}

/// Get the token filter from a [PhoneticAlgorithm]. This will
/// take care of all the boilerplate.
///
/// The boolean indicates if encoded values should be treated as synonyms (`true`), in
/// this case the original token will be present, or if it should replace (`false`) the
/// original token.
impl TryFrom<(PhoneticAlgorithm, bool)> for PhoneticTokenFilter {
    type Error = Error;

    fn try_from((value, inject): (PhoneticAlgorithm, bool)) -> Result<Self, Self::Error> {
        (&value, inject).try_into()
    }
}

/// Get the token filter from a [PhoneticAlgorithm]. This will
/// take care of all the boilerplate.
///
/// The boolean indicates if encoded values should be treated as synonyms (`true`), in
/// this case the original token will be present, or if it should replace (`false`) the
/// original token.
impl TryFrom<(&PhoneticAlgorithm, bool)> for PhoneticTokenFilter {
    type Error = Error;

    fn try_from((value, inject): (&PhoneticAlgorithm, bool)) -> Result<Self, Self::Error> {
        let algorithm: EncoderAlgorithm = value.try_into()?;
        Ok(Self { algorithm, inject })
    }
}

/// Get the token filter from a [PhoneticAlgorithm]. This will
/// take care of all the boilerplate.
///
/// Encoded values will be added as synonyms; that means the original
/// token will be present.
impl TryFrom<PhoneticAlgorithm> for PhoneticTokenFilter {
    type Error = Error;

    fn try_from(value: PhoneticAlgorithm) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

/// Get the token filter from a [PhoneticAlgorithm]. This will
/// take care of all the boilerplate.
///
/// Encoded values will be added as synonyms; that means the original
/// token will be present.
impl TryFrom<&PhoneticAlgorithm> for PhoneticTokenFilter {
    type Error = Error;

    fn try_from(value: &PhoneticAlgorithm) -> Result<Self, Self::Error> {
        let algorithm: EncoderAlgorithm = value.try_into()?;
        Ok(Self {
            algorithm,
            inject: true,
        })
    }
}
