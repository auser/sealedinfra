use std::path::Path;

use crate::error::SealedResult;

pub fn make_dirs(path: &Path) -> SealedResult<()> {
    tracing::debug!("Creating directories: {}", path.display());
    std::fs::create_dir_all(path)?;
    tracing::debug!("Created directories: {}", path.display());
    Ok(())
}
