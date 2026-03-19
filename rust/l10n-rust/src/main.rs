use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

#[derive(Debug, Parser)]
#[command(name = "l10n-rust")]
#[command(about = "Rust localization generator")]
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
    api_key: Option<String>,
    oauth2: Option<serde_yaml::Value>,
    jwt: Option<serde_yaml::Value>,
    localize_path: String,
    #[serde(default = "default_output_type")]
    output_type: OutputType,
}

#[derive(Debug, Deserialize)]
struct SpreadsheetMetadata {
    sheets: Option<Vec<Sheet>>,
}

#[derive(Debug, Deserialize)]
struct Sheet {
    properties: Option<SheetProperties>,
}

#[derive(Debug, Deserialize)]
struct SheetProperties {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpreadsheetValues {
    values: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct GoogleApiErrorWrapper {
    error: Option<GoogleApiError>,
}

#[derive(Debug, Deserialize)]
struct GoogleApiError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    #[serde(alias = "client_email", alias = "email")]
    client_email: String,
    #[serde(alias = "private_key", alias = "key")]
    private_key: String,
    #[serde(alias = "token_uri", alias = "tokenUri", default = "default_token_uri")]
    token_uri: String,
    #[serde(alias = "private_key_id", alias = "keyId", default)]
    private_key_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Oauth2Config {
    client_id: Option<String>,
    client_secret: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    credentials: Option<Oauth2Credentials>,
}

#[derive(Debug, Deserialize)]
struct Oauth2Credentials {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct ServiceAccountClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: usize,
    iat: usize,
}

fn default_token_uri() -> String {
    "https://oauth2.googleapis.com/token".to_string()
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

    println!("データをインポート中...");
    let rows = import_values(&config)?;
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

fn import_sheet(
    path_or_id: &str,
    credential_type: &CredentialType,
    api_key: Option<&str>,
    oauth2: Option<&serde_yaml::Value>,
    jwt: Option<&serde_yaml::Value>,
) -> Result<Vec<Vec<String>>> {
    match credential_type {
        CredentialType::None => fetch_sheet_values(path_or_id, None),
        CredentialType::ApiKey => {
            let key = api_key.ok_or_else(|| anyhow!("API key is required"))?;
            fetch_sheet_values(path_or_id, Some(key))
        }
        CredentialType::Oauth2 => {
            let oauth2_config = load_oauth2_config(oauth2)?;
            let access_token = fetch_access_token_with_oauth2(&oauth2_config)?;
            fetch_sheet_values_with_bearer(path_or_id, &access_token)
        }
        CredentialType::Jwt => {
            let service_account_key = load_service_account_key(jwt)?;
            let access_token = fetch_access_token_with_service_account(&service_account_key)?;
            fetch_sheet_values_with_bearer(path_or_id, &access_token)
        }
    }
}

fn fetch_sheet_values(spreadsheet_input: &str, api_key: Option<&str>) -> Result<Vec<Vec<String>>> {
    fetch_sheet_values_inner(spreadsheet_input, api_key, None)
}

fn fetch_sheet_values_with_bearer(
    spreadsheet_input: &str,
    bearer_token: &str,
) -> Result<Vec<Vec<String>>> {
    fetch_sheet_values_inner(spreadsheet_input, None, Some(bearer_token))
}

fn fetch_sheet_values_inner(
    spreadsheet_input: &str,
    api_key: Option<&str>,
    bearer_token: Option<&str>,
) -> Result<Vec<Vec<String>>> {
    let spreadsheet_id = extract_spreadsheet_id(spreadsheet_input)
        .ok_or_else(|| anyhow!("Invalid Google Sheets URL or ID"))?;

    let metadata_url = build_sheet_api_url(
        &format!("https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}"),
        api_key,
    );
    let metadata: SpreadsheetMetadata = fetch_google_json(&metadata_url, bearer_token)
        .with_context(|| format!("Failed to fetch spreadsheet metadata: {spreadsheet_id}"))?;

    let sheet_name = metadata
        .sheets
        .as_ref()
        .and_then(|sheets| sheets.first())
        .and_then(|sheet| sheet.properties.as_ref())
        .and_then(|props| props.title.clone())
        .unwrap_or_else(|| "Sheet1".to_string());

    let values_url = build_sheet_api_url(
        &format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}/values/{}",
            urlencoding::encode(&sheet_name)
        ),
        api_key,
    );

