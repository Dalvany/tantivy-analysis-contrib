//! This module provide phonetic capabilities through several algorithm.
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

pub use rphonetic::{BMError, LanguageSet, NameType, PhoneticError, RuleType};
use rphonetic::{
    BeiderMorseBuilder, Caverphone1, Caverphone2, Cologne, ConfigFiles, DaitchMokotoffSoundex,
    DaitchMokotoffSoundexBuilder, DoubleMetaphone, MatchRatingApproach, Metaphone, Nysiis,
    RefinedSoundex, Soundex, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
};
use tantivy::tokenizer::{BoxTokenStream, TokenFilter};

use crate::phonetic::beider_morse::BeiderMorseTokenStream;
use crate::phonetic::daitch_mokotoff::DaitchMokotoffTokenStream;
use crate::phonetic::double_metaphone::DoubleMetaphoneTokenStream;
use crate::phonetic::generic::GenericPhoneticTokenStream;

mod beider_morse;
mod daitch_mokotoff;
mod double_metaphone;
mod generic;

/// Errors from encoder.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Fail to create the encoder. It contains the rphonetic error.
    AlgorithmError(PhoneticError),
}

impl From<PhoneticError> for Error {
    fn from(error: PhoneticError) -> Self {
        Self::AlgorithmError(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlgorithmError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {}

/// These are different algorithms from [rphonetic crate](https://docs.rs/rphonetic/1.0.0/rphonetic/).
///
/// It tries to remove most of the boilerplate of getting an [Encoder].
#[derive(Clone, Debug)]
pub enum PhoneticAlgorithm {
    /// [BeiderMorse] algorithm.
    ///
    /// You need to provide the [ConfigFiles]. If feature `embedded_bm` is enabled,
    /// you will be able to get one with a minimal set of rules (see rphonetic crate).
    ///
    /// The [NameType] allow you to choose between the supported set type of names. If none
    /// is provided, it will use [Generic](NameType::Generic).
    ///
    /// The [RuleType] allolw you to choose between [approx](RuleType::Approx) and [exact](RuleType::Exact).
    /// If none is provided, the default is `approx`.
    ///
    /// The boolean indicates if multiple tokens should be concatenated using `|`. If none is provided, it
    /// will default to `true`.
    ///
    /// The integer allow to choose the maximum number of phoneme the encoder will generator. If none is
    /// provided, it will default to 20.
    ///
    /// Finally, you have to provide a set of language. They must be supported by your rule files. If the list
    /// is empty, the encoder will try to guess languages.
    BeiderMorse(
        &'static ConfigFiles,
        Option<NameType>,
        Option<RuleType>,
        Option<bool>,
        Option<usize>,
        Vec<String>,
    ),
    /// [Caverphone1] algorithm.
    Caverphone1,
    /// [Caverphone2] algorithm.
    Caverphone2,
    /// [Cologne] algorithm.
    Cologne,
    #[cfg(feature = "embedded_dm")]
    /// [DaitchMokotoffSoundex] algorithm. If you want to use non default rules,
    /// you need to provide the encoder's rules as a string.
    ///
    /// The first boolean indicates if encoder should apply any folding rules (if
    /// there are any).
    ///
    /// The second boolean indicates if we allow branching. In this case, multiple code
    /// can be generated, otherwise, there will be only one.
    DaitchMokotoffSoundex(Option<String>, bool, bool),
    #[cfg(not(feature = "embedded_dm"))]
    /// [DaitchMokotoffSoundex] algorithm. You will need to provide the encoder's
    /// rules as a string.
    ///
    /// The first boolean indicates if encoder should apply any folding rules (if
    /// there are any).
    ///
    /// The second boolean indicates if we allow branching. In this case, multiple code
    /// can be generated, otherwise, there will be only one.
    DaitchMokotoffSoundex(String, bool, bool),
    /// [DoubleMetaphone] algorithm. The integer is maximum length of generated codes.
    /// If `None` is provided, then the default maximum code length will apply.
    ///
    /// Boolean indicate if we also want to encode alternate value (`true`) or not (`false`).
    DoubleMetaphone(Option<usize>, bool),
    /// This is the [MatchRatingApproach] algorithm.
    MatchRatingApproach,
    /// [Metaphone] algorithm. The integer is maximum length of generated codes.
    /// If `None` is provided, then the default maximum code length will apply.
    Metaphone(Option<usize>),
    /// [Nysiis] algorithm. The boolean indicate if codes will be strict or not.
    /// If `None` it will use the default.
    Nysiis(Option<bool>),
    /// [RefinedSoundex] algorithm. If you provide a mapping it will be use, otherwise
    /// [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] will apply.
    RefinedSoundex(Option<[char; 26]>),
    /// [Soundex] algorithm. If you provide a mapping it will be use, otherwise
    /// [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] will apply.
    /// the boolean indicates `H` and `W` should be treated as silence. If `None`
    /// is provided, then default to `true`.
    Soundex(Option<[char; 26]>, Option<bool>),
}

// Indirection for getting the filter.
// This enum maps PhoneticAlgorithm into the
// proper encoder implem, avoiding to unwrap
// when calling build() on DaitchMokotoffSoundexBuilder.
#[derive(Clone, Debug)]
enum EncoderAlgorithm {
    // We will recreate the BeiderMorse as it has a lifetime and it could be in the phonetic token filter...
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
                    *concat,
                    *max_phonames,
                    languages_set,
                ))
            }
            PhoneticAlgorithm::Caverphone1 => Ok(EncoderAlgorithm::Caverphone1(Caverphone1)),
            PhoneticAlgorithm::Caverphone2 => Ok(EncoderAlgorithm::Caverphone2(Caverphone2)),
            PhoneticAlgorithm::Cologne => Ok(EncoderAlgorithm::Cologne(Cologne)),
            #[cfg(feature = "embedded_dm")]
            PhoneticAlgorithm::DaitchMokotoffSoundex(rules, ascii_folding, branching) => {
                let encoder = match rules {
                    None => DaitchMokotoffSoundexBuilder::default()
                        .ascii_folding(*ascii_folding)
                        .build()?,
                    Some(rules) => DaitchMokotoffSoundexBuilder::with_rules(&rules)
                        .ascii_folding(*ascii_folding)
                        .build()?,
                };
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(encoder, *branching))
            }
            #[cfg(not(feature = "embedded_dm"))]
            PhoneticAlgorithm::DaitchMokotoffSoundex(rules, ascii_folding, branching) => {
                let encoder = DaitchMokotoffSoundexBuilder::with_rules(&rules)
                    .ascii_folding(*ascii_folding)
                    .build()?;
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(encoder, *branching))
            }
            PhoneticAlgorithm::DoubleMetaphone(max_code_length, use_alternate) => {
                match (max_code_length, use_alternate) {
                    (None, true) => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::default(),
                        *use_alternate,
                    )),
                    (Some(max_code_length), true) => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::new(*max_code_length),
                        *use_alternate,
                    )),
                    (None, false) => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::default(),
                        *use_alternate,
                    )),
                    (Some(max_code_length), false) => Ok(EncoderAlgorithm::DoubleMetaphone(
                        DoubleMetaphone::new(*max_code_length),
                        *use_alternate,
                    )),
                }
            }
            PhoneticAlgorithm::MatchRatingApproach => {
                Ok(EncoderAlgorithm::MatchRatingApproach(MatchRatingApproach))
            }
            PhoneticAlgorithm::Metaphone(max_code_length) => match max_code_length {
                None => Ok(EncoderAlgorithm::Metaphone(Metaphone::default())),
                Some(max_code_length) => Ok(EncoderAlgorithm::Metaphone(Metaphone::new(
                    *max_code_length,
                ))),
            },
            PhoneticAlgorithm::Nysiis(strict) => match strict {
                None => Ok(EncoderAlgorithm::Nysiis(Nysiis::default())),
                Some(strict) => Ok(EncoderAlgorithm::Nysiis(Nysiis::new(*strict))),
            },
            PhoneticAlgorithm::RefinedSoundex(mapping) => match mapping {
                None => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::default())),
                Some(mapping) => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::new(
                    *mapping,
                ))),
            },
            PhoneticAlgorithm::Soundex(mapping, special_h_w) => match (mapping, special_h_w) {
                (None, None) => Ok(EncoderAlgorithm::Soundex(Soundex::default())),
                (Some(mapping), None) => Ok(EncoderAlgorithm::Soundex(Soundex::from(*mapping))),
                (None, Some(w_h)) => Ok(EncoderAlgorithm::Soundex(Soundex::new(
                    DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
                    *w_h,
                ))),
                (Some(mapping), Some(h_w)) => {
                    Ok(EncoderAlgorithm::Soundex(Soundex::new(*mapping, *h_w)))
                }
            },
        }
    }
}

