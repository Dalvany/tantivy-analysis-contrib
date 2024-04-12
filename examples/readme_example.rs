use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions, Value};
use tantivy::tokenizer::TextAnalyzer;
use tantivy::{doc, Index, ReloadPolicy, TantivyDocument};
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

    let transform = ICUTransformTokenFilter::new(
        "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
        None,
        Direction::Forward,
    )?;
    let icu_analyzer = TextAnalyzer::builder(ICUTokenizer)
        .filter(transform)
        .build();

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
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![field]);

    let query = query_parser.parse_query("zhong")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        result = retrieved_doc
            .get_all(field)
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);

    let query = query_parser.parse_query("国")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        result = retrieved_doc
            .get_all(field)
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
    }
    let expected: Vec<String> = vec!["中国".to_string()];
    assert_eq!(expected, result);
    let query = query_parser.parse_query("document")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let mut result: Vec<String> = Vec::new();
    for (_, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        result = retrieved_doc
            .get_all(field)
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
    }
    let expected: Vec<String> = vec!["Another Document".to_string()];
    assert_eq!(expected, result);
    Ok(())
}
