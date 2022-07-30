//! This module provide phonetic capabilities through several algorithm.
use std::fmt::{Display, Formatter};

pub use rphonetic::{BMError, LanguageSet, NameType, PhoneticError, RuleType};
use rphonetic::{
    BeiderMorse, BeiderMorseBuilder, Caverphone1, Caverphone2, Cologne, ConfigFiles,
    DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, DoubleMetaphone, MatchRatingApproach,
    Metaphone, Nysiis, RefinedSoundex, Soundex, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
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
        ConfigFiles,
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
    DoubleMetaphone(Option<usize>),
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
// proper encoder implem, avoiding doing all the "match"
// each time we call "transform" on token filter.
#[derive(Clone, Debug)]
enum EncoderAlgorithm {
    // We will recreate the BeiderMorse as it has a lifetime and it could be in the phonetic token filter...
    BeiderMorse(BeiderMorse, Option<LanguageSet>),
    Caverphone1(Caverphone1),
    Caverphone2(Caverphone2),
    Cologne(Cologne),
    DaitchMokotoffSoundex(DaitchMokotoffSoundex, bool),
    DoubleMetaphone(DoubleMetaphone),
    MatchRatingApproach(MatchRatingApproach),
    Metaphone(Metaphone),
    Nysiis(Nysiis),
    RefinedSoundex(RefinedSoundex),
    Soundex(Soundex),
}

impl TryFrom<PhoneticAlgorithm> for EncoderAlgorithm {
    type Error = Error;

    fn try_from(value: PhoneticAlgorithm) -> Result<Self, Self::Error> {
        match value {
            PhoneticAlgorithm::BeiderMorse(
                config_files,
                name_type,
                rule_type,
                concat,
                max_phonemes,
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

                let mut builder = BeiderMorseBuilder::new(&config_files);
                if let Some(name_type) = name_type {
                    builder = builder.name_type(name_type);
                }
                if let Some(rule_type) = rule_type {
                    builder = builder.rule_type(rule_type);
                }
                if let Some(concat) = concat {
                    builder = builder.concat(concat);
                }
                if let Some(max_phonemes) = max_phonemes {
                    builder = builder.max_phonemes(max_phonemes);
                }
                Ok(EncoderAlgorithm::BeiderMorse(
                    builder.build(),
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
                        .ascii_folding(ascii_folding)
                        .build()?,
                    Some(rules) => DaitchMokotoffSoundexBuilder::with_rules(&rules)
                        .ascii_folding(ascii_folding)
                        .build()?,
                };
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(encoder, branching))
            }
            #[cfg(not(feature = "embedded_dm"))]
            PhoneticAlgorithm::DaitchMokotoffSoundex(rules, ascii_folding, branching) => {
                let encoder = DaitchMokotoffSoundexBuilder::with_rules(&rules)
                    .ascii_folding(ascii_folding)
                    .build()?;
                Ok(EncoderAlgorithm::DaitchMokotoffSoundex(encoder, branching))
            }
            PhoneticAlgorithm::DoubleMetaphone(max_code_length) => match max_code_length {
                None => Ok(EncoderAlgorithm::DoubleMetaphone(DoubleMetaphone::default())),
                Some(max_code_length) => Ok(EncoderAlgorithm::DoubleMetaphone(
                    DoubleMetaphone::new(max_code_length),
                )),
            },
            PhoneticAlgorithm::MatchRatingApproach => {
                Ok(EncoderAlgorithm::MatchRatingApproach(MatchRatingApproach))
            }
            PhoneticAlgorithm::Metaphone(max_code_length) => match max_code_length {
                None => Ok(EncoderAlgorithm::Metaphone(Metaphone::default())),
                Some(max_code_length) => {
                    Ok(EncoderAlgorithm::Metaphone(Metaphone::new(max_code_length)))
                }
            },
            PhoneticAlgorithm::Nysiis(strict) => match strict {
                None => Ok(EncoderAlgorithm::Nysiis(Nysiis::default())),
                Some(strict) => Ok(EncoderAlgorithm::Nysiis(Nysiis::new(strict))),
            },
            PhoneticAlgorithm::RefinedSoundex(mapping) => match mapping {
                None => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::default())),
                Some(mapping) => Ok(EncoderAlgorithm::RefinedSoundex(RefinedSoundex::new(
                    mapping,
                ))),
            },
            PhoneticAlgorithm::Soundex(mapping, special_h_w) => match (mapping, special_h_w) {
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

/// This the phonetic token filter. It generates token according
/// to the algorithm provided.
#[derive(Clone, Debug)]
pub struct PhoneticTokenFilter {
    algorithm: EncoderAlgorithm,
}

impl TokenFilter for PhoneticTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        match &self.algorithm {
            // Beider Morse
            EncoderAlgorithm::BeiderMorse(encoder, languages_set) => {
                BoxTokenStream::from(BeiderMorseTokenStream {
                    tail: token_stream,
                    encoder: encoder.clone(),
                    codes: vec![],
                    languages: languages_set.clone(),
                })
            }
            // Caverphone1
            EncoderAlgorithm::Caverphone1(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            // Caverphone2
            EncoderAlgorithm::Caverphone2(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            // Cologne
            EncoderAlgorithm::Cologne(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            EncoderAlgorithm::DaitchMokotoffSoundex(encoder, branching) => {
                BoxTokenStream::from(DaitchMokotoffTokenStream {
                    tail: token_stream,
                    encoder: encoder.clone(),
                    branching: *branching,
                    codes: Vec::new(),
                })
            }
            // Double Metaphone
            EncoderAlgorithm::DoubleMetaphone(encoder) => {
                BoxTokenStream::from(DoubleMetaphoneTokenStream {
                    tail: token_stream,
                    encoder: *encoder,
                    alternate: None,
                })
            }
            // Match Rating Approach
            EncoderAlgorithm::MatchRatingApproach(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            // Metaphone
            EncoderAlgorithm::Metaphone(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            // Nysiis
            EncoderAlgorithm::Nysiis(encoder) => BoxTokenStream::from(GenericPhoneticTokenStream {
                tail: token_stream,
                encoder: Box::new(*encoder),
            }),
            // Refined Soundex
            EncoderAlgorithm::RefinedSoundex(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
            // Soundex
            EncoderAlgorithm::Soundex(encoder) => {
                BoxTokenStream::from(GenericPhoneticTokenStream {
                    tail: token_stream,
                    encoder: Box::new(*encoder),
                })
            }
        }
    }
}

/// Get the token filter from a [PhoneticAlgorithm]. This will
/// take care of all the boilerplate.
impl TryFrom<PhoneticAlgorithm> for PhoneticTokenFilter {
    type Error = Error;

    fn try_from(value: PhoneticAlgorithm) -> Result<Self, Self::Error> {
        let algorithm: EncoderAlgorithm = value.try_into()?;
        Ok(Self { algorithm })
    }
}
