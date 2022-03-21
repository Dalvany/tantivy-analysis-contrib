//! This library tries to bring the same ICU support to Tantivy as [Lucene's ICU](https://lucene.apache.org/core/9_0_0/analysis/icu/index.html).
//!
//! Currently it contains
//! * a [tokenizer](ICUTokenizer) that is an equivalent
//! of [Lucene's ICUTokenizer](https://lucene.apache.org/core/9_0_0/analysis/icu/org/apache/lucene/analysis/icu/segmentation/ICUTokenizer.html)
//! without support of emojis.
//! * a [token filter](ICUNormalizer2TokenFilter) that normalize text. It is an aquivalent of
//! [Lucene's ICUNormalizer2Filter](https://lucene.apache.org/core/9_0_0/analysis/icu/org/apache/lucene/analysis/icu/ICUNormalizer2Filter.html).
//! * another [token filter](ICUTransformTokenFilter) wich is an equivalent of
//! [Lucene's ICUTransformFilter](https://lucene.apache.org/core/9_0_0/analysis/icu/org/apache/lucene/analysis/icu/ICUNormalizer2Filter.html)
//!
//! Here is an example of how tokenize using [ICUTokenizer] and do transliteration and lowercase each tokens using [ICUTransformTokenFilter]:
//! ```rust
//! use tantivy::{doc, Index, ReloadPolicy};
//! use tantivy::collector::TopDocs;
//! use tantivy::query::QueryParser;
//! use tantivy::schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions};
//! use tantivy::tokenizer::TextAnalyzer;
//! use tantivy_analysis_contrib::icu::{Direction, ICUTokenizer, ICUTransformTokenFilter};
//!
//! const ANALYSIS_NAME :&str= "test";
//!
//! let options = TextOptions::default()
//!             .set_indexing_options(
//!                 TextFieldIndexing::default()
//!                     .set_tokenizer(ANALYSIS_NAME)
//!                     .set_index_option(IndexRecordOption::WithFreqsAndPositions),
//!             )
//!            .set_stored();
//! let mut schema = SchemaBuilder::new();
//! schema.add_text_field("field", options);
//! let schema = schema.build();
//!
//! let transform = ICUTransformTokenFilter {
//!     compound_id: "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
//!     rules: None,
//!     direction: Direction::Forward
//! };
//! let icu_analyzer = TextAnalyzer::from(ICUTokenizer).filter(transform);
//!
//! let field = schema.get_field("field").unwrap();
//!
//! let index = Index::create_in_ram(schema);
//! index.tokenizers().register(ANALYSIS_NAME, icu_analyzer);
//!
//! let mut index_writer = index.writer(3_000_000).expect("Error getting index writer");
//!
//! index_writer.add_document(doc!(
//!     field => "中国"
//! ));
//! index_writer.add_document(doc!(
//!     field => "Another Document"
//! ));
//!
//! index_writer.commit();
//!
//! let reader = index
//!         .reader_builder()
//!         .reload_policy(ReloadPolicy::OnCommit)
//!         .try_into().expect("Error getting index reader");
//!
//! let searcher = reader.searcher();
//!
//! let query_parser = QueryParser::for_index(&index, vec![field]);
//!
//! let query = query_parser.parse_query("zhong").expect("Can't create query parser.");
//! let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
//! let mut result:Vec<String> = Vec::new();
//! for (_, doc_address) in top_docs {
//!     let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
//!     let values:Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
//!     for v in values {
//!         result.push(v.to_string());
//!     }
//! }
//! let expected:Vec<String> = vec!["中国".to_string()];
//! assert_eq!(expected,  result);
//!
//! let query = query_parser.parse_query("国").expect("Can't create query parser.");
//! let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
//! let mut result:Vec<String> = Vec::new();
//! for (_, doc_address) in top_docs {
//!     let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
//!     let values:Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
//!     for v in values {
//!         result.push(v.to_string());
//!     }
//! }
//! let expected:Vec<String> = vec!["中国".to_string()];
//! assert_eq!(expected,  result);
//!
//! let query = query_parser.parse_query("document").expect("Can't create query parser.");
//! let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).expect("Error running search");
//! let mut result:Vec<String> = Vec::new();
//! for (_, doc_address) in top_docs {
//!     let retrieved_doc = searcher.doc(doc_address).expect("Can't retrieve document");
//!     let values:Vec<&str> = retrieved_doc.get_all(field).map(|v| v.as_text().unwrap()).collect();
//!     for v in values {
//!         result.push(v.to_string());
//!     }
//! }
//! let expected:Vec<String> = vec!["Another Document".to_string()];
//! assert_eq!(expected,  result);
//! ```
#![cfg_attr(test, deny(warnings))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_numeric_casts,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

#[cfg(any(feature = "normalizer", feature = "tokenizer", feature = "transform"))]
pub mod icu;
