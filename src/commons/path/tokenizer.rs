use super::PathTokenStream;
use super::DEFAULT_SEPARATOR;
use either::Either;
use std::iter::Rev;
use std::str::Split;
use tantivy_tokenizer_api::Tokenizer;

/// Path tokenizer. It will tokenize this :
/// ```norust
/// /part1/part2/part3
/// ```
/// into
/// ```norust
/// /part1
/// /part1/part2
/// /part1/part2/part3
/// ```
///
/// Enabling `reverse` will make this tokenizer to behave like Lucene's except that tokens will not be ordered the same way. See
/// [ReversePathHierarchyTokenizer](https://lucene.apache.org/core/9_1_0/analysis/common/org/apache/lucene/analysis/path/ReversePathHierarchyTokenizer.html)
///
/// # Warning
/// To construct a new [PathTokenizer] you should use the [PathTokenizerBuilder] or the [Default] implementation as
/// [From] trait will probably be removed.
///
/// # Examples
///
/// Here is an example with `reverse` set to `false` and use `\` as character separator. It will also skip the first token.
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::{PathTokenizer, PathTokenizerBuilder};
///
/// let path_tokenizer = PathTokenizerBuilder::default()
///    .skip(1_usize)
///    .delimiter('\\')
///    .build()?;
///
/// let mut tmp = TextAnalyzer::builder(path_tokenizer).build();
/// let mut token_stream = tmp.token_stream("c:\\a\\b\\c");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "\\a".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "\\a\\b".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "\\a\\b\\c".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
///
/// This second example shows what tokens are produced if `reverse` is set to `true` and what does `replacement` parameter.
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tantivy::tokenizer::{TextAnalyzer, Token};
/// use tantivy_analysis_contrib::commons::{PathTokenizer, PathTokenizerBuilder};
///
/// let path_tokenizer = PathTokenizerBuilder::default()
///    .delimiter('\\')
///    .replacement('/')
///    .reverse(true)
///    .build()?;
///
/// let mut tmp = TextAnalyzer::builder(path_tokenizer).build();
/// let mut token_stream = tmp.token_stream("c:\\a\\b\\c");
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "c".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "b/c".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "a/b/c".to_string());
///
/// let token = token_stream.next().expect("A token should be present.");
/// assert_eq!(token.text, "c:/a/b/c".to_string());
///
/// assert_eq!(None, token_stream.next());
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Builder)]
#[builder(setter(into), default)]
pub struct PathTokenizer {
    /// Do the tokenization backward.
    /// ```norust
    /// mail.google.com
    /// ```
    /// into
    /// ```norust
    /// com
    /// google.com
    /// mail.google.com
    /// ```
    #[builder(default = "false")]
    pub reverse: bool,
    /// Number of parts to skip.
    #[builder(default = "0")]
    pub skip: usize,
    /// Delimiter of path parts
    /// In the following exemple, delimiter is the `/` character :
    /// ```norust
    /// /part1/part2/part3
    /// ```
    #[builder(default = "DEFAULT_SEPARATOR")]
    pub delimiter: char,
    /// Character that replaces delimiter for generated parts.
    /// If [None] then the same char as delimiter will be used.
    /// For example, if delimiter is `/` and replacement is `|`
    /// ```norust
    /// /part1/part2/part3
    /// ```
    /// will generate
    /// ```norust
    /// |part1
    /// |part1|part2
    /// |part1|part2|part3
    /// ```
    pub replacement: Option<char>,
}

impl Default for PathTokenizer {
    /// Construct a [PathTokenizer] with no skip and
    /// `/` as delimiter and replacement.
    fn default() -> Self {
        PathTokenizer {
            reverse: false,
            skip: 0,
            delimiter: DEFAULT_SEPARATOR,
            replacement: None,
        }
    }
}

impl Tokenizer for PathTokenizer {
    type TokenStream<'a> = PathTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let mut offset = 0;
        let mut starts_with = if self.reverse {
            text.ends_with(self.delimiter)
        } else {
            text.starts_with(self.delimiter)
        };
        let split = text.split(self.delimiter);
        let split: Either<Split<char>, Rev<Split<char>>> = if self.reverse {
            Either::Right(split.rev())
        } else {
            Either::Left(split)
        };

        let skip = if starts_with { 1 } else { 0 };

        let mut split = split.skip(skip);
        let mut i = self.skip;
        while i > 0 {
            if let Some(token) = split.next() {
                if starts_with {
                    offset += 1;
                } else {
                    starts_with = true;
                }
                offset += token.len();
            }
            i -= 1;
        }

        if self.reverse {
            offset = text.len() - offset;
        }

        PathTokenStream {
            text: split,
            buffer: String::with_capacity(text.len()),
            token: Default::default(),
            separator: self.replacement.unwrap_or(self.delimiter),
            offset,
            starts_with,
            reverse: self.reverse,
        }
    }
}
