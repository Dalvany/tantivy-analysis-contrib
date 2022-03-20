use std::str::Chars;

use rust_icu::brk::UBreakIterator;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

/// Default rules, copy from Lucene's binary rules
const DEFAULT_RULES: &str = std::include_str!("breaking_rules/Default.rbbi");

/// Myanmar rules, copy from Lucene's binary rules
/*const MYANMAR_SYLLABLE_RULES: &str = std::include_str!("breaking_rules/MyanmarSyllable.rbbi");*/

struct ICUBreakingWord<'a> {
    text: Chars<'a>,
    default_breaking_iterator: UBreakIterator,
}

impl<'a> From<&'a str> for ICUBreakingWord<'a> {
    fn from(text: &'a str) -> Self {
        ICUBreakingWord {
            text: text.chars(),
            default_breaking_iterator: UBreakIterator::try_new_rules(DEFAULT_RULES, text)
                .expect("Can't read default rules."),
        }
    }
}

impl<'a> Iterator for ICUBreakingWord<'a> {
    type Item = (String, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut cont = true;
        let mut start = self.default_breaking_iterator.current();
        let mut end = self.default_breaking_iterator.next();
        while cont && end.is_some() {
            if end.is_some() && self.default_breaking_iterator.get_rule_status() == 0 {
                start = end.unwrap();
                end = self.default_breaking_iterator.next();
            }
            if let Some(index) = end {
                cont = !self
                    .text
                    .clone()
                    .take(index as usize)
                    .skip(start as usize)
                    .any(char::is_alphanumeric);
            }
        }

        match end {
            None => Option::None,
            Some(index) => {
                let substring: String = self
                    .text
                    .clone()
                    .take(index as usize)
                    .skip(start as usize)
                    .collect();
                Option::Some((substring, start as usize, index as usize))
            }
        }
    }
}

/// ICU [Tokenizer]. It does not (yet ?) work as Lucene's counterpart.
#[derive(Clone)]
pub struct ICUTokenizer;

impl Tokenizer for ICUTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        BoxTokenStream::from(ICUTokenizerTokenStream::new(text))
    }
}

/// ICU [TokenStream], it relies on [us::UnicodeWordIndices]
/// to do the actual tokenizing.
struct ICUTokenizerTokenStream<'a> {
    breaking_word: ICUBreakingWord<'a>,
    token: Token,
}

impl<'a> ICUTokenizerTokenStream<'a> {
    fn new(text: &'a str) -> Self {
        ICUTokenizerTokenStream {
            breaking_word: ICUBreakingWord::from(text),
            token: Token::default(),
        }
    }
}

