use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};
use tiny_http::{Response, Server, StatusCode};
use url::Url;

#[derive(Debug, Parser)]
#[command(name = "l10n-rust")]
#[command(about = "Rust localization generator")]
struct Cli {
    #[arg(long, default_value = "l10n-generator.config.yaml")]
    config: String,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Diagnose(DiagnoseArgs),
    Oauth2Helper,
}

#[derive(Debug, Parser)]
struct DiagnoseArgs {
    #[arg(long, default_value = "test.config.yaml")]
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
#[serde(rename_all = "camelCase")]
struct DiagnoseConfig {
    file_type: FileType,
    path: String,
    credential_type: CredentialType,
    api_key: Option<String>,
    oauth2: Option<serde_yaml::Value>,
    jwt: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct SpreadsheetMetadata {
    properties: Option<SpreadsheetProperties>,
    sheets: Option<Vec<Sheet>>,
}

#[derive(Debug, Deserialize)]
struct Sheet {
    properties: Option<SheetProperties>,
}

#[derive(Debug, Deserialize)]
struct SheetProperties {
    title: Option<String>,
    grid_properties: Option<GridProperties>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpreadsheetProperties {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GridProperties {
    row_count: Option<u32>,
    column_count: Option<u32>,
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
    refresh_token: Option<String>,
    expires_in: Option<u64>,
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

fn load_diagnose_config(config_path: &str) -> Result<DiagnoseConfig> {
    let absolute_path = resolve_absolute_path(config_path)?;

    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("設定ファイルが見つかりません: {}", absolute_path.display()))?;

    serde_yaml::from_str(&content).with_context(|| "設定ファイルの読み込みに失敗しました")
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

fn run_oauth2_helper() -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔑 OAuth2 トークン取得ヘルパー (Rust)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let client_id = prompt_input("Client ID: ")?;
    let client_secret = prompt_input("Client Secret: ")?;
    if client_id.trim().is_empty() || client_secret.trim().is_empty() {
        bail!("Client IDとClient Secretは必須です");
    }

    let redirect_uri = "http://localhost:3000/oauth2callback";
    let auth_url = build_google_oauth_authorize_url(&client_id, redirect_uri)?;

    println!("以下のURLをブラウザで開いて認証してください:");
    println!("{auth_url}\n");

    println!("ローカルサーバーを起動して認証コードを待機中... (http://localhost:3000)");
    let code = wait_for_oauth_code()?;
    println!("✅ 認証コードを取得しました");

    let token = exchange_oauth_authorization_code(&client_id, &client_secret, redirect_uri, &code)?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ トークンの取得に成功しました!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("oauth2:");
    println!("  clientId: {client_id}");
    println!("  clientSecret: {client_secret}");
    println!("  redirectUri: {redirect_uri}");
    if let Some(refresh_token) = token.refresh_token.as_deref() {
        println!("  refreshToken: {refresh_token}");
    } else {
        println!("  # refreshToken: 未取得（prompt=consent や権限再付与を確認してください）");
    }
    println!("  accessToken: {}", token.access_token);
    if let Some(expires_in) = token.expires_in {
        println!("  # expiresIn: {expires_in} seconds");
    }

    Ok(())
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
    let (spreadsheet_id, metadata) =
        fetch_sheet_metadata(spreadsheet_input, api_key, bearer_token)?;

    let sheet_name = metadata
        .sheets
        .as_ref()
        .and_then(|sheets| sheets.first())
        .and_then(|sheet| sheet.properties.as_ref())
        .and_then(|props| props.title.clone())
        .unwrap_or_else(|| "Sheet1".to_string());

    fetch_sheet_values_by_id(&spreadsheet_id, &sheet_name, api_key, bearer_token)
}

fn build_sheet_api_url(base: &str, api_key: Option<&str>) -> String {
    match api_key {
        Some(key) => format!("{base}?key={key}"),
        None => base.to_string(),
    }
}

fn fetch_sheet_metadata(
    spreadsheet_input: &str,
    api_key: Option<&str>,
    bearer_token: Option<&str>,
) -> Result<(String, SpreadsheetMetadata)> {
    let spreadsheet_id = extract_spreadsheet_id(spreadsheet_input)
        .ok_or_else(|| anyhow!("Invalid Google Sheets URL or ID"))?;
    let metadata_url = build_sheet_api_url(
        &format!("https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}"),
        api_key,
    );
    let metadata: SpreadsheetMetadata = fetch_google_json(&metadata_url, bearer_token)
        .with_context(|| format!("Failed to fetch spreadsheet metadata: {spreadsheet_id}"))?;
    Ok((spreadsheet_id, metadata))
}

fn fetch_sheet_values_by_id(
    spreadsheet_id: &str,
    sheet_name: &str,
    api_key: Option<&str>,
    bearer_token: Option<&str>,
) -> Result<Vec<Vec<String>>> {
    let values_url = build_sheet_api_url(
        &format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}/values/{}",
            urlencoding::encode(sheet_name)
        ),
        api_key,
    );
    let values: SpreadsheetValues = fetch_google_json(&values_url, bearer_token)
        .with_context(|| format!("Failed to fetch spreadsheet values: {spreadsheet_id}"))?;
    Ok(values.values.unwrap_or_default())
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

fn build_google_oauth_authorize_url(client_id: &str, redirect_uri: &str) -> Result<String> {
    let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .with_context(|| "Failed to build OAuth authorize URL")?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair(
            "scope",
            "https://www.googleapis.com/auth/spreadsheets.readonly",
        )
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");
    Ok(url.to_string())
}

fn wait_for_oauth_code() -> Result<String> {
    let server = Server::http("127.0.0.1:3000")
        .map_err(|error| anyhow!("Failed to start local server on :3000: {error}"))?;

    for request in server.incoming_requests() {
        let request_url: &str = request.url();
        let callback_url = format!("http://localhost{request_url}");
        if let Some(code) = parse_code_from_callback_url(&callback_url) {
            let response =
                Response::from_string("認証成功! このタブを閉じて、ターミナルに戻ってください。")
                    .with_status_code(StatusCode(200));
            let _ = request.respond(response);
            return Ok(code);
        }

        let response =
            Response::from_string("Not Found. /oauth2callback?code=... を呼び出してください。")
                .with_status_code(StatusCode(404));
        let _ = request.respond(response);
    }

    bail!("OAuth2認証コードを取得できませんでした")
}

fn parse_code_from_callback_url(callback_url: &str) -> Option<String> {
    let url = Url::parse(callback_url).ok()?;
    if url.path() != "/oauth2callback" {
        return None;
    }
    url.query_pairs()
        .find_map(|(key, value)| (key == "code").then(|| value.to_string()))
}

fn exchange_oauth_authorization_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<OAuthTokenResponse> {
    let token_endpoint = "https://oauth2.googleapis.com/token";
    let response = reqwest::blocking::Client::new()
        .post(token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .with_context(|| "Failed to exchange OAuth2 authorization code")?;

    if !response.status().is_success() {
        let status_text = response.status().to_string();
        let body = response.text().unwrap_or_default();
        let message = extract_oauth_error_message(&body).unwrap_or(status_text);
        bail!("{message}");
    }

    response
        .json::<OAuthTokenResponse>()
        .with_context(|| "Failed to parse OAuth2 code exchange response")
}

fn prompt_input(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout()
        .flush()
        .with_context(|| "Failed to flush stdout")?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .with_context(|| "Failed to read stdin")?;
    Ok(input.trim().to_string())
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

    #[test]
    fn parse_code_from_callback_url_extracts_code() {
        let code =
            parse_code_from_callback_url("http://localhost/oauth2callback?code=abc123&scope=test");
        assert_eq!(code, Some("abc123".to_string()));
    }
}
