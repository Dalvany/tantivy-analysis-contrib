//! This module contains all ICU related components.
#[cfg(feature = "normalizer")]
pub mod normalizer;
#[cfg(feature = "tokenizer")]
pub mod tokenizer;
#[cfg(feature = "transform")]
pub mod transform;
