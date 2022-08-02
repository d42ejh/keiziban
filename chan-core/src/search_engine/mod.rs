use std::path::Path;
use tantivy::directory::MmapDirectory;
use tantivy::schema::{Schema, SchemaBuilder, STORED, STRING, TEXT};
use tantivy::Index;
use tracing::{event, Level};

/// Open or Create a tantivy index.
pub fn init_tantivy(index_dir: &Path) -> tantivy::Result<Index> {
    if !index_dir.exists() {
        event!(
            Level::INFO,
            "{:?} does not exist.\nCreating a new one.",
            index_dir
        );
        std::fs::create_dir(index_dir)?;
    }

    let mut builder = Schema::builder();

    //board
    //  builder.add_text_field("board_name", TEXT | STORED);
    // builder.add_bytes_field("board_uuid", STORED);
    // builder.add_text_field("board_description", TEXT);

    //thread
    builder.add_text_field("thread_title", TEXT | STORED);
    builder.add_bytes_field("thread_uuid", STORED);

    //thread post
    builder.add_bytes_field("threadpost_uuid", STORED);
    builder.add_text_field("threadpost_body_text", TEXT | STORED);

    let schema = builder.build();
    let mmap_dir = MmapDirectory::open(&index_dir)?;
    Index::open_or_create(mmap_dir, schema)
}
