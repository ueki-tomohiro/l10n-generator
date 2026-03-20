use std::io::{self, Write};

use anyhow::{Context, Result, anyhow, bail};
use tiny_http::{Response, Server, StatusCode};
use url::Url;

use crate::google::extract_oauth_error_message;
use crate::model::OAuthTokenResponse;

pub(crate) fn run_oauth2_helper() -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_from_callback_url_extracts_code() {
        let code =
            parse_code_from_callback_url("http://localhost/oauth2callback?code=abc123&scope=test");
        assert_eq!(code, Some("abc123".to_string()));
    }
}
