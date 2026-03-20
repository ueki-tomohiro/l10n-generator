use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FileType {
    Csv,
    Sheet,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) enum CredentialType {
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
pub(crate) enum OutputType {
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
pub(crate) struct Config {
    pub(crate) file_type: FileType,
    pub(crate) path: String,
    pub(crate) credential_type: CredentialType,
    pub(crate) api_key: Option<String>,
    pub(crate) oauth2: Option<serde_yaml::Value>,
    pub(crate) jwt: Option<serde_yaml::Value>,
    pub(crate) localize_path: String,
    #[serde(default = "default_output_type")]
    pub(crate) output_type: OutputType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnoseConfig {
    pub(crate) file_type: FileType,
    pub(crate) path: String,
    pub(crate) credential_type: CredentialType,
    pub(crate) api_key: Option<String>,
    pub(crate) oauth2: Option<serde_yaml::Value>,
    pub(crate) jwt: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpreadsheetMetadata {
    pub(crate) properties: Option<SpreadsheetProperties>,
    pub(crate) sheets: Option<Vec<Sheet>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Sheet {
    pub(crate) properties: Option<SheetProperties>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SheetProperties {
    pub(crate) title: Option<String>,
    pub(crate) grid_properties: Option<GridProperties>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpreadsheetProperties {
    pub(crate) title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GridProperties {
    pub(crate) row_count: Option<u32>,
    pub(crate) column_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpreadsheetValues {
    pub(crate) values: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleApiErrorWrapper {
    pub(crate) error: Option<GoogleApiError>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleApiError {
    pub(crate) message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServiceAccountKey {
    #[serde(alias = "client_email", alias = "email")]
    pub(crate) client_email: String,
    #[serde(alias = "private_key", alias = "key")]
    pub(crate) private_key: String,
    #[serde(alias = "token_uri", alias = "tokenUri", default = "default_token_uri")]
    pub(crate) token_uri: String,
    #[serde(alias = "private_key_id", alias = "keyId", default)]
    pub(crate) private_key_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OAuthTokenResponse {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OAuthErrorResponse {
    pub(crate) error: Option<String>,
    pub(crate) error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Oauth2Config {
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) credentials: Option<Oauth2Credentials>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Oauth2Credentials {
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ServiceAccountClaims {
    pub(crate) iss: String,
    pub(crate) scope: String,
    pub(crate) aud: String,
    pub(crate) exp: usize,
    pub(crate) iat: usize,
}

fn default_token_uri() -> String {
    "https://oauth2.googleapis.com/token".to_string()
}

pub(crate) fn display_file_type(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Csv => "csv",
        FileType::Sheet => "sheet",
    }
}

pub(crate) fn display_credential_type(credential_type: &CredentialType) -> &'static str {
    match credential_type {
        CredentialType::None => "none",
        CredentialType::ApiKey => "apiKey",
        CredentialType::Oauth2 => "oauth2",
        CredentialType::Jwt => "jwt",
    }
}

pub(crate) fn display_output_type(output_type: &OutputType) -> &'static str {
    match output_type {
        OutputType::Dart => "dart",
        OutputType::TypeScript => "typescript",
        OutputType::Both => "both",
    }
}