/// This the phonetic token filter. It generates token according
/// to the algorithm provided.
#[derive(Clone, Debug)]
pub struct PhoneticTokenFilter {
    algorithm: EncoderAlgorithm,
    inject: bool,
}

impl TokenFilter for PhoneticTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        match &self.algorithm {
            // Beider Morse
            EncoderAlgorithm::BeiderMorse(
                config_files,
                name_type,
                rule_type,
                concat,
                max_phonemes,
                languages_set,
            ) => {
                let mut builder = BeiderMorseBuilder::new(config_files);
                if let Some(name_type) = name_type {
                    builder = builder.name_type(*name_type);
                }
                if let Some(rule_type) = rule_type {
                    builder = builder.rule_type(*rule_type);
                }
                if let Some(concat) = concat {
                    builder = builder.concat(*concat);
                }
                if let Some(max_phonemes) = max_phonemes {
                    builder = builder.max_phonemes(*max_phonemes);
                }

                let max_phonemes = match max_phonemes {
                    Some(max_phonemes) => *max_phonemes,
                    None => 20,
                };
                let encoder = builder.build();
                BoxTokenStream::from(BeiderMorseTokenStream {
                    tail: token_stream,
                    encoder,
                    codes: VecDeque::with_capacity(max_phonemes),
                    languages: languages_set.clone(),
                    inject: self.inject,
                })
            }
            // Caverphone1
            EncoderAlgorithm::Caverphone1(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            // Caverphone2
            EncoderAlgorithm::Caverphone2(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            // Cologne
            EncoderAlgorithm::Cologne(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            EncoderAlgorithm::DaitchMokotoffSoundex(encoder, branching) => {
                BoxTokenStream::from(DaitchMokotoffTokenStream {
                    tail: token_stream,
                    encoder: encoder.clone(),
                    branching: *branching,
                    codes: vec![],
                    inject: self.inject,
                })
            }
            // Double Metaphone
            EncoderAlgorithm::DoubleMetaphone(encoder, use_alternate) => match use_alternate {
                true => BoxTokenStream::from(DoubleMetaphoneTokenStream {
                    tail: token_stream,
                    encoder: *encoder,
                    codes: vec![],
                    inject: self.inject,
                }),
                false => BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(encoder.clone()),
                    inject: self.inject,
                    backup: None,
                }),
            },
            // Match Rating Approach
            EncoderAlgorithm::MatchRatingApproach(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            // Metaphone
            EncoderAlgorithm::Metaphone(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            // Nysiis
            EncoderAlgorithm::Nysiis(encoder) => BoxTokenStream::from(GenericPhoneticTokenStream {
                tail: token_stream,
                encoder: Box::new(*encoder),
                backup: None,
                inject: self.inject,
            }),
            // Refined Soundex
            EncoderAlgorithm::RefinedSoundex(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
            // Soundex
            EncoderAlgorithm::Soundex(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                    backup: None,
                    inject: self.inject,
                })
            }
        }
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
/// Encoded values will be added as synonyms, that means the original
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
/// Encoded values will be added as synonyms, that means the original
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

// Tests are in the respective token stream tested
// It contains the helper method...
#[cfg(test)]
pub(crate) mod tests {
    use crate::phonetic::PhoneticTokenFilter;
    use tantivy::tokenizer::{TextAnalyzer, Token, WhitespaceTokenizer};

    pub fn token_stream_helper(text: &str, token_filter: PhoneticTokenFilter) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(WhitespaceTokenizer)
            .filter(token_filter)
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }
}