    let values: SpreadsheetValues = fetch_google_json(&values_url, bearer_token)
        .with_context(|| format!("Failed to fetch spreadsheet values: {spreadsheet_id}"))?;

    Ok(values.values.unwrap_or_default())
}

fn build_sheet_api_url(base: &str, api_key: Option<&str>) -> String {
    match api_key {
        Some(key) => format!("{base}?key={key}"),
        None => base.to_string(),
    }
}

fn fetch_google_json<T: DeserializeOwned>(url: &str, bearer_token: Option<&str>) -> Result<T> {
    let client = reqwest::blocking::Client::new();
    let mut request = client.get(url);
    if let Some(token) = bearer_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .with_context(|| format!("HTTP request failed: {url}"))?;

    if !response.status().is_success() {
        let status_text = response.status().to_string();
        let body = response.text().unwrap_or_default();
        let message = extract_google_error_message(&body).unwrap_or(status_text);
        bail!("{message}");
    }

    response
        .json::<T>()
        .with_context(|| format!("Failed to parse response JSON: {url}"))
}

fn load_service_account_key(jwt_field: Option<&serde_yaml::Value>) -> Result<ServiceAccountKey> {
    let raw_value = jwt_field.ok_or_else(|| anyhow!("JWT credentials are required"))?;

    match raw_value {
        serde_yaml::Value::String(path) => {
            let file_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read JWT credentials from file: {path}"))?;
            serde_json::from_str::<ServiceAccountKey>(&file_content)
                .with_context(|| format!("Failed to parse JWT credentials file as JSON: {path}"))
        }
        serde_yaml::Value::Mapping(_) => {
            serde_yaml::from_value::<ServiceAccountKey>(raw_value.clone())
                .with_context(|| "Failed to parse inline JWT credentials")
        }
        _ => bail!("JWT credentials format is invalid. Use file path or mapping object"),
    }
}

fn load_oauth2_config(oauth2_field: Option<&serde_yaml::Value>) -> Result<Oauth2Config> {
    let raw_value = oauth2_field.ok_or_else(|| anyhow!("OAuth2 credentials are required"))?;
    serde_yaml::from_value::<Oauth2Config>(raw_value.clone())
        .with_context(|| "Failed to parse OAuth2 credentials")
}

fn fetch_access_token_with_service_account(service_account: &ServiceAccountKey) -> Result<String> {
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock appears to be before UNIX_EPOCH")?
        .as_secs() as usize;

    let claims = ServiceAccountClaims {
        iss: service_account.client_email.clone(),
        scope: "https://www.googleapis.com/auth/spreadsheets.readonly".to_string(),
        aud: service_account.token_uri.clone(),
        iat: issued_at,
        exp: issued_at + 3600,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = service_account.private_key_id.clone();

    let encoding_key = EncodingKey::from_rsa_pem(service_account.private_key.as_bytes())
        .with_context(|| "Failed to parse service account private key")?;
    let assertion = encode(&header, &claims, &encoding_key)
        .with_context(|| "Failed to generate service account JWT assertion")?;

    let token_endpoint = &service_account.token_uri;
    let response = reqwest::blocking::Client::new()
        .post(token_endpoint)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", assertion.as_str()),
        ])
        .send()
        .with_context(|| format!("Failed to request OAuth token: {token_endpoint}"))?;

    if !response.status().is_success() {
        let status_text = response.status().to_string();
        let body = response.text().unwrap_or_default();
        let message = extract_oauth_error_message(&body).unwrap_or(status_text);
        bail!("{message}");
    }

