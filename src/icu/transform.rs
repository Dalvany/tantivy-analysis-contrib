use std::error::Error;
use rust_icu_sys as sys;
use rust_icu_utrans as utrans;
use std::mem;
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
        let _ = utrans::UTransliterator::new(&compound_id, rules.as_ref().map(|x| &*x).map(|x| &**x), direction)?;
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
            transform: utrans::UTransliterator::new(&self.compound_id, self.rules.as_ref().map(|x| &*x).map(|x| &**x), self.direction).unwrap(),
            tail: token_stream,
            temp: String::with_capacity(100),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_token_stream() {}
}