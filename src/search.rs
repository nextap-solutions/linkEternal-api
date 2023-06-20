use std::path::Path;

use tantivy::query::QueryParserError;
use thiserror::Error;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Schema;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::ReloadPolicy;
use tantivy::TantivyError;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to create temp dir")]
    TempDirError(#[from] std::io::Error),
    #[error("Failed to create index")]
    IndexError(TantivyError),
    #[error("Failed to create index writer")]
    IndexWriterError(TantivyError),
    #[error("Failed to create index reader")]
    IndexReaderError(TantivyError),
    #[error("Failed to search")]
    SearchError(TantivyError),
    #[error("Failed parse query")]
    ParserError(QueryParserError),
}

pub struct Search {
    index: Index,
    index_writer: IndexWriter,
    schema: Schema,
    reader: IndexReader,
}

impl Search {
    pub fn new() -> Result<Self, SearchError> {
        let index_path = Path::new("./data");
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("url", TEXT | STORED);
        schema_builder.add_text_field("description", TEXT | STORED);
        schema_builder.add_text_field("tags", TEXT | STORED);

        let schema = schema_builder.build();
        let index = match Index::create_in_dir(index_path, schema.clone()) {
            Ok(v) => v,
            Err(err) => return Err(SearchError::IndexError(err)),
        };
        let index_writer = match index.writer(50_000_000) {
            Ok(v) => v,
            Err(err) => return Err(SearchError::IndexWriterError(err)),
        };
        let reader = match index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
        {
            Ok(v) => v,
            Err(err) => return Err(SearchError::IndexReaderError(err)),
        };

        Ok(Self {
            index,
            schema,
            index_writer,
            reader,
        })
    }

    pub fn add(
        &mut self,
        url: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<(), SearchError> {
        let url_field = self.schema.get_field("url").unwrap();
        let description_field = self.schema.get_field("description").unwrap();
        let tags_field = self.schema.get_field("tags").unwrap();

        let mut new_doc = Document::default();
        new_doc.add_text(url_field, url);
        new_doc.add_text(description_field, description);
        for tag in tags {
            new_doc.add_text(tags_field, tag);
        }
        match self.index_writer.add_document(new_doc) {
            Ok(v) => v,
            Err(err) => return Err(SearchError::IndexError(err)),
        };

        match self.index_writer.commit() {
            Ok(v) => v,
            Err(err) => return Err(SearchError::IndexError(err)),
        };

        Ok(())
    }

    pub fn search(&self, keyword: String) -> Result<(), SearchError> {
        let url_field = self.schema.get_field("url").unwrap();
        let description_field = self.schema.get_field("description").unwrap();
        let tags_field = self.schema.get_field("tags").unwrap();

        let searcher = self.reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![url_field, description_field, tags_field]);

        let query = match query_parser.parse_query(keyword.as_str()) {
            Ok(v) => v,
            Err(err) => return Err(SearchError::ParserError(err)),
        };
        let top_docs = match searcher.search(&query, &TopDocs::with_limit(10)) {
            Ok(v) => v,
            Err(err) => return Err(SearchError::SearchError(err)),
        };
        println!("{}", top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc = match searcher.doc(doc_address) {
                Ok(v) => v,
                Err(err) => return Err(SearchError::SearchError(err)),
            };
            println!("{}", self.schema.to_json(&retrieved_doc));
        }

        Ok(())
    }
}