    let token = response
        .json::<OAuthTokenResponse>()
        .with_context(|| "Failed to parse OAuth token response")?;

    Ok(token.access_token)
}

fn fetch_access_token_with_oauth2(oauth2: &Oauth2Config) -> Result<String> {
    let access_token = oauth2.access_token.clone().or_else(|| {
        oauth2
            .credentials
            .as_ref()
            .and_then(|credentials| credentials.access_token.clone())
    });
    if let Some(access_token) = access_token {
        return Ok(access_token);
    }

    let refresh_token = oauth2.refresh_token.clone().or_else(|| {
        oauth2
            .credentials
            .as_ref()
            .and_then(|credentials| credentials.refresh_token.clone())
    });
    let refresh_token =
        refresh_token.ok_or_else(|| anyhow!("OAuth2 refresh token or access token is required"))?;

    let client_id = oauth2
        .client_id
        .as_ref()
        .ok_or_else(|| anyhow!("OAuth2 clientId is required for refresh token flow"))?;
    let client_secret = oauth2
        .client_secret
        .as_ref()
        .ok_or_else(|| anyhow!("OAuth2 clientSecret is required for refresh token flow"))?;

    let token_endpoint = "https://oauth2.googleapis.com/token";
    let response = reqwest::blocking::Client::new()
        .post(token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
        ])
        .send()
        .with_context(|| "Failed to request OAuth2 access token")?;

    if !response.status().is_success() {
        let status_text = response.status().to_string();
        let body = response.text().unwrap_or_default();
        let message = extract_oauth_error_message(&body).unwrap_or(status_text);
        bail!("{message}");
    }

    let token = response
        .json::<OAuthTokenResponse>()
        .with_context(|| "Failed to parse OAuth2 token response")?;
    Ok(token.access_token)
}

fn extract_oauth_error_message(body: &str) -> Option<String> {
    let parsed = serde_json::from_str::<OAuthErrorResponse>(body).ok()?;
    match (parsed.error, parsed.error_description) {
        (Some(error), Some(description)) => Some(format!("{error}: {description}")),
        (Some(error), None) => Some(error),
        (None, Some(description)) => Some(description),
        (None, None) => None,
    }
}

fn extract_google_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<GoogleApiErrorWrapper>(body)
        .ok()
        .and_then(|wrapper| wrapper.error)
        .and_then(|error| error.message)
}

fn extract_spreadsheet_id(input: &str) -> Option<String> {
    if let Some(prefix_idx) = input.find("/spreadsheets/d/") {
        let suffix = &input[prefix_idx + "/spreadsheets/d/".len()..];
        let id: String = suffix
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !id.is_empty() {
            return Some(id);
        }
    }

    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Some(input.to_string());
    }

    None
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

    #[test]
    fn extract_spreadsheet_id_from_full_url() {
        let id = extract_spreadsheet_id(
            "https://docs.google.com/spreadsheets/d/1A2B3C4D5E6F7G8H9I0J/edit#gid=12345",
        );
        assert_eq!(id, Some("1A2B3C4D5E6F7G8H9I0J".to_string()));
    }

    #[test]
    fn extract_spreadsheet_id_from_plain_id() {
        let id = extract_spreadsheet_id("1A2B3C4D5E6F7G8H9I0J");
        assert_eq!(id, Some("1A2B3C4D5E6F7G8H9I0J".to_string()));
    }

    #[test]
    fn extract_spreadsheet_id_rejects_invalid_input() {
        let id = extract_spreadsheet_id("https://example.com/not-a-sheet");
        assert_eq!(id, None);
    }

    #[test]
    fn oauth2_direct_access_token_is_used_without_refresh() {
        let config = Oauth2Config {
            client_id: None,
            client_secret: None,
            access_token: Some("test-access-token".to_string()),
            refresh_token: None,
            credentials: None,
        };
        let token = fetch_access_token_with_oauth2(&config).expect("token should be resolved");
        assert_eq!(token, "test-access-token");
    }
}
