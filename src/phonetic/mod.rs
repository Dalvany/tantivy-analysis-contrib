//! This module provides phonetic capabilities through several algorithms.
//!
//! It contains the following algorithms :
//! * Beider-Morse
//! * Caverphone 1 & 2
//! * Cologne
//! * Daitch Mokotoff Soundex
//! * Double Metaphone
//! * Match Rating Approach
//! * Metaphone
//! * Nysiis
//! * Refined Soundex
//! * Soundex
//! * Phonex
//!
//! To get a [PhoneticTokenFilter] you need to use [PhoneticAlgorithm] :
//!
//! ```rust
//! # fn main() -> Result<(), tantivy_analysis_contrib::phonetic::Error> {
//! use tantivy_analysis_contrib::phonetic::{
//!     Mapping,
//!     PhoneticAlgorithm,
//!     PhoneticTokenFilter,
//!     SpecialHW
//! };
//!
//! let algorithm = PhoneticAlgorithm::Soundex(Mapping(None), SpecialHW(None));
//! let token_filter = PhoneticTokenFilter::try_from(algorithm)?;
//! #   Ok(())
//! # }
//! ```
//!
//! Every parameter of [PhoneticAlgorithm]'s variant is typed to try to make it clear what is their purpose.
//! Most of them are [Option] allowing to use default values.

pub use rphonetic::{BMError, LanguageSet, NameType, PhoneticError, RuleType};
use rphonetic::{
    Caverphone1, Caverphone2, Cologne, ConfigFiles, DaitchMokotoffSoundex,
    DaitchMokotoffSoundexBuilder, DoubleMetaphone, MatchRatingApproach, Metaphone, Nysiis, Phonex,
    RefinedSoundex, Soundex, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
};
use thiserror::Error;
pub use token_filter::PhoneticTokenFilter;
use token_stream::{
    BeiderMorseTokenStream, DaitchMokotoffTokenStream, DoubleMetaphoneTokenStream,
    GenericPhoneticTokenStream,
};
pub use types::*;
use wrapper::PhoneticFilterWrapper;

mod token_filter;
mod token_stream;
mod types;
mod wrapper;

/// Errors from encoder.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Fail to create the encoder. It contains the rphonetic error.
    #[error("{0}")]
    AlgorithmError(#[from] PhoneticError),
}

/// These are different algorithms from [rphonetic crate](https://docs.rs/rphonetic/1.0.0/rphonetic/).
///
/// It tries to remove most of the boilerplate of getting an [Encoder](rphonetic::Encoder).
///
/// Parameters are mostly wrapper to make clearer what they mean.
#[derive(Clone, Debug)]
pub enum PhoneticAlgorithm {
    /// [BeiderMorse](rphonetic::BeiderMorse) algorithm.
    ///
    /// You need to provide the [ConfigFiles]. If feature `embedded_bm` is enabled,
    /// you will be able to get one with a minimal set of rules (see rphonetic crate).
    ///
    /// The [NameType] allows you to choose between the supported set type of names. If none
    /// is provided, it will use [Generic](NameType::Generic).
    ///
    /// The [RuleType] allows you to choose between [approx](RuleType::Approx) and [exact](RuleType::Exact).
    /// If none is provided, the default is `approx`.
    ///
    /// You have to provide a set of languages. They must be supported by your rule files. If the list
    /// is empty, the encoder will try to guess languages.
    BeiderMorse(
        &'static ConfigFiles,
        Option<NameType>,
        Option<RuleType>,
        Concat,
        MaxPhonemeNumber,
        Vec<String>,
    ),
    /// [Caverphone1] algorithm.
    Caverphone1,
    /// [Caverphone2] algorithm.
    Caverphone2,
    /// [Cologne] algorithm.
    Cologne,
    /// [DaitchMokotoffSoundex] algorithm. You will need to provide the encoder's
    /// rules as a string.
    ///
    DaitchMokotoffSoundex(DMRule, Folding, Branching),
    /// [DoubleMetaphone] algorithm. The integer is maximum length of generated codes.
    /// If `None` is provided, then the default maximum code length will apply.
    ///
    /// Boolean indicates if we also want to encode alternate value (`true`) or not (`false`).
    DoubleMetaphone(MaxCodeLength, Alternate),
    /// This is the [MatchRatingApproach] algorithm.
    MatchRatingApproach,
    /// [Metaphone] algorithm. The integer is maximum length of generated codes.
    /// If `None` is provided, then the default maximum code length will apply.
    Metaphone(MaxCodeLength),
    /// [Nysiis] algorithm.
    /// The boolean indicate if codes are strict or not.
    /// If `None` it will use the default.
    Nysiis(Strict),
    /// [Phonex] algorithm. The integer is the maximum length of generated codes.
    Phonex(MaxCodeLength),
    /// [RefinedSoundex] algorithm.
    /// If you provide a mapping it will be used, otherwise
    /// [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] will apply.
    RefinedSoundex(Mapping),
    /// [Soundex] algorithm.
    /// If you provide a mapping it will be used, otherwise
    /// [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] will apply.
    /// The boolean indicates `H` and `W` should be treated as silence.
    /// If `None`
    /// is provided, then default to `true`.
    Soundex(Mapping, SpecialHW),
}

