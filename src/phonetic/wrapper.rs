//! Module that contains the `wrapper`. From what I understand
//! it's mostly here to give to the bottom component of the analysis
//! stack (which is a [Tokenizer]) the text to parse.

use rphonetic::{BeiderMorseBuilder, Encoder, Phonex};
use tantivy_tokenizer_api::{TokenStream, Tokenizer};

use super::{
    BeiderMorseTokenStream, DaitchMokotoffTokenStream, DoubleMetaphoneTokenStream,
    EncoderAlgorithm, GenericPhoneticTokenStream,
};

/// Phonex wrapper to handle the case only '0'.
/// This structure implements rphonetic's trait
/// [Encoder] that delegates call to phonex encoder
/// and then handle the specific case.
struct PhonexWrapper(Phonex);

impl Encoder for PhonexWrapper {
    fn encode(&self, s: &str) -> String {
        let result = self.0.encode(s);
        // If only '0' then treat as empty string.
        if result.bytes().any(|b| b != b'0') {
            result
        } else {
            "".to_owned()
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhoneticFilterWrapper<T> {
    algorithm: EncoderAlgorithm,
    inject: bool,
    inner: T,
}

impl<T> PhoneticFilterWrapper<T> {
    pub(crate) fn new(inner: T, algorithm: EncoderAlgorithm, inject: bool) -> Self {
        Self {
            algorithm,
            inject,
            inner,
        }
    }
}

impl<T: Tokenizer> Tokenizer for PhoneticFilterWrapper<T> {
    type TokenStream<'a> = Box<dyn TokenStream + 'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
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
                Box::new(BeiderMorseTokenStream::new(
                    self.inner.token_stream(text),
                    encoder,
                    max_phonemes,
                    languages_set.clone(),
                    self.inject,
                ))
            }
            // Caverphone1
            EncoderAlgorithm::Caverphone1(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Caverphone2
            EncoderAlgorithm::Caverphone2(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Cologne
            EncoderAlgorithm::Cologne(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Daitch Mokotoff
            EncoderAlgorithm::DaitchMokotoffSoundex(encoder, branching) => {
                Box::new(DaitchMokotoffTokenStream::new(
                    self.inner.token_stream(text),
                    encoder.clone(),
                    *branching,
                    self.inject,
                ))
            }
            // Double Metaphone
            EncoderAlgorithm::DoubleMetaphone(encoder, use_alternate) => match use_alternate {
                // Alternate: if true, use specific token filter, otherwise, use generic
                true => Box::new(DoubleMetaphoneTokenStream::new(
                    self.inner.token_stream(text),
                    *encoder,
                    self.inject,
                )),
                false => Box::new(GenericPhoneticTokenStream::new(
                    self.inner.token_stream(text),
                    Box::new(*encoder),
                    self.inject,
                )),
            },
            // Match Rating Approach
            EncoderAlgorithm::MatchRatingApproach(encoder) => {
                Box::new(GenericPhoneticTokenStream::new(
                    self.inner.token_stream(text),
                    Box::new(*encoder),
                    self.inject,
                ))
            }
            // Metaphone
            EncoderAlgorithm::Metaphone(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Nysiis
            EncoderAlgorithm::Nysiis(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Phonex
            EncoderAlgorithm::Phonex(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(PhonexWrapper(*encoder)),
                self.inject,
            )),
            // Refined Soundex
            EncoderAlgorithm::RefinedSoundex(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
            // Soundex
            EncoderAlgorithm::Soundex(encoder) => Box::new(GenericPhoneticTokenStream::new(
                self.inner.token_stream(text),
                Box::new(*encoder),
                self.inject,
            )),
        }
    }
}