impl<'a> TokenStream for ICUTokenizerTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let token = self.breaking_word.next();
        match token {
            None => false,
            Some(token) => {
                self.token.text.clear();
                self.token.position = self.token.position.wrapping_add(1);
                self.token.offset_from = token.1;
                self.token.offset_to = token.2;
                self.token.text.push_str(&token.0);
                true
            }
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests {
    /// Same tests as Lucene ICU tokenizer might be enough
    use super::*;

    impl<'a> Iterator for ICUTokenizerTokenStream<'a> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item> {
            if self.advance() {
                return Option::Some(self.token.clone());
            }

            Option::None
        }
    }

    #[test]
    fn test_huge_doc() {
        let mut huge_doc = " ".repeat(4094);
        huge_doc.push_str("testing 1234");
        let tokenizer = &mut ICUTokenizerTokenStream::new(huge_doc.as_str());
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 4094,
                offset_to: 4101,
                position: 0,
                text: "testing".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4102,
                offset_to: 4106,
                position: 1,
                text: "1234".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_armenian() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("Վիքիպեդիայի 13 միլիոն հոդվածները (4,600` հայերեն վիքիպեդիայում) գրվել են կամավորների կողմից ու համարյա բոլոր հոդվածները կարող է խմբագրել ցանկաց մարդ ով կարող է բացել Վիքիպեդիայի կայքը։");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 11,
                position: 0,
                text: "Վիքիպեդիայի".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 14,
                position: 1,
                text: "13".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 21,
                position: 2,
                text: "միլիոն".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 32,
                position: 3,
                text: "հոդվածները".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 39,
                position: 4,
                text: "4,600".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 41,
                offset_to: 48,
                position: 5,
                text: "հայերեն".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 49,
                offset_to: 62,
                position: 6,
                text: "վիքիպեդիայում".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 64,
                offset_to: 69,
                position: 7,
                text: "գրվել".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 70,
                offset_to: 72,
                position: 8,
                text: "են".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 73,
                offset_to: 84,
                position: 9,
                text: "կամավորների".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 85,
                offset_to: 91,
                position: 10,
                text: "կողմից".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 92,
                offset_to: 94,
                position: 11,
                text: "ու".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 95,
                offset_to: 102,
                position: 12,
                text: "համարյա".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 103,
                offset_to: 108,
                position: 13,
                text: "բոլոր".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 109,
                offset_to: 119,
                position: 14,
                text: "հոդվածները".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 120,
                offset_to: 125,
                position: 15,
                text: "կարող".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 126,
                offset_to: 127,
                position: 16,
                text: "է".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 128,
                offset_to: 136,
                position: 17,
                text: "խմբագրել".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 137,
                offset_to: 143,
                position: 18,
                text: "ցանկաց".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 144,
                offset_to: 148,
                position: 19,
                text: "մարդ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 149,
                offset_to: 151,
                position: 20,
                text: "ով".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 152,
                offset_to: 157,
                position: 21,
                text: "կարող".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 158,
                offset_to: 159,
                position: 22,
                text: "է".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 160,
                offset_to: 165,
                position: 23,
                text: "բացել".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 166,
                offset_to: 177,
                position: 24,
                text: "Վիքիպեդիայի".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 178,
                offset_to: 183,
                position: 25,
                text: "կայքը".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_amharic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "ዊኪፔድያ የባለ ብዙ ቋንቋ የተሟላ ትክክለኛና ነጻ መዝገበ ዕውቀት (ኢንሳይክሎፒዲያ) ነው። ማንኛውም",
        );
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "ዊኪፔድያ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "የባለ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "ብዙ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 16,
                position: 3,
                text: "ቋንቋ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 4,
                text: "የተሟላ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 28,
                position: 5,
                text: "ትክክለኛና".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 6,
                text: "ነጻ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 36,
                position: 7,
                text: "መዝገበ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 41,
                position: 8,
                text: "ዕውቀት".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 52,
                position: 9,
                text: "ኢንሳይክሎፒዲያ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 54,
                offset_to: 56,
                position: 10,
                text: "ነው".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 63,
                position: 11,
                text: "ማንኛውም".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_arabic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("الفيلم الوثائقي الأول عن ويكيبيديا يسمى \"الحقيقة بالأرقام: قصة ويكيبيديا\" (بالإنجليزية: Truth in Numbers: The Wikipedia Story)، سيتم إطلاقه في 2008.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "الفيلم".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 7,
                offset_to: 15,
                position: 1,
                text: "الوثائقي".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 21,
                position: 2,
                text: "الأول".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 24,
                position: 3,
                text: "عن".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 34,
                position: 4,
                text: "ويكيبيديا".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 35,
                offset_to: 39,
                position: 5,
                text: "يسمى".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 41,
                offset_to: 48,
                position: 6,
                text: "الحقيقة".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 49,
                offset_to: 57,
                position: 7,
                text: "بالأرقام".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 59,
                offset_to: 62,
                position: 8,
                text: "قصة".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 63,
                offset_to: 72,
                position: 9,
                text: "ويكيبيديا".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 75,
                offset_to: 86,
                position: 10,
                text: "بالإنجليزية".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 88,
                offset_to: 93,
                position: 11,
                text: "Truth".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 94,
                offset_to: 96,
                position: 12,
                text: "in".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 97,
                offset_to: 104,
                position: 13,
                text: "Numbers".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 106,
                offset_to: 109,
                position: 14,
                text: "The".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 110,
                offset_to: 119,
                position: 15,
                text: "Wikipedia".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 120,
                offset_to: 125,
                position: 16,
                text: "Story".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 128,
                offset_to: 132,
                position: 17,
                text: "سيتم".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 133,
                offset_to: 139,
                position: 18,
                text: "إطلاقه".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 140,
                offset_to: 142,
                position: 19,
                text: "في".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 143,
                offset_to: 147,
                position: 20,
                text: "2008".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_aramaic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ܘܝܩܝܦܕܝܐ (ܐܢܓܠܝܐ: Wikipedia) ܗܘ ܐܝܢܣܩܠܘܦܕܝܐ ܚܐܪܬܐ ܕܐܢܛܪܢܛ ܒܠܫܢ̈ܐ ܣܓܝܐ̈ܐ܂ ܫܡܗ ܐܬܐ ܡܢ ܡ̈ܠܬܐ ܕ\"ܘܝܩܝ\" ܘ\"ܐܝܢܣܩܠܘܦܕܝܐ\"܀");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "ܘܝܩܝܦܕܝܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 16,
                position: 1,
                text: "ܐܢܓܠܝܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 18,
                offset_to: 27,
                position: 2,
                text: "Wikipedia".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 3,
                text: "ܗܘ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 43,
                position: 4,
                text: "ܐܝܢܣܩܠܘܦܕܝܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 44,
                offset_to: 49,
                position: 5,
                text: "ܚܐܪܬܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 50,
                offset_to: 57,
                position: 6,
                text: "ܕܐܢܛܪܢܛ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 64,
                position: 7,
                text: "ܒܠܫܢ̈ܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 65,
                offset_to: 71,
                position: 8,
                text: "ܣܓܝܐ̈ܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 73,
                offset_to: 76,
                position: 9,
                text: "ܫܡܗ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 77,
                offset_to: 80,
                position: 10,
                text: "ܐܬܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 83,
                position: 11,
                text: "ܡܢ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 84,
                offset_to: 89,
                position: 12,
                text: "ܡ̈ܠܬܐ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 90,
                offset_to: 91,
                position: 13,
                text: "ܕ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 92,
                offset_to: 96,
                position: 14,
                text: "ܘܝܩܝ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 98,
                offset_to: 99,
                position: 15,
                text: "ܘ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 100,
                offset_to: 111,
                position: 16,
                text: "ܐܝܢܣܩܠܘܦܕܝܐ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bengali() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("এই বিশ্বকোষ পরিচালনা করে উইকিমিডিয়া ফাউন্ডেশন (একটি অলাভজনক সংস্থা)। উইকিপিডিয়ার শুরু ১৫ জানুয়ারি, ২০০১ সালে। এখন পর্যন্ত ২০০টিরও বেশী ভাষায় উইকিপিডিয়া রয়েছে।");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "এই".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 11,
                position: 1,
                text: "বিশ্বকোষ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 20,
                position: 2,
                text: "পরিচালনা".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 21,
                offset_to: 24,
                position: 3,
                text: "করে".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 36,
                position: 4,
                text: "উইকিমিডিয়া".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 46,
                position: 5,
                text: "ফাউন্ডেশন".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 48,
                offset_to: 52,
                position: 6,
                text: "একটি".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 60,
                position: 7,
                text: "অলাভজনক".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 61,
                offset_to: 67,
                position: 8,
                text: "সংস্থা".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 70,
                offset_to: 82,
                position: 9,
                text: "উইকিপিডিয়ার".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 83,
                offset_to: 87,
                position: 10,
                text: "শুরু".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 88,
                offset_to: 90,
                position: 11,
                text: "১৫".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 91,
                offset_to: 100,
                position: 12,
                text: "জানুয়ারি".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 102,
                offset_to: 106,
                position: 13,
                text: "২০০১".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 107,
                offset_to: 111,
                position: 14,
                text: "সালে".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 113,
                offset_to: 116,
                position: 15,
                text: "এখন".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 117,
                offset_to: 124,
                position: 16,
                text: "পর্যন্ত".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 125,
                offset_to: 132,
                position: 17,
                text: "২০০টিরও".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 133,
                offset_to: 137,
                position: 18,
                text: "বেশী".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 138,
                offset_to: 144,
                position: 19,
                text: "ভাষায়".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 145,
                offset_to: 156,
                position: 20,
                text: "উইকিপিডিয়া".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 157,
                offset_to: 163,
                position: 21,
                text: "রয়েছে".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_farsi() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ویکی پدیای انگلیسی در تاریخ ۲۵ دی ۱۳۷۹ به صورت مکملی برای دانشنامهٔ تخصصی نوپدیا نوشته شد.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "ویکی".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 10,
                position: 1,
                text: "پدیای".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 18,
                position: 2,
                text: "انگلیسی".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 19,
                offset_to: 21,
                position: 3,
                text: "در".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 27,
                position: 4,
                text: "تاریخ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 28,
                offset_to: 30,
                position: 5,
                text: "۲۵".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 31,
                offset_to: 33,
                position: 6,
                text: "دی".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 38,
                position: 7,
                text: "۱۳۷۹".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 39,
                offset_to: 41,
                position: 8,
                text: "به".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 42,
                offset_to: 46,
                position: 9,
                text: "صورت".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 47,
                offset_to: 52,
                position: 10,
                text: "مکملی".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 57,
                position: 11,
                text: "برای".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 67,
                position: 12,
                text: "دانشنامهٔ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 68,
                offset_to: 73,
                position: 13,
                text: "تخصصی".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 74,
                offset_to: 80,
                position: 14,
                text: "نوپدیا".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 86,
                position: 15,
                text: "نوشته".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 87,
                offset_to: 89,
                position: 16,
                text: "شد".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_greek() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("Γράφεται σε συνεργασία από εθελοντές με το λογισμικό wiki, κάτι που σημαίνει ότι άρθρα μπορεί να προστεθούν ή να αλλάξουν από τον καθένα.");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 8,
                position: 0,
                text: "Γράφεται".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 11,
                position: 1,
                text: "σε".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 22,
                position: 2,
                text: "συνεργασία".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 26,
                position: 3,
                text: "από".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 27,
                offset_to: 36,
                position: 4,
                text: "εθελοντές".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 37,
                offset_to: 39,
                position: 5,
                text: "με".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 42,
                position: 6,
                text: "το".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 52,
                position: 7,
                text: "λογισμικό".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 57,
                position: 8,
                text: "wiki".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 59,
                offset_to: 63,
                position: 9,
                text: "κάτι".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 64,
                offset_to: 67,
                position: 10,
                text: "που".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 68,
                offset_to: 76,
                position: 11,
                text: "σημαίνει".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 77,
                offset_to: 80,
                position: 12,
                text: "ότι".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 81,
                offset_to: 86,
                position: 13,
                text: "άρθρα".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 87,
                offset_to: 93,
                position: 14,
                text: "μπορεί".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 94,
                offset_to: 96,
                position: 15,
                text: "να".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 97,
                offset_to: 107,
                position: 16,
                text: "προστεθούν".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 108,
                offset_to: 109,
                position: 17,
                text: "ή".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 110,
                offset_to: 112,
                position: 18,
                text: "να".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 113,
                offset_to: 121,
                position: 19,
                text: "αλλάξουν".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 122,
                offset_to: 125,
                position: 20,
                text: "από".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 126,
                offset_to: 129,
                position: 21,
                text: "τον".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 130,
                offset_to: 136,
                position: 22,
                text: "καθένα".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_khmer() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ផ្ទះស្កឹមស្កៃបីបួនខ្នងនេះ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "ផ្ទះ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 13,
                position: 1,
                text: "ស្កឹមស្កៃ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 15,
                position: 2,
                text: "បី".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 18,
                position: 3,
                text: "បួន".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 18,
                offset_to: 22,
                position: 4,
                text: "ខ្នង".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 25,
                position: 5,
                text: "នេះ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lao() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ກວ່າດອກ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "ກວ່າ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "ດອກ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("ພາສາລາວ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "ພາສາ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "ລາວ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_myanmar() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("သက်ဝင်လှုပ်ရှားစေပြီး");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 6,
                position: 0,
                text: "သက်ဝင်".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 15,
                position: 1,
                text: "လှုပ်ရှား".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 17,
                position: 2,
                text: "စေ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 21,
                position: 3,
                text: "ပြီး".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_thai() {
        let tokenizer =
            &mut ICUTokenizerTokenStream::new("การที่ได้ต้องแสดงว่างานดี. แล้วเธอจะไปไหน? ๑๒๓๔");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "การ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 6,
                position: 1,
                text: "ที่".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 2,
                text: "ได้".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 13,
                position: 3,
                text: "ต้อง".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 17,
                position: 4,
                text: "แสดง".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 17,
                offset_to: 20,
                position: 5,
                text: "ว่า".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 20,
                offset_to: 23,
                position: 6,
                text: "งาน".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 25,
                position: 7,
                text: "ดี".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 27,
                offset_to: 31,
                position: 8,
                text: "แล้ว".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 31,
                offset_to: 34,
                position: 9,
                text: "เธอ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 34,
                offset_to: 36,
                position: 10,
                text: "จะ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 36,
                offset_to: 38,
                position: 11,
                text: "ไป".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 38,
                offset_to: 41,
                position: 12,
                text: "ไหน".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 43,
                offset_to: 47,
                position: 13,
                text: "๑๒๓๔".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tibetan() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "སྣོན་མཛོད་དང་ལས་འདིས་བོད་ཡིག་མི་ཉམས་གོང་འཕེལ་དུ་གཏོང་བར་ཧ་ཅང་དགེ་མཚན་མཆིས་སོ། །",
        );
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "སྣོན".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 1,
                text: "མཛོད".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "དང".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 15,
                position: 3,
                text: "ལས".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 20,
                position: 4,
                text: "འདིས".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 21,
                offset_to: 24,
                position: 5,
                text: "བོད".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 25,
                offset_to: 28,
                position: 6,
                text: "ཡིག".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 29,
                offset_to: 31,
                position: 7,
                text: "མི".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 32,
                offset_to: 35,
                position: 8,
                text: "ཉམས".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 36,
                offset_to: 39,
                position: 9,
                text: "གོང".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 40,
                offset_to: 44,
                position: 10,
                text: "འཕེལ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 45,
                offset_to: 47,
                position: 11,
                text: "དུ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 48,
                offset_to: 52,
                position: 12,
                text: "གཏོང".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 53,
                offset_to: 55,
                position: 13,
                text: "བར".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 56,
                offset_to: 57,
                position: 14,
                text: "ཧ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 58,
                offset_to: 60,
                position: 15,
                text: "ཅང".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 61,
                offset_to: 64,
                position: 16,
                text: "དགེ".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 65,
                offset_to: 68,
                position: 17,
                text: "མཚན".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 69,
                offset_to: 73,
                position: 18,
                text: "མཆིས".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 74,
                offset_to: 76,
                position: 19,
                text: "སོ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chinese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("我是中国人。 １２３４ Ｔｅｓｔｓ ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "我".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 2,
                position: 1,
                text: "是".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 2,
                text: "中".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 3,
                text: "国".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 5,
                position: 4,
                text: "人".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 7,
                offset_to: 11,
                position: 5,
                text: "１２３４".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 17,
                position: 6,
                text: "Ｔｅｓｔｓ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hebrew() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("דנקנר תקף את הדו\"ח");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "דנקנר".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "תקף".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 12,
                position: 2,
                text: "את".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 13,
                offset_to: 18,
                position: 3,
                text: "הדו\"ח".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("חברת בת של מודי'ס");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "חברת".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 1,
                text: "בת".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 10,
                position: 2,
                text: "של".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 17,
                position: 3,
                text: "מודי'ס".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty() {
        let expected: Vec<Token> = vec![];

        let tokenizer = &mut ICUTokenizerTokenStream::new("");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(".");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(" ");
        let result: Vec<Token> = tokenizer.collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lucene1545() {
        /*
         * Standard analyzer does not correctly tokenize combining character U+0364 COMBINING LATIN SMALL LETTRE E.
         * The word "moͤchte" is incorrectly tokenized into "mo" "chte", the combining character is lost.
         * Expected result is only on token "moͤchte".
         */
        let tokenizer = &mut ICUTokenizerTokenStream::new("moͤchte");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 7,
            position: 0,
            text: "moͤchte".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphanumeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("B2B");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "B2B".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("2B");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 2,
            position: 0,
            text: "2B".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_delimiters_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("some-dashed-phrase");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "some".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 11,
                position: 1,
                text: "dashed".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 18,
                position: 2,
                text: "phrase".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("dogs,chase,cats");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "dogs".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 10,
                position: 1,
                text: "chase".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 15,
                position: 2,
                text: "cats".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("ac/dc");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "ac".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "dc".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apostrophes_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 8,
            position: 0,
            text: "O'Reilly".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("you're");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 6,
            position: 0,
            text: "you're".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("she's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "she's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("Jim's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "Jim's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("don't");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "don't".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly's");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 10,
            position: 0,
            text: "O'Reilly's".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("21.35");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 5,
            position: 0,
            text: "21.35".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("R2D2 C3PO");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "R2D2".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 1,
                text: "C3PO".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("216.239.63.104");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 14,
            position: 0,
            text: "216.239.63.104".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_text_with_numbers_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("David has 5000 bones");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "David".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "has".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 14,
                position: 2,
                text: "5000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 20,
                position: 3,
                text: "bones".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_various_text_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("C embedded developers wanted");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "C".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 10,
                position: 1,
                text: "embedded".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 11,
                offset_to: 21,
                position: 2,
                text: "developers".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 22,
                offset_to: 28,
                position: 3,
                text: "wanted".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo bar FOO BAR");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "foo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 7,
                position: 1,
                text: "bar".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 8,
                offset_to: 11,
                position: 2,
                text: "FOO".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 12,
                offset_to: 15,
                position: 3,
                text: "BAR".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo      bar .  FOO <> BAR");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "foo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 12,
                position: 1,
                text: "bar".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 16,
                offset_to: 19,
                position: 2,
                text: "FOO".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 23,
                offset_to: 26,
                position: 3,
                text: "BAR".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("\"QUOTED\" word");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 1,
                offset_to: 7,
                position: 0,
                text: "QUOTED".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 9,
                offset_to: 13,
                position: 1,
                text: "word".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_korean_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("안녕하세요 한글입니다");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "안녕하세요".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 11,
                position: 1,
                text: "한글입니다".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_offsets() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("David has 5000 bones");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 5,
                position: 0,
                text: "David".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 6,
                offset_to: 9,
                position: 1,
                text: "has".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 10,
                offset_to: 14,
                position: 2,
                text: "5000".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 15,
                offset_to: 20,
                position: 3,
                text: "bones".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_korean() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("훈민정음");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 4,
            position: 0,
            text: "훈민정음".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_japanese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("仮名遣い カタカナ");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 1,
                position: 0,
                text: "仮".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 1,
                offset_to: 2,
                position: 1,
                text: "名".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 2,
                text: "遣".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 3,
                text: "い".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 9,
                position: 4,
                text: "カタカナ".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("💩 💩💩");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "💩".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "💩".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 7,
                position: 2,
                text: "💩".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("👩‍❤️‍👩");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 8,
            position: 0,
            text: "👩‍❤️‍👩".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence_with_modifier() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("👨🏼‍⚕️");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 7,
            position: 0,
            text: "👨🏼‍⚕️".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_regional_indicator() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("🇺🇸🇺🇸");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 4,
                position: 0,
                text: "🇺🇸".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 8,
                position: 1,
                text: "🇺🇸".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_variation_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("#️⃣");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 3,
            position: 0,
            text: "3️⃣".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tag_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("🏴󠁧󠁢󠁥󠁮󠁧󠁿");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![Token {
            offset_from: 0,
            offset_to: 14,
            position: 0,
            text: "🏴󠁧󠁢󠁥󠁮󠁧󠁿".to_string(),
            position_length: 1,
        }];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tokenization() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("poo💩poo");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 3,
                position: 0,
                text: "poo".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 5,
                position: 1,
                text: "💩".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 5,
                offset_to: 8,
                position: 2,
                text: "poo".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("💩中國💩");
        let result: Vec<Token> = tokenizer.collect();
        let expected = vec![
            Token {
                offset_from: 0,
                offset_to: 2,
                position: 0,
                text: "💩".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 2,
                offset_to: 3,
                position: 1,
                text: "中".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 3,
                offset_to: 4,
                position: 2,
                text: "國".to_string(),
                position_length: 1,
            },
            Token {
                offset_from: 4,
                offset_to: 6,
                position: 3,
                text: "💩".to_string(),
                position_length: 1,
            },
        ];
        assert_eq!(result, expected);
    }
}
