#![feature(test)]
use rust_icu::brk::UBreakIterator;
use std::str::Chars;
use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

/// Default rules, copy from Lucene's binary rules
const DEFAULT_RULES: &str = std::include_str!("breaking_rules/Default.rbbi");
/// Myanmar rules, copy from Lucene's binary rules
const MYANMAR_SYLLABLE_RULES: &str = std::include_str!("breaking_rules/MyanmarSyllable.rbbi");

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
pub struct ICUTokenizerTokenStream<'a> {
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
    /// TODO non working tests see other unicode library google's rust_icu or icu4x.
    /// TODO run Lucene tests to get offsets so that they can be checked here.
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
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "Վիքիպեդիայի".to_string(),
            "13".to_string(),
            "միլիոն".to_string(),
            "հոդվածները".to_string(),
            "4,600".to_string(),
            "հայերեն".to_string(),
            "վիքիպեդիայում".to_string(),
            "գրվել".to_string(),
            "են".to_string(),
            "կամավորների".to_string(),
            "կողմից".to_string(),
            "ու".to_string(),
            "համարյա".to_string(),
            "բոլոր".to_string(),
            "հոդվածները".to_string(),
            "կարող".to_string(),
            "է".to_string(),
            "խմբագրել".to_string(),
            "ցանկաց".to_string(),
            "մարդ".to_string(),
            "ով".to_string(),
            "կարող".to_string(),
            "է".to_string(),
            "բացել".to_string(),
            "Վիքիպեդիայի".to_string(),
            "կայքը".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_amharic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "ዊኪፔድያ የባለ ብዙ ቋንቋ የተሟላ ትክክለኛና ነጻ መዝገበ ዕውቀት (ኢንሳይክሎፒዲያ) ነው። ማንኛውም",
        );
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "ዊኪፔድያ".to_string(),
            "የባለ".to_string(),
            "ብዙ".to_string(),
            "ቋንቋ".to_string(),
            "የተሟላ".to_string(),
            "ትክክለኛና".to_string(),
            "ነጻ".to_string(),
            "መዝገበ".to_string(),
            "ዕውቀት".to_string(),
            "ኢንሳይክሎፒዲያ".to_string(),
            "ነው".to_string(),
            "ማንኛውም".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_arabic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("الفيلم الوثائقي الأول عن ويكيبيديا يسمى \"الحقيقة بالأرقام: قصة ويكيبيديا\" (بالإنجليزية: Truth in Numbers: The Wikipedia Story)، سيتم إطلاقه في 2008.");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "الفيلم".to_string(),
            "الوثائقي".to_string(),
            "الأول".to_string(),
            "عن".to_string(),
            "ويكيبيديا".to_string(),
            "يسمى".to_string(),
            "الحقيقة".to_string(),
            "بالأرقام".to_string(),
            "قصة".to_string(),
            "ويكيبيديا".to_string(),
            "بالإنجليزية".to_string(),
            "Truth".to_string(),
            "in".to_string(),
            "Numbers".to_string(),
            "The".to_string(),
            "Wikipedia".to_string(),
            "Story".to_string(),
            "سيتم".to_string(),
            "إطلاقه".to_string(),
            "في".to_string(),
            "2008".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_aramaic() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ܘܝܩܝܦܕܝܐ (ܐܢܓܠܝܐ: Wikipedia) ܗܘ ܐܝܢܣܩܠܘܦܕܝܐ ܚܐܪܬܐ ܕܐܢܛܪܢܛ ܒܠܫܢ̈ܐ ܣܓܝܐ̈ܐ܂ ܫܡܗ ܐܬܐ ܡܢ ܡ̈ܠܬܐ ܕ\"ܘܝܩܝ\" ܘ\"ܐܝܢܣܩܠܘܦܕܝܐ\"܀");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "ܘܝܩܝܦܕܝܐ".to_string(),
            "ܐܢܓܠܝܐ".to_string(),
            "Wikipedia".to_string(),
            "ܗܘ".to_string(),
            "ܐܝܢܣܩܠܘܦܕܝܐ".to_string(),
            "ܚܐܪܬܐ".to_string(),
            "ܕܐܢܛܪܢܛ".to_string(),
            "ܒܠܫܢ̈ܐ".to_string(),
            "ܣܓܝܐ̈ܐ".to_string(),
            "ܫܡܗ".to_string(),
            "ܐܬܐ".to_string(),
            "ܡܢ".to_string(),
            "ܡ̈ܠܬܐ".to_string(),
            "ܕ".to_string(),
            "ܘܝܩܝ".to_string(),
            "ܘ".to_string(),
            "ܐܝܢܣܩܠܘܦܕܝܐ".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bengali() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("এই বিশ্বকোষ পরিচালনা করে উইকিমিডিয়া ফাউন্ডেশন (একটি অলাভজনক সংস্থা)। উইকিপিডিয়ার শুরু ১৫ জানুয়ারি, ২০০১ সালে। এখন পর্যন্ত ২০০টিরও বেশী ভাষায় উইকিপিডিয়া রয়েছে।");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "এই".to_string(),
            "বিশ্বকোষ".to_string(),
            "পরিচালনা".to_string(),
            "করে".to_string(),
            "উইকিমিডিয়া".to_string(),
            "ফাউন্ডেশন".to_string(),
            "একটি".to_string(),
            "অলাভজনক".to_string(),
            "সংস্থা".to_string(),
            "উইকিপিডিয়ার".to_string(),
            "শুরু".to_string(),
            "১৫".to_string(),
            "জানুয়ারি".to_string(),
            "২০০১".to_string(),
            "সালে".to_string(),
            "এখন".to_string(),
            "পর্যন্ত".to_string(),
            "২০০টিরও".to_string(),
            "বেশী".to_string(),
            "ভাষায়".to_string(),
            "উইকিপিডিয়া".to_string(),
            "রয়েছে".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_farsi() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ویکی پدیای انگلیسی در تاریخ ۲۵ دی ۱۳۷۹ به صورت مکملی برای دانشنامهٔ تخصصی نوپدیا نوشته شد.");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "ویکی".to_string(),
            "پدیای".to_string(),
            "انگلیسی".to_string(),
            "در".to_string(),
            "تاریخ".to_string(),
            "۲۵".to_string(),
            "دی".to_string(),
            "۱۳۷۹".to_string(),
            "به".to_string(),
            "صورت".to_string(),
            "مکملی".to_string(),
            "برای".to_string(),
            "دانشنامهٔ".to_string(),
            "تخصصی".to_string(),
            "نوپدیا".to_string(),
            "نوشته".to_string(),
            "شد".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_greek() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("Γράφεται σε συνεργασία από εθελοντές με το λογισμικό wiki, κάτι που σημαίνει ότι άρθρα μπορεί να προστεθούν ή να αλλάξουν από τον καθένα.");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "Γράφεται".to_string(),
            "σε".to_string(),
            "συνεργασία".to_string(),
            "από".to_string(),
            "εθελοντές".to_string(),
            "με".to_string(),
            "το".to_string(),
            "λογισμικό".to_string(),
            "wiki".to_string(),
            "κάτι".to_string(),
            "που".to_string(),
            "σημαίνει".to_string(),
            "ότι".to_string(),
            "άρθρα".to_string(),
            "μπορεί".to_string(),
            "να".to_string(),
            "προστεθούν".to_string(),
            "ή".to_string(),
            "να".to_string(),
            "αλλάξουν".to_string(),
            "από".to_string(),
            "τον".to_string(),
            "καθένα".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_khmer() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ផ្ទះស្កឹមស្កៃបីបួនខ្នងនេះ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "ផ្ទះ".to_string(),
            "ស្កឹមស្កៃ".to_string(),
            "បី".to_string(),
            "បួន".to_string(),
            "ខ្នង".to_string(),
            "នេះ".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_lao() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("ກວ່າດອກ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["ກວ່າ".to_string(), "ດອກ".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("ພາສາລາວ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["ພາສາ".to_string(), "ລາວ".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_myanmar() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("သက်ဝင်လှုပ်ရှားစေပြီး");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "သက်ဝင်".to_string(),
            "လှုပ်ရှား".to_string(),
            "စေ".to_string(),
            "ပြီး".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_thai() {
        let tokenizer =
            &mut ICUTokenizerTokenStream::new("การที่ได้ต้องแสดงว่างานดี. แล้วเธอจะไปไหน? ๑๒๓๔");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "การ".to_string(),
            "ที่".to_string(),
            "ได้".to_string(),
            "ต้อง".to_string(),
            "แสดง".to_string(),
            "ว่า".to_string(),
            "งาน".to_string(),
            "ดี".to_string(),
            "แล้ว".to_string(),
            "เธอ".to_string(),
            "จะ".to_string(),
            "ไป".to_string(),
            "ไหน".to_string(),
            "๑๒๓๔".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tibetan() {
        let tokenizer = &mut ICUTokenizerTokenStream::new(
            "སྣོན་མཛོད་དང་ལས་འདིས་བོད་ཡིག་མི་ཉམས་གོང་འཕེལ་དུ་གཏོང་བར་ཧ་ཅང་དགེ་མཚན་མཆིས་སོ། །",
        );
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "སྣོན".to_string(),
            "མཛོད".to_string(),
            "དང".to_string(),
            "ལས".to_string(),
            "འདིས".to_string(),
            "བོད".to_string(),
            "ཡིག".to_string(),
            "མི".to_string(),
            "ཉམས".to_string(),
            "གོང".to_string(),
            "འཕེལ".to_string(),
            "དུ".to_string(),
            "གཏོང".to_string(),
            "བར".to_string(),
            "ཧ".to_string(),
            "ཅང".to_string(),
            "དགེ".to_string(),
            "མཚན".to_string(),
            "མཆིས".to_string(),
            "སོ".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chinese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("我是中国人。 １２３４ Ｔｅｓｔｓ ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "我".to_string(),
            "是".to_string(),
            "中".to_string(),
            "国".to_string(),
            "人".to_string(),
            "１２３４".to_string(),
            "Ｔｅｓｔｓ".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hebrew() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("דנקנר תקף את הדו\"ח");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "דנקנר".to_string(),
            "תקף".to_string(),
            "את".to_string(),
            "הדו\"ח".to_string(),
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("חברת בת של מודי'ס");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "חברת".to_string(),
            "בת".to_string(),
            "של".to_string(),
            "מודי'ס".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty() {
        let expected: Vec<String> = vec![];

        let tokenizer = &mut ICUTokenizerTokenStream::new("");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(".");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new(" ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
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
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["moͤchte".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alphanumeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("B2B");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["B2B".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("2B");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["2B".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_delimiters_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("some-dashed-phrase");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "some".to_string(),
            "dashed".to_string(),
            "phrase".to_string(),
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("dogs,chase,cats");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["dogs".to_string(), "chase".to_string(), "cats".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("ac/dc");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["ac".to_string(), "dc".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apostrophes_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["O'Reilly".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("you're");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["you're".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("she's");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["she's".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("Jim's");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["Jim's".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("don't");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["don't".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("O'Reilly's");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["O'Reilly's".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_numeric_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("21.35");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["21.35".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("R2D2 C3PO");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["R2D2".to_string(), "C3PO".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("R2D2 C3PO");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["R2D2".to_string(), "C3PO".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("216.239.63.104");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["216.239.63.104".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_text_with_numbers_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("David has 5000 bones");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "David".to_string(),
            "has".to_string(),
            "5000".to_string(),
            "bones".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_various_text_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("C embedded developers wanted");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "C".to_string(),
            "embedded".to_string(),
            "developers".to_string(),
            "wanted".to_string(),
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo bar FOO BAR");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "foo".to_string(),
            "bar".to_string(),
            "FOO".to_string(),
            "BAR".to_string(),
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("foo      bar .  FOO <> BAR");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "foo".to_string(),
            "bar".to_string(),
            "FOO".to_string(),
            "BAR".to_string(),
        ];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("\"QUOTED\" word");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["QUOTED".to_string(), "word".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_korean_sa() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("안녕하세요 한글입니다");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["안녕하세요".to_string(), "한글입니다".to_string()];
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
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["훈민정음".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_japanese() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("仮名遣い カタカナ");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "仮".to_string(),
            "名".to_string(),
            "遣".to_string(),
            "い".to_string(),
            "カタカナ".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("💩 💩💩");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["💩".to_string(), "💩".to_string(), "💩".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("👩‍❤️‍👩");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["👩‍❤️‍👩".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_sequence_with_modifier() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("👨🏼‍⚕️");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["👨🏼‍⚕️".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_regional_indicator() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("🇺🇸🇺🇸");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["🇺🇸".to_string(), "🇺🇸".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_variation_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("#️⃣");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["3️⃣".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tag_sequence() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("🏴󠁧󠁢󠁥󠁮󠁧󠁿");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["🏴󠁧󠁢󠁥󠁮󠁧󠁿".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn test_emoji_tokenization() {
        let tokenizer = &mut ICUTokenizerTokenStream::new("poo💩poo");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec!["poo".to_string(), "💩".to_string(), "poo".to_string()];
        assert_eq!(result, expected);

        let tokenizer = &mut ICUTokenizerTokenStream::new("💩中國💩");
        let result: Vec<String> = tokenizer.map(|v| v.text).collect();
        let expected = vec![
            "💩".to_string(),
            "中".to_string(),
            "國".to_string(),
            "💩".to_string(),
        ];
        assert_eq!(result, expected);
    }
}
