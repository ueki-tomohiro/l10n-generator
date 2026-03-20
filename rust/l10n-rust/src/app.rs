use std::fs;

use anyhow::{Result, anyhow, bail};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::{load_config, load_diagnose_config};
use crate::export::{export_dart, export_typescript, extract_locales};
use crate::google::{
    fetch_access_token_with_oauth2, fetch_access_token_with_service_account, fetch_sheet_metadata,
    fetch_sheet_values_by_id, import_sheet, load_oauth2_config, load_service_account_key,
};
use crate::model::{Config, CredentialType, FileType, OutputType};
use crate::model::{display_credential_type, display_file_type, display_output_type};
use crate::oauth2_helper::run_oauth2_helper;
use crate::path_utils::resolve_absolute_path;

pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Diagnose(args)) => run_diagnose(&args.config),
        Some(Command::Oauth2Helper) => run_oauth2_helper(),
        None => run_generate(&cli.config),
    }
}

fn run_generate(config_path: &str) -> Result<()> {
    println!("=== l10n-rust ローカライゼーションファイル生成 ===\n");
    println!("設定ファイルを読み込み中: {config_path}");
    let config = load_config(config_path)?;

    println!(
        "  - ファイルタイプ: {}",
        display_file_type(&config.file_type)
    );
    println!(
        "  - 認証方式: {}",
        display_credential_type(&config.credential_type)
    );
    println!("  - 出力形式: {}", display_output_type(&config.output_type));
    println!("  - 出力先: {}\n", config.localize_path);

    println!("データをインポート中...");
    let rows = import_values(&config)?;
    println!("  - {}行のデータを読み込みました\n", rows.len());

    if rows.len() < 2 {
        bail!("データが不足しています（ヘッダー行とデータ行が必要です）");
    }

    fs::create_dir_all(&config.localize_path).map_err(|e| {
        anyhow!(
            "出力ディレクトリを作成できません: {} ({e})",
            config.localize_path
        )
    })?;

    let locales = extract_locales(&rows)?;
    match config.output_type {
        OutputType::Dart => {
            export_dart(&config.localize_path, &rows, &locales)?;
        }
        OutputType::TypeScript => {
            export_typescript(&config.localize_path, &rows, &locales)?;
        }
        OutputType::Both => {
            export_dart(&config.localize_path, &rows, &locales)?;
            println!();
            export_typescript(&config.localize_path, &rows, &locales)?;
        }
    }

    println!("\n✓ ローカライゼーションファイルの生成が完了しました");
    Ok(())
}

