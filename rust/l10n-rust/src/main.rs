use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Parser)]
#[command(name = "l10n-rust")]
#[command(about = "Rust localization generator (CSV path for migration phase 1)")]
struct Cli {
    #[arg(long, default_value = "l10n-generator.config.yaml")]
    config: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum FileType {
    Csv,
    Sheet,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum CredentialType {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "apiKey")]
    ApiKey,
    #[serde(rename = "oauth2")]
    Oauth2,
    #[serde(rename = "jwt")]
    Jwt,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum OutputType {
    #[serde(rename = "dart")]
    Dart,
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "both")]
    Both,
}

fn default_output_type() -> OutputType {
    OutputType::Dart
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    file_type: FileType,
    path: String,
    credential_type: CredentialType,
    localize_path: String,
    #[serde(default = "default_output_type")]
    output_type: OutputType,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("\n✗ エラーが発生しました:");
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    println!("=== l10n-rust ローカライゼーションファイル生成 ===\n");
    println!("設定ファイルを読み込み中: {}", cli.config);
    let config = load_config(&cli.config)?;

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

    if config.file_type != FileType::Csv {
        bail!(
            "Rust移行フェーズ1では fileType: csv のみ対応しています（現在: {}）",
            display_file_type(&config.file_type)
        );
    }

    println!("データをインポート中...");
    let rows = import_csv(&config.path)?;
    println!("  - {}行のデータを読み込みました\n", rows.len());

    if rows.len() < 2 {
        bail!("データが不足しています（ヘッダー行とデータ行が必要です）");
    }

    fs::create_dir_all(&config.localize_path)
        .with_context(|| format!("出力ディレクトリを作成できません: {}", config.localize_path))?;

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

fn load_config(config_path: &str) -> Result<Config> {
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

fn resolve_absolute_path(path: &str) -> Result<PathBuf> {
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        Ok(path_obj.to_path_buf())
    } else {
        let current_dir =
            std::env::current_dir().context("現在ディレクトリの取得に失敗しました")?;
        Ok(current_dir.join(path_obj))
    }
}

fn import_csv(path: &str) -> Result<Vec<Vec<String>>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .with_context(|| format!("CSVファイルを開けません: {path}"))?;

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.with_context(|| "CSVの解析に失敗しました")?;
        rows.push(
            record
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        );
    }

    Ok(rows)
}

fn extract_locales(rows: &[Vec<String>]) -> Result<Vec<String>> {
    let header = rows
        .first()
        .ok_or_else(|| anyhow!("データが空です（ヘッダー行が必要です）"))?;

    if header.len() < 3 {
        bail!("列数が不足しています（最低でも key, description, 言語1 の3列が必要です）");
    }

    Ok(header[2..].to_vec())
}

fn export_dart(output_dir: &str, rows: &[Vec<String>], locales: &[String]) -> Result<()> {
    println!("Dart ARB形式でエクスポート中: {}", locales.join(", "));

    for (locale_index, locale) in locales.iter().enumerate() {
        let arb_json = build_arb_for_locale(rows, locale, locale_index)?;
        let destination = Path::new(output_dir).join(format!("app_{locale}.arb"));
        fs::write(&destination, arb_json)
            .with_context(|| format!("ARBファイルを書き込めません: {}", destination.display()))?;
        println!("  - app_{locale}.arb を生成しました");
    }

    Ok(())
}

fn build_arb_for_locale(rows: &[Vec<String>], locale: &str, locale_index: usize) -> Result<String> {
    let mut root = Map::new();
    root.insert("@@locale".to_string(), Value::String(locale.to_string()));

    for row in rows.iter().skip(1) {
        let key = row.first().map_or("", String::as_str).trim();
        if key.is_empty() {
            continue;
        }

        let description = row.get(1).map_or("", String::as_str);
        let word = row
            .get(locale_index + 2)
            .map_or_else(String::new, std::string::ToString::to_string);

        root.insert(key.to_string(), Value::String(word.clone()));

        let placeholders = parse_input_parameters(&word)?;
        let mut metadata = Map::new();
        metadata.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
        if let Some(placeholders) = placeholders {
            metadata.insert("placeholders".to_string(), placeholders);
        }

        root.insert(format!("@{key}"), Value::Object(metadata));
    }

    serde_json::to_string(&Value::Object(root)).context("ARBのJSON変換に失敗しました")
}

