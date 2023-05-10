use rphonetic::Encoder;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct GenericPhoneticTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: Box<dyn Encoder>,
    pub inject: bool,
    pub backup: Option<String>,
}

impl<'a> TokenStream for GenericPhoneticTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if let Some(backup) = &self.backup {
            self.tail.token_mut().text = backup.clone();
            self.backup = None;
            return true;
        }

        let mut result = false;
        // We while skip empty code
        while !result {
            let tail_result = self.tail.advance();
            // If an end of stream, return false, this will end the loop
            if !tail_result {
                return false;
            }
            let token = self.encoder.encode(&self.tail.token().text);

            if self.tail.token().text.is_empty() || token.is_empty() {
                return true;
            }

            if token.is_empty() && self.inject {
                // We only keep the original token
                result = true;
            } else if !token.is_empty() {
                // Otherwise, if token isn't empty
                if self.inject {
                    // We back it up if inject
                    self.backup = Some(token)
                } else {
                    // Otherwise we replace original token
                    self.tail.token_mut().text = token;
                }
                result = true;
            }
        }

        result
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Token;

    use crate::phonetic::tests::{token_stream_helper, token_stream_helper_raw};
    use crate::phonetic::{
        Alternate, Error, Mapping, MaxCodeLength, PhoneticAlgorithm, PhoneticTokenFilter,
        SpecialHW, Strict,
    };

    #[test]
    fn test_metaphone_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Metaphone(MaxCodeLength(None));
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "KKK".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ESKS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_metaphone_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Metaphone(MaxCodeLength(None));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "KKK".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ESKS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_double_metaphone_no_alternate_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(None), Alternate(false));
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "PP".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "KK".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ASKS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_double_metaphone_no_alternate_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(None), Alternate(false));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "PP".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "KK".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ASKS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_soundex_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Soundex(Mapping(None), SpecialHW(None));
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "E220".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_soundex_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Soundex(Mapping(None), SpecialHW(None));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "E220".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_refined_soundex_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::RefinedSoundex(Mapping(None));
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A0".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B1".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C3".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "E034034".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_refined_soundex_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::RefinedSoundex(Mapping(None));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A0".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B1".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C3".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "E034034".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_caverphone1_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Caverphone1;
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "P11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "K11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ASKSK1".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_caverphone1_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Caverphone1;
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "P11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "K11111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "ASKSK1".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_caverphone2_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Caverphone2;
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("Darda Karleen Datha Carlene", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "Darda".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "TTA1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 13,
                position: 1,
                text: "Karleen".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 13,
                position: 1,
                text: "KLN1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 14,
                offset_to: 19,
                position: 2,
                text: "Datha".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 14,
                offset_to: 19,
                position: 2,
                text: "TTA1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 27,
                position: 3,
                text: "Carlene".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 27,
                position: 3,
                text: "KLN1111111".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_caverphone2_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Caverphone2;
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("Darda Karleen Datha Carlene", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "TTA1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 13,
                position: 1,
                text: "KLN1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 14,
                offset_to: 19,
                position: 2,
                text: "TTA1111111".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 27,
                position: 3,
                text: "KLN1111111".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_nysiis_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Nysiis(Strict(None));
        let token_filter: PhoneticTokenFilter = algorithm.try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "aaa".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bbb".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "ccc".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "easgasg".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "EASGAS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_nysiis_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::Nysiis(Strict(None));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        // Not in the same order as Lucene
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "A".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "B".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "C".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "EASGAS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_numbers() -> Result<(), Error> {
        // No caverphone 1 & 2 because it will render 111111 & 11111111111
        let algorithms = vec![
            (
                PhoneticAlgorithm::Metaphone(MaxCodeLength(None)),
                "Metaphone",
            ),
            (
                PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(None), Alternate(false)),
                "Double Metaphone (no alternate)",
            ),
            (
                PhoneticAlgorithm::Soundex(Mapping(None), SpecialHW(None)),
                "Soundex",
            ),
            (
                PhoneticAlgorithm::RefinedSoundex(Mapping(None)),
                "Refined Soundex",
            ),
            (PhoneticAlgorithm::Nysiis(Strict(None)), "Nyiis"),
            (PhoneticAlgorithm::Phonex(MaxCodeLength(None)), "Phonex"),
        ];

        for (algorithm, name) in &algorithms {
            let token_filter = (algorithm, false).try_into()?;

            let result = token_stream_helper_raw("1234567891011", token_filter);
            let expected = vec![Token {
                offset_from: 0,
                offset_to: 13,
                position: 0,
                text: "1234567891011".to_string(),
                position_length: 1,
            }];

            assert_eq!(result, expected, "\n{name}");
        }

        Ok(())
    }

    #[test]
    fn test_empty_term() -> Result<(), Error> {
        let inject = vec![true, false];
        let algorithms = vec![
            (
                PhoneticAlgorithm::Metaphone(MaxCodeLength(None)),
                "Metaphone",
            ),
            (
                PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(None), Alternate(false)),
                "Double Metaphone (no alternate)",
            ),
            (
                PhoneticAlgorithm::Soundex(Mapping(None), SpecialHW(None)),
                "Soundex",
            ),
            (
                PhoneticAlgorithm::RefinedSoundex(Mapping(None)),
                "Refined Soundex",
            ),
            (PhoneticAlgorithm::Caverphone1, "Caverphone 1"),
            (PhoneticAlgorithm::Caverphone2, "Caverphone 2"),
            (PhoneticAlgorithm::Nysiis(Strict(None)), "Nyiis"),
            (PhoneticAlgorithm::Phonex(MaxCodeLength(None)), "Phonex"),
        ];

        for inject in inject {
            for (algorithm, name) in &algorithms {
                let token_filter = (algorithm, inject).try_into()?;

                let result = token_stream_helper_raw("", token_filter);
                let expected = vec![Token {
                    offset_from: 0,
                    offset_to: 0,
                    position: 0,
                    text: "".to_string(),
                    position_length: 1,
                }];

                assert_eq!(result, expected, "\n{name} (inject {inject})");
            }
        }

        Ok(())
    }
}