// Indirection for getting the filter.
// This enum maps PhoneticAlgorithm into the
// proper encoder implem, avoiding unwrapping
// when calling build() on DaitchMokotoffSoundexBuilder.
#[derive(Clone, Debug)]
pub(crate) enum EncoderAlgorithm {
    // We will recreate the BeiderMorse as it has a lifetime, and it could be in the phonetic token filter...
    BeiderMorse(
        &'static ConfigFiles,
        Option<NameType>,
        Option<RuleType>,
        Option<bool>,
        Option<usize>,
        Option<LanguageSet>,
    ),
    Caverphone1(Caverphone1),
    Caverphone2(Caverphone2),
    Cologne(Cologne),
    DaitchMokotoffSoundex(DaitchMokotoffSoundex, bool),
    DoubleMetaphone(DoubleMetaphone, bool),
    MatchRatingApproach(MatchRatingApproach),
    Metaphone(Metaphone),
    Nysiis(Nysiis),
    Phonex(Phonex),
    RefinedSoundex(RefinedSoundex),
    Soundex(Soundex),
}

impl TryFrom<PhoneticAlgorithm> for EncoderAlgorithm {
    type Error = Error;

    fn try_from(value: PhoneticAlgorithm) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&PhoneticAlgorithm> for EncoderAlgorithm {
    type Error = Error;

    fn try_from(value: &PhoneticAlgorithm) -> Result<Self, Self::Error> {
        match value {
            PhoneticAlgorithm::BeiderMorse(
                config_files,
                name_type,
                rule_type,
                concat,
                max_phonames,
                languages_set,
            ) => {
                let languages_set = languages_set
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<&str>>();
                let languages_set = LanguageSet::from(languages_set);
                let languages_set = if languages_set.is_empty() {
                    None
                } else {
                    Some(languages_set)
                };
                Ok(EncoderAlgorithm::BeiderMorse(
                    config_files,
                    *name_type,
                    *rule_type,
                    concat.0,
                    max_phonames.0,
                    languages_set,
                ))
            }
            PhoneticAlgorithm::Caverphone1 => Ok(EncoderAlgorithm::Caverphone1(Caverphone1)),
            PhoneticAlgorithm::Caverphone2 => Ok(EncoderAlgorithm::Caverphone2(Caverphone2)),
            PhoneticAlgorithm::Cologne => Ok(EncoderAlgorithm::Cologne(Cologne)),
            #[cfg(feature = "embedded_dm")]
            PhoneticAlgorithm::DaitchMokotoffSoundex(rules, ascii_folding, branching) => {
                let encoder = match &rules.0 {
                    None => DaitchMokotoffSoundexBuilder::default()
                        .ascii_folding(ascii_folding.0)
                        .build()?,
                    Some(rules) => DaitchMokotoffSoundexBuilder::with_rules(rules.as_str())
                        .ascii_folding(ascii_folding.0)
                        .build()?,
                };
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(
                    encoder,
                    branching.0,
                ))
            }
            #[cfg(not(feature = "embedded_dm"))]
            PhoneticAlgorithm::DaitchMokotoffSoundex(rules, ascii_folding, branching) => {
                let encoder = DaitchMokotoffSoundexBuilder::with_rules(rules.0.as_str())
                    .ascii_folding(ascii_folding.0)
                    .build()?;
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(
                    encoder,
                    branching.0,
                ))
            }
            PhoneticAlgorithm::DoubleMetaphone(max_code_length, use_alternate) => {
                // Alternate: if true, uses specific token filter, otherwise, use generic
                match max_code_length.0 {
                    None => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::default(),
                        use_alternate.0,
                    )),
                    Some(max_code_length) => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::new(Some(max_code_length)),
                        use_alternate.0,
                    )),
                }
            }
            PhoneticAlgorithm::MatchRatingApproach => {
                Ok(EncoderAlgorithm::MatchRatingApproach(MatchRatingApproach))
            }
            PhoneticAlgorithm::Metaphone(max_code_length) => match max_code_length.0 {
                None => Ok(EncoderAlgorithm::Metaphone(Metaphone::default())),
                Some(max_code_length) => Ok(EncoderAlgorithm::Metaphone(Metaphone::new(Some(
                    max_code_length,
                )))),
            },
            PhoneticAlgorithm::Nysiis(strict) => match strict.0 {
                None => Ok(EncoderAlgorithm::Nysiis(Nysiis::default())),
                Some(strict) => Ok(EncoderAlgorithm::Nysiis(Nysiis::new(strict))),
            },
            PhoneticAlgorithm::Phonex(max_code_length) => match max_code_length.0 {
                None => Ok(EncoderAlgorithm::Phonex(Phonex::default())),
                Some(max_code_length) => Ok(EncoderAlgorithm::Phonex(Phonex::new(max_code_length))),
            },
            PhoneticAlgorithm::RefinedSoundex(mapping) => match mapping.0 {
                None => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::default())),
                Some(mapping) => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::new(
                    mapping,
                ))),
            },
            PhoneticAlgorithm::Soundex(mapping, special_h_w) => match (mapping.0, special_h_w.0) {
                (None, None) => Ok(EncoderAlgorithm::Soundex(Soundex::default())),
                (Some(mapping), None) => Ok(EncoderAlgorithm::Soundex(Soundex::from(mapping))),
                (None, Some(w_h)) => Ok(EncoderAlgorithm::Soundex(Soundex::new(
                    DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
                    w_h,
                ))),
                (Some(mapping), Some(h_w)) => {
                    Ok(EncoderAlgorithm::Soundex(Soundex::new(mapping, h_w)))
                }
            },
        }
    }
}

// Tests are in the respective token stream tested
// It contains the helper method...
#[cfg(test)]
pub(crate) mod tests {
    use tantivy::tokenizer::{RawTokenizer, TextAnalyzer, Token, WhitespaceTokenizer};

    use crate::phonetic::PhoneticTokenFilter;

    pub fn token_stream_helper(text: &str, token_filter: PhoneticTokenFilter) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(token_filter)
            .build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    pub fn token_stream_helper_raw(text: &str, token_filter: PhoneticTokenFilter) -> Vec<Token> {
        let mut a = TextAnalyzer::builder(RawTokenizer::default())
            .filter(token_filter)
            .build();

        let mut token_stream = a.token_stream(text);

        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }
}