fn export_typescript(output_dir: &str, rows: &[Vec<String>], locales: &[String]) -> Result<()> {
    println!("TypeScript形式でエクスポート中: {}", locales.join(", "));

    let data_rows = &rows[1..];

    let translation_content = build_translation_interface(data_rows);
    let translation_path = Path::new(output_dir).join("translation.ts");
    fs::write(&translation_path, translation_content).with_context(|| {
        format!(
            "translation.tsを書き込めません: {}",
            translation_path.display()
        )
    })?;
    println!("  - translation.ts を生成しました");

    let functions_content = build_translate_functions(data_rows)?;
    let functions_path = Path::new(output_dir).join("translateFunction.ts");
    fs::write(&functions_path, functions_content).with_context(|| {
        format!(
            "translateFunction.tsを書き込めません: {}",
            functions_path.display()
        )
    })?;
    println!("  - translateFunction.ts を生成しました");

    for (locale_index, locale) in locales.iter().enumerate() {
        let locale_content = build_locale_translation_file(data_rows, locale_index)?;
        let locale_path = Path::new(output_dir).join(format!("{locale}.ts"));
        fs::write(&locale_path, locale_content).with_context(|| {
            format!(
                "ロケールファイルを書き込めません: {}",
                locale_path.display()
            )
        })?;
        println!("  - {locale}.ts を生成しました");
    }

    Ok(())
}

fn build_translation_interface(data_rows: &[Vec<String>]) -> String {
    let mut blocks = Vec::new();

    for row in data_rows {
        let key = row.first().map_or("", String::as_str).trim();
        if key.is_empty() {
            continue;
        }
        let description = row.get(1).map_or("", String::as_str);
        let first_translation = row
            .get(2)
            .map_or("", String::as_str)
            .replace(char::is_whitespace, "");

        blocks.push(format!(
            "\n  /**\n   * {}: {}\n   */\n  {}: string;",
            first_translation, description, key
        ));
    }

    format!("export interface Translation {{{}\n}}", blocks.join("\n"))
}

fn build_translate_functions(data_rows: &[Vec<String>]) -> Result<String> {
    let mut output = String::from("import { Translation } from \"./translation\";\n");

    for row in data_rows {
        let key = row.first().map_or("", String::as_str).trim();
        if key.is_empty() {
            continue;
        }

        let description = row.get(1).map_or("", String::as_str);
        let first_translation = row.get(2).map_or("", String::as_str);
        let params = parse_parameter_names(first_translation)?;
        if params.is_empty() {
            continue;
        }

        let typed_params = params
            .iter()
            .map(|param| format!("{param}: string;"))
            .collect::<Vec<_>>()
            .join(" ");

        let replacements = params
            .iter()
            .map(|param| format!(".replaceAll(\"{{{param}}}\", params.{param})"))
            .collect::<String>();

        let function_name = to_camel_case(key);
        let summary_translation = first_translation.replace(char::is_whitespace, "");

        output.push_str(&format!(
            "\n/**\n * {}: {}\n */\nexport const {} = (t: Translation, params: {{ {} }}) => t.{}{};\n",
            summary_translation, description, function_name, typed_params, key, replacements
        ));
    }

    Ok(output)
}

fn build_locale_translation_file(data_rows: &[Vec<String>], locale_index: usize) -> Result<String> {
    let mut translations = Map::new();

    for row in data_rows {
        let key = row.first().map_or("", String::as_str).trim();
        if key.is_empty() {
            continue;
        }
        let value = row
            .get(locale_index + 2)
            .map_or_else(String::new, std::string::ToString::to_string);
        translations.insert(key.to_string(), Value::String(value));
    }

    let json_body = serde_json::to_string_pretty(&Value::Object(translations))
        .context("翻訳JSONの変換に失敗しました")?;

    Ok(format!(
        "import {{ Translation }} from \"./translation\";\n\nexport const translation: Translation = {};",
        json_body
    ))
}

