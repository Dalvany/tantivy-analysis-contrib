use rphonetic::DoubleMetaphone;
use tantivy::tokenizer::{Token, TokenStream};

pub(crate) struct DoubleMetaphoneTokenStream<T> {
    tail: T,
    encoder: DoubleMetaphone,
    codes: Vec<String>,
    inject: bool,
}

impl<T> DoubleMetaphoneTokenStream<T> {
    pub(crate) fn new(tail: T, encoder:DoubleMetaphone, inject:bool) -> Self {
        Self {
            tail,
            encoder,
            codes: Vec::with_capacity(10),
            inject,
        }
    }
}

impl<T: TokenStream> TokenStream for DoubleMetaphoneTokenStream<T> {
    fn advance(&mut self) -> bool {
        if self.codes.is_empty() {
            let mut result = false;
            while !result {
                result = self.tail.advance();
                if !result {
                    return false;
                }
                if self.tail.token().text.is_empty() {
                    return true;
                }

                let encoded = self.encoder.double_metaphone(&self.tail.token().text);
                let primary = encoded.primary();
                let alternate = encoded.alternate();
                if primary.is_empty() && alternate.is_empty() && self.inject {
                    return true;
                }
                if !primary.is_empty() && !alternate.is_empty() && primary != alternate {
                    // None of the primary and the alternate are empty, then we set the whole thing.
                    if self.inject {
                        self.codes.push(primary);
                    } else {
                        self.tail.token_mut().text = primary;
                    }
                    self.codes.push(alternate);
                    result = true;
                } else if !primary.is_empty() {
                    // Primary is not empty, but alternate is empty otherwise, it
                    // runs the first if.
                    if self.inject {
                        self.codes.push(primary);
                    } else {
                        self.tail.token_mut().text = primary;
                    }
                    result = true;
                } else if !alternate.is_empty() {
                    // Alternate is not empty, but primary is empty otherwise, it
                    // runs the first if or the first else-if.
                    if self.inject {
                        self.codes.push(alternate);
                    } else {
                        self.tail.token_mut().text = alternate;
                    }
                    result = true;
                }
            }
            result
        } else {
            self.tail.token_mut().text = self.codes.pop().unwrap();
            true
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
    use tantivy::tokenizer::Token;

    use crate::phonetic::tests::{token_stream_helper, token_stream_helper_raw};
    use crate::phonetic::{
        Alternate, Error, MaxCodeLength, PhoneticAlgorithm, PhoneticTokenFilter,
    };

    #[test]
    fn test_size_4_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(4)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("international", token_filter);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 13,
            position: 0,
            text: "ANTR".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_size_4_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(4)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, true).try_into()?;

        let result = token_stream_helper("international", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 13,
                position: 0,
                text: "international".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 13,
                position: 0,
                text: "ANTR".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_alternate_not_inject_false() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(4)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("Kuczewski", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 9,
                position: 0,
                text: "KSSK".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 0,
                offset_to: 9,
                position: 0,
                text: "KXFS".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_size_8_not_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(8)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("international", token_filter);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 13,
            position: 0,
            text: "ANTRNXNL".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_non_convertable_strings_with_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(8)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, true).try_into()?;

        let result = token_stream_helper("12345 #$%@#^%&", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "12345".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 14,
                position: 1,
                text: "#$%@#^%&".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_non_convertable_strings_without_inject() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(8)), Alternate(true));

        let token_filter: PhoneticTokenFilter = (&algorithm, false).try_into()?;
        let result = token_stream_helper("12345 #$%@#^%&", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "12345".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 14,
                position: 1,
                text: "#$%@#^%&".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        let token_filter: PhoneticTokenFilter = (&algorithm, false).try_into()?;
        let result = token_stream_helper("12345 #$%@#^%& hello", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "12345".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 14,
                position: 1,
                text: "#$%@#^%&".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 20,
                position: 2,
                text: "HL".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_empty_term() -> Result<(), Error> {
        let algorithm = PhoneticAlgorithm::DoubleMetaphone(MaxCodeLength(Some(8)), Alternate(true));
        let token_filter: PhoneticTokenFilter = (algorithm, true).try_into()?;

        let result = token_stream_helper_raw("", token_filter);
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 0,
            position: 0,
            text: "".to_string(),
            position_length: 1,
        }];

        assert_eq!(result, expected);

        Ok(())
    }
}
