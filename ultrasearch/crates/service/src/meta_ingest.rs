use anyhow::Result;
use core_types::FileMeta;
use core_types::config::PathsSection;
use meta_index::{WriterConfig, add_file_meta_batch, create_writer, open_or_create_index};
use std::path::Path;

/// Ingest a batch of `FileMeta` records into the metadata index and commit.
pub fn ingest_file_meta_batch(
    index_path: &Path,
    metas: impl IntoIterator<Item = FileMeta>,
    writer_cfg: Option<WriterConfig>,
) -> Result<()> {
    let meta = open_or_create_index(index_path)?;
    let mut writer = create_writer(&meta, &writer_cfg.unwrap_or_default())?;
    add_file_meta_batch(&mut writer, &meta.fields, metas)?;
    writer.commit()?;
    Ok(())
}

/// Convenience for ingesting using configured paths.
pub fn ingest_with_paths(
    paths: &PathsSection,
    metas: impl IntoIterator<Item = FileMeta>,
    writer_cfg: Option<WriterConfig>,
) -> Result<()> {
    ingest_file_meta_batch(Path::new(&paths.meta_index), metas, writer_cfg)
}
