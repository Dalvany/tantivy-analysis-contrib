//! This module contains all ICU related components.
#[cfg(feature = "normalizer")]
mod normalizer;
#[cfg(feature = "tokenizer")]
mod tokenizer;
#[cfg(feature = "transform")]
mod transform;

#[cfg(feature = "normalizer")]
pub use crate::icu::normalizer::ICUNormalizer2TokenFilter;
#[cfg(feature = "normalizer")]
pub use crate::icu::normalizer::Mode;
#[cfg(feature = "tokenizer")]
pub use crate::icu::tokenizer::ICUTokenizer;
#[cfg(feature = "transform")]
pub use crate::icu::transform::Direction;
#[cfg(feature = "transform")]
pub use crate::icu::transform::ICUTransformTokenFilter;
