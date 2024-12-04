use std::collections::VecDeque;

use rphonetic::{BeiderMorse, Encoder, LanguageSet};
use tantivy_tokenizer_api::{Token, TokenStream};

pub(crate) struct BeiderMorseTokenStream<'a, T> {
    tail: T,
    encoder: BeiderMorse<'a>,
    codes: VecDeque<String>,
    languages: Option<LanguageSet>,
    inject: bool,
}

impl<'a, T> BeiderMorseTokenStream<'a, T> {
    pub(crate) fn new(
        tail: T,
        encoder: BeiderMorse<'a>,
        max_phonemes: usize,
        languages: Option<LanguageSet>,
        inject: bool,
    ) -> Self {
        Self {
            tail,
            encoder,
            codes: VecDeque::with_capacity(max_phonemes),
            languages,
            inject,
        }
    }
}

impl<T: TokenStream> TokenStream for BeiderMorseTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        while self.codes.is_empty() {
            if !self.tail.advance() {
                return false;
            }
            if self.tail.token().text.is_empty() {
                return true;
            }

            let encoded = match &self.languages {
                None => self.encoder.encode(&self.tail.token().text),
                Some(languages) => self
                    .encoder
                    .encode_with_languages(&self.tail.token().text, languages),
            };
            let mut start_token = 0;
            let mut end_token = 0;
            let mut start = true;
            // "Simple" parsing of potentially nested (...|...|...)-(...|...|...)
            for (index, ch) in encoded.char_indices() {
                if ch != '(' && ch != ')' && ch != '-' && ch != '|' {
                    if start {
                        start_token = index;
                        end_token = index;
                        start = false;
                    } else {
                        end_token += 1;
                    }
                } else if start_token < end_token {
                    self.codes
                        .push_back(encoded[start_token..=end_token].to_string());
                    start_token = end_token;
                    start = true;
                }
            }

            // Handle last code
            if start_token < end_token {
                self.codes
                    .push_back(encoded[start_token..=end_token].to_string());
            }

            if self.inject || encoded.is_empty() {
                return true;
            }
        }

        let code = self.codes.pop_front();
        match code {
            Some(code) => {
                self.tail.token_mut().text = code;
                true
            }
            None => false,
        }
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
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use rphonetic::{ConfigFiles, RuleType};

    use super::*;
    use crate::phonetic::tests::token_stream_helper;
    use crate::phonetic::{Concat, Error, MaxPhonemeNumber, PhoneticAlgorithm};

    lazy_static! {
        static ref CONFIG_FILES: ConfigFiles =
            ConfigFiles::new(&PathBuf::from("./test_assets/bm-cc-rules")).unwrap();
    }

    #[test]
    fn test_basic_usage_inject() -> Result<(), Error> {
        let algorithm = &PhoneticAlgorithm::BeiderMorse(
            &CONFIG_FILES,
            None,
            Some(RuleType::Exact),
            Concat(Some(true)),
            MaxPhonemeNumber(None),
            vec![],
        );

        let result = token_stream_helper("Angelo", algorithm.try_into()?);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "Angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "andZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anxelo".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        let result = token_stream_helper("D'Angelo", algorithm.try_into()?);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "D'Angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "andZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anxelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "dandZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "dangelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danxelo".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_basic_usage_not_inject() -> Result<(), Error> {
        let algorithm = &PhoneticAlgorithm::BeiderMorse(
            &CONFIG_FILES,
            None,
            Some(RuleType::Exact),
            Concat(Some(true)),
            MaxPhonemeNumber(None),
            vec![],
        );

        let result = token_stream_helper("Angelo", (algorithm, false).try_into()?);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "andZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anxelo".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        let result = token_stream_helper("D'Angelo", (algorithm, false).try_into()?);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "andZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "anxelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "dandZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "dangelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danhelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danjelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "danxelo".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_language_set() -> Result<(), Error> {
        let algorithm = &PhoneticAlgorithm::BeiderMorse(
            &CONFIG_FILES,
            None,
            Some(RuleType::Exact),
            Concat(Some(true)),
            MaxPhonemeNumber(None),
            vec![
                "italian".to_string(),
                "greek".to_string(),
                "spanish".to_string(),
            ],
        );

        let result = token_stream_helper("Angelo", (algorithm, false).try_into()?);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "andZelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "angelo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "anxelo".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_numbers() -> Result<(), Error> {
        let algorithm = &PhoneticAlgorithm::BeiderMorse(
            &CONFIG_FILES,
            None,
            Some(RuleType::Exact),
            Concat(Some(true)),
            MaxPhonemeNumber(None),
            vec![],
        );

        let result = token_stream_helper("1234", (algorithm, false).try_into()?);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "1234".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);

        Ok(())
    }
}
