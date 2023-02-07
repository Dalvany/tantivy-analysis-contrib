use std::collections::BTreeSet;
use std::num::NonZeroUsize;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, TokenizerManager, WhitespaceTokenizer};
use tantivy::{doc, DocAddress, Index, ReloadPolicy, Score, Searcher, TantivyError};
use tempdir::TempDir;

use tantivy_analysis_contrib::commons::EdgeNgramTokenFilter;

const ANALYSIS_NAME: &str = "test";

fn get_values(
    searcher: &Searcher,
    field: Field,
    top_docs: Vec<(Score, DocAddress)>,
) -> Result<BTreeSet<String>, TantivyError> {
    let mut result: BTreeSet<String> = BTreeSet::new();

    for (_, doc_address) in &top_docs {
        let doc = searcher.doc(*doc_address)?;
        if let Some(data) = doc.get_first(field) {
            result.insert(data.as_text().unwrap().to_string());
        }
    }

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = TempDir::new("index")?;
    println!("Temp dir : {index_path:?}");

    // Setup everything to index
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

    // Simple analysis pipeline that tokenize on whitespace, lowercase tokens then apply
    // edge ngram filter.
    let edge_ngram = EdgeNgramTokenFilter::new(NonZeroUsize::new(1).unwrap(), None, false)?;
    let analysis = TextAnalyzer::from(WhitespaceTokenizer)
        .filter(LowerCaser)
        .filter(edge_ngram);

    let field = schema.get_field("field").expect("Can't get field.");

    let index = Index::create_in_dir(&index_path, schema.clone())?;
    index.tokenizers().register(ANALYSIS_NAME, analysis);

    // Index few documents.
    let mut index_writer = index.writer(3_000_000)?;
    let data = vec![
        "The quick brown fox jumps over the lazy dog",
        "Another test",
        "Things are done quickly",
    ];
    for d in data.iter() {
        index_writer.add_document(doc!(
            field => *d
        ))?;
    }
    index_writer.commit()?;

    // Setup thing to search
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();

    // We will build a new pipeline and registry it in a separate TokenizerManger we will use for search
    // because we do not want to apply the edge ngram at search time (not efficient and irrelevant results
    // will match).
    let search_analysis = TextAnalyzer::from(WhitespaceTokenizer).filter(LowerCaser);
    // You should also register other analysis pipelines for other fields if any.
    let search_tokenizer_manager = TokenizerManager::new();
    search_tokenizer_manager.register(ANALYSIS_NAME, search_analysis);

    let parser = QueryParser::new(schema, vec![field], search_tokenizer_manager);

    // Try start with "qui"
    let query = parser.parse_query("qui")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let result = get_values(&searcher, field, top_docs)?;

    let mut expected: BTreeSet<String> = BTreeSet::new();
    expected.insert("The quick brown fox jumps over the lazy dog".to_string());
    expected.insert("Things are done quickly".to_string());
    assert_eq!(result, expected);

    // Try start with "quickly"
    let query = parser.parse_query("quickly")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let result = get_values(&searcher, field, top_docs)?;

    let mut expected: BTreeSet<String> = BTreeSet::new();
    expected.insert("Things are done quickly".to_string());
    assert_eq!(result, expected);

    // Try start with "quicker"
    let query = parser.parse_query("quicker")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let result = get_values(&searcher, field, top_docs)?;

    let expected: BTreeSet<String> = BTreeSet::new();
    assert_eq!(result, expected);

    Ok(())
}
