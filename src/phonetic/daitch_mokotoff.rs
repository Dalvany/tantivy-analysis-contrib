use std::collections::VecDeque;

use rphonetic::DaitchMokotoffSoundex;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct DaitchMokotoffTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: DaitchMokotoffSoundex,
    pub branching: bool,
    pub codes: VecDeque<String>,
    pub inject: bool,
}

impl<'a> TokenStream for DaitchMokotoffTokenStream<'a> {
    fn advance(&mut self) -> bool {
        while self.codes.is_empty() {
            let result = self.tail.advance();
            if !result {
                return false;
            }
            if self.tail.token().text.is_empty() {
                return true;
            }

            self.codes = self
                .encoder
                .inner_soundex(&self.tail.token().text, self.branching)
                .iter()
                .filter(|v| !v.is_empty())
                .cloned()
                .collect();

            if self.inject {
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
    use tantivy::tokenizer::Token;

    use crate::phonetic::tests::{token_stream_helper, token_stream_helper_raw};
    use crate::phonetic::{
        Branching, DMRule, Error, Folding, PhoneticAlgorithm, PhoneticTokenFilter,
    };

    const RULES: &str = include_str!("../../test_assets/dm-cc-rules/dmrules.txt");

    #[test]
    fn test_algorithms_inject() -> Result<(), Error> {
        #[cfg(feature = "embedded_dm")]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(Some(RULES.to_string())),
            Folding(true),
            Branching(true),
        );
        #[cfg(not(feature = "embedded_dm"))]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(RULES.to_string()),
            Folding(true),
            Branching(true),
        );

        let token_filter: PhoneticTokenFilter = (algorithm, true).try_into()?;
        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
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
                text: "000000".to_string(),
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
                text: "700000".to_string(),
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
                text: "400000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "450000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "454000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "540000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "545000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "500000".to_string(),
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
                text: "045450".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_algorithms_not_inject() -> Result<(), Error> {
        #[cfg(feature = "embedded_dm")]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(Some(RULES.to_string())),
            Folding(true),
            Branching(true),
        );
        #[cfg(not(feature = "embedded_dm"))]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(RULES.to_string()),
            Folding(true),
            Branching(true),
        );
        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

        let result = token_stream_helper("aaa bbb ccc easgasg", token_filter);
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "000000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "700000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "400000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "450000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "454000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "540000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "545000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "500000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 19,
                position: 3,
                text: "045450".to_string(),
                position_length: 1,
            },
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_empty_term() -> Result<(), Error> {
        #[cfg(feature = "embedded_dm")]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(Some(RULES.to_string())),
            Folding(true),
            Branching(true),
        );
        #[cfg(not(feature = "embedded_dm"))]
        let algorithm = PhoneticAlgorithm::DaitchMokotoffSoundex(
            DMRule(RULES.to_string()),
            Folding(true),
            Branching(true),
        );

        let token_filter: PhoneticTokenFilter = (algorithm, false).try_into()?;

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
