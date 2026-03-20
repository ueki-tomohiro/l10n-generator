use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(crate) fn resolve_absolute_path(path: &str) -> Result<PathBuf> {
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        Ok(path_obj.to_path_buf())
    } else {
        let current_dir =
            std::env::current_dir().context("現在ディレクトリの取得に失敗しました")?;
        Ok(current_dir.join(path_obj))
    }
}
