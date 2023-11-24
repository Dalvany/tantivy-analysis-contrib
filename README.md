[![Crate](https://img.shields.io/crates/v/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)
[![Build Status](https://github.com/Dalvany/tantivy-analysis-contrib/actions/workflows/quality.yml/badge.svg)](https://github.com/Dalvany/tantivy-analysis-contrib/actions/workflows/quality.yml)
[![codecov](https://codecov.io/gh/Dalvany/tantivy-analysis-contrib/branch/main/graph/badge.svg)](https://codecov.io/gh/Dalvany/tantivy-analysis-contrib)
[![dependency status](https://deps.rs/repo/github/Dalvany/tantivy-analysis-contrib/status.svg)](https://deps.rs/repo/github/Dalvany/tantivy-analysis-contrib)
[![Documentation](https://docs.rs/tantivy-analysis-contrib/badge.svg)](https://docs.rs/tantivy-analysis-contrib/)
[![Crate](https://img.shields.io/crates/d/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)
[![Crate](https://img.shields.io/crates/l/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)

# Tantivy analysis

This a collection of `Tokenizer` and `TokenFilters` for [Tantivy](https://github.com/quickwit-oss/tantivy) that aims to
replicate features available in [Lucene](https://lucene.apache.org/).

It relies on Google's [Rust ICU](https://crates.io/crates/rust_icu). `libicu-dev` and clang needs to be installed in order to compile.

Breaking word rules are from [Lucene](https://github.com/apache/lucene/tree/main/lucene/analysis/icu/src/data/uax29).

## Features

* `icu` feature includes the following components  (they are also features) :
    * `ICUTokenizer`
    * `ICUNormalizer2TokenFilter`
    * `ICUTransformTokenFilter`
* `commons` features includes the following components
    * `LengthTokenFilter`
    * `LimitTokenCountFilter`
    * `PathTokenizer`
    * `ReverseTokenFilter`
    * `ElisionTokenFilter`
    * `EdgeNgramTokenFilter`
* `phonetic` feature includes some phonetic algorithm (Beider-Morse, Soundex, Metaphone, ... see 
[crate documentation](https://docs.rs/tantivy-analysis-contrib/latest/tantivy_analysis_contrib/))
  * `PhoneticTokenFilter`
* `embedded` which enables embedded rules of rphonetic crate. This feature is not included by default. It has two 
sub-features `embedded-bm` that enables only embedded Beider-Morse rules, and `embedded-dm` which enables only
Daitch-Mokotoff rules.

Note that phonetic support probably needs improvements.

By default, `icu`, `commons` and `phonetic` are included.

## Example

```rust
use tantivy::{doc, Index, ReloadPolicy};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::TextAnalyzer;
use tantivy_analysis_contrib::icu::{Direction, ICUTokenizer, ICUTransformTokenFilter};

const ANALYSIS_NAME: &str = "test";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(ANALYSIS_NAME)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let mut schema = SchemaBuilder::new();
    schema.add_text_field("field", options);
    let schema = schema.build();

    let transform = ICUTransformTokenFilter {
        compound_id: "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
        rules: None,
        direction: Direction::Forward
    };
    let icu_analyzer = TextAnalyzer::from(ICUTokenizer).filter(transform);

    let field = schema.get_field("field").expect("Can't get field.");

    let index = Index::create_in_ram(schema);
    index.tokenizers().register(ANALYSIS_NAME, icu_analyzer);

    let mut index_writer = index.writer(15_000_000)?;

    index_writer.add_document(doc!(
        field => "中国"
    ))?;
    index_writer.add_document(doc!(
        field => "Another Document"
    ))?;

    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![field]);

    let query = query_parser.parse_query("zhong")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);

    let query = query_parser.parse_query("国")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);
    let query = query_parser.parse_query("document")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["Another Document".to_string()];
    assert_eq!(expected, result);
    Ok(())
}
```

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
