use std::error::Error;
use std::mem;

use rust_icu::sys as sys;
use rust_icu::trans as utrans;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenFilter, TokenStream};

struct ICUTransformTokenStream<'a> {
    transform: utrans::UTransliterator,
    tail: BoxTokenStream<'a>,
    temp: String,
}

impl<'a> TokenStream for ICUTransformTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let result = self.tail.advance();
        if !result {
            return result;
        }
        if let Ok(t) = self.transform.transliterate(&self.tail.token().text) {
            self.temp = t;
            mem::swap(&mut self.tail.token_mut().text, &mut self.temp);
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

#[derive(Clone, Debug)]
pub struct ICUTransformTokenFilter {
    compound_id: String,
    rules: Option<String>,
    direction: sys::UTransDirection,
}


impl ICUTransformTokenFilter {
    pub fn new(compound_id: String, rules: Option<String>, direction: sys::UTransDirection) -> Result<Self, Box<dyn Error>> {
        let _ = utrans::UTransliterator::new(compound_id.as_str(), rules.as_ref().map(|x| x.as_str()), direction)?;
        Ok(ICUTransformTokenFilter {
            compound_id,
            rules,
            direction,
        })
    }
}


impl TokenFilter for ICUTransformTokenFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(ICUTransformTokenStream {
            // unwrap work, we checked in new method.
            transform: utrans::UTransliterator::new(self.compound_id.as_str(), self.rules.as_ref().map(|x| x.as_str()), self.direction).unwrap(),
            tail: token_stream,
            temp: String::with_capacity(100),
        })
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use super::*;

    fn token_stream_helper(text: &str, compound_id: &str, rules: Option<String>, direction: sys::UTransDirection) -> Vec<Token> {
        let mut token_stream = TextAnalyzer::from(SimpleTokenizer)
            .filter(ICUTransformTokenFilter::new(String::from(compound_id), rules, direction).unwrap())
            .token_stream(text);
        let mut tokens = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }

    #[test]
    fn test_transform_filter_1() {
        let tokens = token_stream_helper("Joséphine Baker", "NFD; [:Nonspacing Mark:] Remove; Lower;  NFC", None, sys::UTransDirection::UTRANS_FORWARD);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[0].text, "josephine");
        assert_eq!(tokens[0].offset_from, 0);
        assert_eq!(tokens[0].offset_to, 10);

        assert_eq!(tokens[1].position, 1);
        assert_eq!(tokens[1].text, "baker");
        assert_eq!(tokens[1].offset_from, 11);
        assert_eq!(tokens[1].offset_to, 16);
    }

    #[test]
    fn test_transform_filter_2() {
        let tokens = token_stream_helper("Русский текст", "Any-Latin;", None, sys::UTransDirection::UTRANS_FORWARD);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[0].text, "Russkij");
        assert_eq!(tokens[0].offset_from, 0);
        assert_eq!(tokens[0].offset_to, 14);

        assert_eq!(tokens[1].position, 1);
        assert_eq!(tokens[1].text, "tekst");
        assert_eq!(tokens[1].offset_from, 15);
        assert_eq!(tokens[1].offset_to, 25);
    }
}