use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::de::DeserializeOwned;

use crate::model::{
    CredentialType, GoogleApiErrorWrapper, OAuthErrorResponse, OAuthTokenResponse, Oauth2Config,
    ServiceAccountClaims, ServiceAccountKey, SpreadsheetMetadata, SpreadsheetValues,
};

pub(crate) fn import_sheet(
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

pub(crate) fn fetch_sheet_metadata(
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

pub(crate) fn fetch_sheet_values_by_id(
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

pub(crate) fn load_service_account_key(
    jwt_field: Option<&serde_yaml::Value>,
) -> Result<ServiceAccountKey> {
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

pub(crate) fn load_oauth2_config(oauth2_field: Option<&serde_yaml::Value>) -> Result<Oauth2Config> {
    let raw_value = oauth2_field.ok_or_else(|| anyhow!("OAuth2 credentials are required"))?;
    serde_yaml::from_value::<Oauth2Config>(raw_value.clone())
        .with_context(|| "Failed to parse OAuth2 credentials")
}

pub(crate) fn fetch_access_token_with_service_account(
    service_account: &ServiceAccountKey,
) -> Result<String> {
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

pub(crate) fn fetch_access_token_with_oauth2(oauth2: &Oauth2Config) -> Result<String> {
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

pub(crate) fn extract_oauth_error_message(body: &str) -> Option<String> {
    let parsed = serde_json::from_str::<OAuthErrorResponse>(body).ok()?;
    match (parsed.error, parsed.error_description) {
        (Some(error), Some(description)) => Some(format!("{error}: {description}")),
        (Some(error), None) => Some(error),
        (None, Some(description)) => Some(description),
        (None, None) => None,
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

fn extract_google_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<GoogleApiErrorWrapper>(body)
        .ok()
        .and_then(|wrapper| wrapper.error)
        .and_then(|error| error.message)
}

pub(crate) fn extract_spreadsheet_id(input: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
