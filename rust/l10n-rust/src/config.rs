use std::fs;

use anyhow::{Context, Result, bail};

use crate::model::{Config, DiagnoseConfig};
use crate::path_utils::resolve_absolute_path;

pub(crate) fn load_config(config_path: &str) -> Result<Config> {
    let absolute_path = resolve_absolute_path(config_path)?;

    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("設定ファイルが見つかりません: {}", absolute_path.display()))?;

    let config: Config =
        serde_yaml::from_str(&content).with_context(|| "設定ファイルの読み込みに失敗しました")?;

    if config.path.trim().is_empty() || config.localize_path.trim().is_empty() {
        bail!("設定ファイルに必須フィールドが不足しています（path, localizePathが必要です）");
    }

    Ok(config)
}

pub(crate) fn load_diagnose_config(config_path: &str) -> Result<DiagnoseConfig> {
    let absolute_path = resolve_absolute_path(config_path)?;

    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("設定ファイルが見つかりません: {}", absolute_path.display()))?;

    serde_yaml::from_str(&content).with_context(|| "設定ファイルの読み込みに失敗しました")
}