fn parse_input_parameters(text: &str) -> Result<Option<Value>> {
    let names = parse_parameter_names(text)?;
    if names.is_empty() {
        return Ok(None);
    }

    let mut place = Map::new();
    for name in names {
        place.insert(
            name.clone(),
            json!({
                "type": "String",
                "example": name,
            }),
        );
    }

    Ok(Some(Value::Object(place)))
}

fn parse_parameter_names(text: &str) -> Result<Vec<String>> {
    let regex = Regex::new(r"\{(\w+)\}").context("パラメータ検出用の正規表現が不正です")?;
    Ok(regex
        .captures_iter(text)
        .map(|caps| caps[1].to_string())
        .collect())
}

fn to_camel_case(input: &str) -> String {
    let mut output = String::new();
    let mut uppercase_next = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if output.is_empty() {
                output.push(ch.to_ascii_lowercase());
                uppercase_next = false;
            } else if uppercase_next {
                output.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                output.push(ch.to_ascii_lowercase());
            }
        } else if !output.is_empty() {
            uppercase_next = true;
        }
    }

    output
}

fn display_file_type(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Csv => "csv",
        FileType::Sheet => "sheet",
    }
}

fn display_credential_type(credential_type: &CredentialType) -> &'static str {
    match credential_type {
        CredentialType::None => "none",
        CredentialType::ApiKey => "apiKey",
        CredentialType::Oauth2 => "oauth2",
        CredentialType::Jwt => "jwt",
    }
}

fn display_output_type(output_type: &OutputType) -> &'static str {
    match output_type {
        OutputType::Dart => "dart",
        OutputType::TypeScript => "typescript",
        OutputType::Both => "both",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_rows() -> Vec<Vec<String>> {
        vec![
            vec!["key", "description", "ja", "en"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
            vec![
                "welcome",
                "Welcome message",
                "ようこそ{name}さん",
                "Welcome {name}",
            ]
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
            vec![
                "error_count",
                "Error count",
                "{count}件のエラー",
                "{count} errors",
            ]
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        ]
    }

    #[test]
    fn parse_parameter_names_extracts_multiple_placeholders() {
        let names = parse_parameter_names("Welcome {name}, {count}").expect("parse should succeed");
        assert_eq!(names, vec!["name", "count"]);
    }

    #[test]
    fn camel_case_converts_snake_case_key() {
        assert_eq!(to_camel_case("error_count"), "errorCount");
        assert_eq!(to_camel_case("hello"), "hello");
    }

    #[test]
    fn build_arb_for_locale_contains_placeholders() {
        let rows = sample_rows();
        let arb = build_arb_for_locale(&rows, "ja", 0).expect("arb should be generated");
        assert!(arb.contains("\"@@locale\":\"ja\""));
        assert!(arb.contains("\"welcome\":\"ようこそ{name}さん\""));
        assert!(arb.contains("\"placeholders\""));
        assert!(arb.contains("\"name\""));
    }

    #[test]
    fn export_typescript_creates_expected_files() {
        let rows = sample_rows();
        let locales = extract_locales(&rows).expect("locales should be extracted");
        let temp_dir = TempDir::new().expect("tempdir should be created");
        export_typescript(
            temp_dir.path().to_str().expect("path should be utf8"),
            &rows,
            &locales,
        )
        .expect("export should succeed");

        let translation = fs::read_to_string(temp_dir.path().join("translation.ts"))
            .expect("translation.ts should exist");
        assert!(translation.contains("export interface Translation"));
        assert!(translation.contains("welcome: string;"));

        let function_content = fs::read_to_string(temp_dir.path().join("translateFunction.ts"))
            .expect("translateFunction.ts should exist");
        assert!(function_content.contains("export const welcome"));
        assert!(function_content.contains("replaceAll(\"{name}\", params.name)"));

        let ja_content =
            fs::read_to_string(temp_dir.path().join("ja.ts")).expect("ja.ts should be generated");
        assert!(ja_content.contains("\"welcome\": \"ようこそ{name}さん\""));
    }
}
