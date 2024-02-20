use super::ICUTokenizerTokenStream;
use tantivy_tokenizer_api::Tokenizer;

/// ICU [Tokenizer]. It does not (yet ?) work as Lucene's counterpart.
/// Getting a tokenizer is simple :
/// ```rust
/// use tantivy_analysis_contrib::icu::ICUTokenizer;
///
/// let tokenizer = ICUTokenizer;
/// ```
///
/// # Example
///
/// Here is an example of a tokenization result
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{TextAnalyzer, Token};
/// use tantivy_analysis_contrib::icu::ICUTokenizer;
///
/// let mut tmp = TextAnalyzer::builder(ICUTokenizer::default()).build();
/// let mut token_stream = tmp.token_stream("我是中国人。 １２３４ Ｔｅｓｔｓ ");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "我".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "是".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "中".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "国".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "人".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "１２３４".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "Ｔｅｓｔｓ".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
#[derive(Clone, Copy, Debug, Default)]
pub struct ICUTokenizer;

impl Tokenizer for ICUTokenizer {
    type TokenStream<'a> = ICUTokenizerTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        ICUTokenizerTokenStream::new(text)
    }
}
