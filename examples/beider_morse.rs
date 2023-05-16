use std::path::PathBuf;

use lazy_static::lazy_static;
use rphonetic::ConfigFiles;
use tantivy::schema::{IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index};
use tempdir::TempDir;

use tantivy_analysis_contrib::phonetic::{
    Concat, MaxPhonemeNumber, PhoneticAlgorithm, PhoneticTokenFilter,
};

const ANALYSIS_NAME: &str = "test";
lazy_static! {
    static ref CONFIG_FILES: ConfigFiles =
        ConfigFiles::new(&PathBuf::from("./test_assets/bm-cc-rules/")).unwrap();
}

#[allow(clippy::disallowed_macros)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = TempDir::new("index")?;
    println!("Temp dir : {index_path:?}");

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
    let algorithm = PhoneticAlgorithm::BeiderMorse(
        &CONFIG_FILES,
        None,
        None,
        Concat(None),
        MaxPhonemeNumber(None),
        vec![],
    );

    let phonetic_filter: PhoneticTokenFilter = algorithm.try_into()?;
    let beider_morse = TextAnalyzer::from(WhitespaceTokenizer).filter(phonetic_filter);

    let field = schema.get_field("field").expect("Can't get field.");

    let index = Index::create_in_dir(&index_path, schema)?;
    index.tokenizers().register(ANALYSIS_NAME, beider_morse);

    let mut index_writer = index.writer(3_000_000)?;
    let data = vec![
        "Angelo",
        "GOLDEN",
        "Alpert",
        "Breuer",
        "Haber",
        "Mannheim",
        "Mintz",
        "Topf",
        "Kleinmann",
        "Ben Aron",
        "AUERBACH",
        "OHRBACH",
        "Українська",
        "азбука",
        "",
    ];
    for (i, d) in data.iter().enumerate() {
        index_writer.add_document(doc!(
            field => *d
        ))?;

        if i % 10 == 0 {
            index_writer.commit()?;
        }
    }

    index_writer.commit()?;

    Ok(())
}
