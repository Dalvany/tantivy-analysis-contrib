use rphonetic::Encoder;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream};

pub struct GenericPhoneticTokenStream<'a> {
    pub tail: BoxTokenStream<'a>,
    pub encoder: Box<dyn Encoder>,
}

impl<'a> TokenStream for GenericPhoneticTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let mut result = false;

        // We while skip empty code
        while !result {
            let tail_result = self.tail.advance();
            // If end of stream, return false, this will end the loop
            if !tail_result {
                return false;
            }
            let token = self.encoder.encode(&self.tail.token().text);

            // If encoded value is not empty, then udate token in tail, and stop the loop
            if !token.is_empty() {
                self.tail.token_mut().text = token;
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