fn run_diagnose(config_path: &str) -> Result<()> {
    println!("🔍 Google Sheets API 接続診断ツール\n");

    println!("📋 ステップ1: 設定ファイルの確認");
    let absolute_path = resolve_absolute_path(config_path)?;
    if !absolute_path.exists() {
        bail!("{} が見つかりません", absolute_path.display());
    }
    println!("✓ {} を検出しました\n", absolute_path.display());

    println!("📖 ステップ2: 設定の読み込み");
    let config = load_diagnose_config(config_path)?;
    println!("✓ 設定を読み込みました");
    println!(
        "  - ファイルタイプ: {}",
        display_file_type(&config.file_type)
    );
    println!(
        "  - 認証方式: {}",
        display_credential_type(&config.credential_type)
    );

    if config.file_type == FileType::Csv {
        println!("  - CSV Path: {}\n", config.path);
        println!("✓ CSV形式の設定です。診断はGoogle Sheets専用です。");
        println!("\n次のステップ:");
        println!("  cargo run --manifest-path rust/l10n-rust/Cargo.toml -- --config {config_path}");
        return Ok(());
    }

    if config.path.trim().is_empty() {
        bail!("Sheet IDが設定されていません");
    }
    println!("  - Sheet ID: {}", config.path);

    let (api_key, oauth2, jwt) = match config.credential_type {
        CredentialType::None => (None, None, None),
        CredentialType::ApiKey => {
            let key = config
                .api_key
                .as_deref()
                .ok_or_else(|| anyhow!("API key is required"))?;
            println!(
                "  - API Key: {}...\n",
                &key.chars().take(10).collect::<String>()
            );
            (Some(key.to_string()), None, None)
        }
        CredentialType::Oauth2 => {
            let oauth2 = load_oauth2_config(config.oauth2.as_ref())?;
            println!(
                "  - Client ID: {}",
                oauth2.client_id.as_deref().unwrap_or("未設定")
            );
            let has_refresh = oauth2.refresh_token.is_some()
                || oauth2
                    .credentials
                    .as_ref()
                    .and_then(|c| c.refresh_token.as_ref())
                    .is_some();
            println!(
                "  - Refresh Token: {}\n",
                if has_refresh {
                    "設定済み"
                } else {
                    "未設定"
                }
            );
            (None, Some(oauth2), None)
        }
        CredentialType::Jwt => {
            let jwt = load_service_account_key(config.jwt.as_ref())?;
            println!("  - Service Account Email: {}\n", jwt.client_email);
            (None, None, Some(jwt))
        }
    };

    println!("🌐 ステップ3: Google Sheets APIへの接続テスト");
    println!("  接続中...");

    let bearer_token = if let Some(oauth2) = oauth2.as_ref() {
        Some(fetch_access_token_with_oauth2(oauth2)?)
    } else if let Some(jwt) = jwt.as_ref() {
        Some(fetch_access_token_with_service_account(jwt)?)
    } else {
        None
    };

    let (spreadsheet_id, metadata) =
        fetch_sheet_metadata(&config.path, api_key.as_deref(), bearer_token.as_deref())?;

    let rows = if let Some(sheet) = metadata.sheets.as_ref().and_then(|sheets| sheets.first()) {
        let sheet_name = sheet
            .properties
            .as_ref()
            .and_then(|props| props.title.clone())
            .unwrap_or_else(|| "Sheet1".to_string());
        fetch_sheet_values_by_id(
            &spreadsheet_id,
            &sheet_name,
            api_key.as_deref(),
            bearer_token.as_deref(),
        )?
    } else {
        Vec::new()
    };

    println!("\n✅ 接続成功!\n");
    println!("📊 スプレッドシート情報:");
    println!(
        "  - タイトル: {}",
        metadata
            .properties
            .as_ref()
            .and_then(|p| p.title.as_deref())
            .unwrap_or("不明")
    );
    println!(
        "  - シート数: {}",
        metadata.sheets.as_ref().map_or(0, std::vec::Vec::len)
    );
    if let Some(first_sheet) = metadata.sheets.as_ref().and_then(|s| s.first()) {
        println!(
            "  - 最初のシート名: {}",
            first_sheet
                .properties
                .as_ref()
                .and_then(|props| props.title.as_deref())
                .unwrap_or("不明")
        );
        println!(
            "  - 行数: {}",
            first_sheet
                .properties
                .as_ref()
                .and_then(|props| props.grid_properties.as_ref())
                .and_then(|grid| grid.row_count)
                .map_or("不明".to_string(), |v| v.to_string())
        );
        println!(
            "  - 列数: {}",
            first_sheet
                .properties
                .as_ref()
                .and_then(|props| props.grid_properties.as_ref())
                .and_then(|grid| grid.column_count)
                .map_or("不明".to_string(), |v| v.to_string())
        );
    }

    println!("\n📥 ステップ4: データの取得テスト");
    println!("✅ データの取得に成功しました");
    println!("  - 取得した行数: {}", rows.len());
    if let Some(header) = rows.first() {
        println!("  - ヘッダー行: {}", header.join(", "));
        println!("  - データ行数: {}", rows.len().saturating_sub(1));
    }

    println!("\n🔍 ステップ5: データ形式の検証");
    if rows.len() < 2 {
        println!("⚠️  データ行が不足しています");
        println!("   ヘッダー行とデータ行が必要です");
    } else if rows[0].len() < 3 {
        println!("⚠️  列数が不足しています");
        println!("   最低でも key, description, 言語1 の3列が必要です");
        println!("   現在の列数: {}", rows[0].len());
    } else {
        println!("✅ データ形式は正常です");
        println!("   - ロケール数: {}", rows[0].len() - 2);
        println!("   - ロケール: {}", rows[0][2..].join(", "));
    }

    println!("\n🎉 すべての診断テストに合格しました!");
    println!("\n次のステップ:");
    println!("  cargo run --manifest-path rust/l10n-rust/Cargo.toml -- --config {config_path}");
    Ok(())
}

fn import_csv(path: &str) -> Result<Vec<Vec<String>>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|e| anyhow!("CSVファイルを開けません: {path} ({e})"))?;

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| anyhow!("CSVの解析に失敗しました: {e}"))?;
        rows.push(
            record
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        );
    }

    Ok(rows)
}

fn import_values(config: &Config) -> Result<Vec<Vec<String>>> {
    match config.file_type {
        FileType::Csv => import_csv(&config.path),
        FileType::Sheet => import_sheet(
            &config.path,
            &config.credential_type,
            config.api_key.as_deref(),
            config.oauth2.as_ref(),
            config.jwt.as_ref(),
        ),
    }
}
