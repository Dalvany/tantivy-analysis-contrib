[![Crate](https://img.shields.io/crates/v/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)
[![Documentation](https://docs.rs/tantivy-analysis-contrib/badge.svg)](https://docs.rs/tantivy-analysis-contrib/)
[![Build Status](https://github.com/Dalvany/tantivy-analysis-contrib/actions/workflows/rust.yml/badge.svg)](https://github.com/Dalvany/tantivy-analysis-contrib/actions/workflows/rust.yml)
[![Crate](https://img.shields.io/crates/d/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)
[![dependency status](https://deps.rs/repo/github/Dalvany/tantivy-analysis-contrib/status.svg)](https://deps.rs/repo/github/Dalvany/tantivy-analysis-contrib)
[![Crate](https://img.shields.io/crates/l/tantivy-analysis-contrib.svg)](https://crates.io/crates/tantivy-analysis-contrib)

# Tantivy analysis

This a collection of `Tokenizer` and `TokenFilters` that aims to replicate features available
in [Lucene](https://lucene.apache.org/).

It relies on Google's [Rust ICU](https://crates.io/crates/rust_icu).

Breaking word rules are from [Lucene](https://github.com/apache/lucene/tree/main/lucene/analysis/icu/src/data/uax29).

## Features

* `tokenizer` : it enables `ICUTokenizer`.
* `normalizer` : it enables `ICUNormalizer2TokenFilter`.
* `transform` : it enables `ICUTransformTokenFilter` 
* `icu` : all above features
* `commons` : some common token filter 
  * `LengthTokenFilter`
  * `TrimTokenFilter`
  * `LimitTokenCountFilter`
  * `PathTokenizer`
  * `ReverseTokenFilter`
  * `ElisionTokenFilter`

By default, all features are included.

## Example

```rust
use tantivy::{doc, Index, ReloadPolicy};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::TextAnalyzer;
use tantivy_analysis_contrib::{Direction, ICUTokenizer, ICUTransformTokenFilter};

const ANALYSIS_NAME: &str = "test";

fn main() {
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

    let field = schema.get_field("field").unwrap();

    let index = Index::create_in_ram(schema);
    index.tokenizers().register(ANALYSIS_NAME, icu_analyzer);

    let mut index_writer = index.writer(3_000_000).expect("Error getting index writer");

    index_writer.add_document(doc!(
        field => "中国"
    ));
    index_writer.add_document(doc!(
        field => "Another Document"
    ));

    index_writer.commit();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into().expect("Error getting index reader");

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![field]);

    let query = query_parser.parse_query("zhong").expect("Can't create query parser.");
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);

    let query = query_parser.parse_query("国").expect("Can't create query parser.");
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);

    let query = query_parser.parse_query("document").expect("Can't create query parser.");
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
        let values: Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
        for v in values {
            result.push(v.to_string());
        }
    }
    let expected: Vec<String> = vec!["Another Document".to_string()];
    assert_eq!(expected, result);
}
```

## TODO

* Phonetic
* Reverse

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
