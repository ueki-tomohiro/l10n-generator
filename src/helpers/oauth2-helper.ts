#!/usr/bin/env node

/**
 * OAuth2トークン取得ヘルパースクリプト
 *
 * このスクリプトは、Google Sheets APIのOAuth2認証用のトークンを取得します。
 */

import { OAuth2Client } from "google-auth-library";
import http from "http";
import { URL } from "url";
import open from "open";
import readline from "readline";

const SCOPES = ["https://www.googleapis.com/auth/spreadsheets.readonly"];
const REDIRECT_URI = "http://localhost:3000/oauth2callback";

interface OAuth2Credentials {
  clientId: string;
  clientSecret: string;
}

/**
 * ユーザーにクライアントIDとシークレットを入力させる
 */
async function promptForCredentials(): Promise<OAuth2Credentials> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    console.log("\n📋 Google Cloud ConsoleからOAuth 2.0クライアントの情報を入力してください\n");

    rl.question("Client ID: ", (clientId) => {
      rl.question("Client Secret: ", (clientSecret) => {
        rl.close();
        resolve({ clientId, clientSecret });
      });
    });
  });
}

/**
 * ローカルサーバーを起動して認証コードを受け取る
 */
async function startLocalServer(oauth2Client: OAuth2Client): Promise<string> {
  return new Promise((resolve, reject) => {
    const server = http.createServer(async (req, res) => {
      try {
        if (req.url && req.url.indexOf("/oauth2callback") > -1) {
          const qs = new URL(req.url, REDIRECT_URI).searchParams;
          const code = qs.get("code");

          if (!code) {
            res.end("認証に失敗しました: コードが取得できませんでした");
            reject(new Error("No code found"));
            return;
          }

          res.end("認証成功! このタブを閉じて、ターミナルに戻ってください。");
          server.close();
          resolve(code);
        }
      } catch (e) {
        reject(e);
      }
    });

    server.listen(3000, () => {
      console.log("\n🌐 ローカルサーバーを起動しました (http://localhost:3000)");
    });
  });
}

/**
 * OAuth2認証フローを実行してトークンを取得
 */
async function getTokens(oauth2Client: OAuth2Client): Promise<void> {
  // 認証URLを生成
  const authorizeUrl = oauth2Client.generateAuthUrl({
    access_type: "offline",
    scope: SCOPES,
    prompt: "consent", // 常にリフレッシュトークンを取得
  });

  console.log("\n🔐 認証URLを開いています...");
  console.log("   ブラウザが自動的に開かない場合は、以下のURLを手動で開いてください:");
  console.log(`   ${authorizeUrl}\n`);

  // ブラウザで認証URLを開く
  try {
    await open(authorizeUrl);
  } catch {
    console.warn("⚠️  ブラウザを自動的に開けませんでした");
  }

  // ローカルサーバーで認証コードを受け取る
  const code = await startLocalServer(oauth2Client);

  console.log("\n✅ 認証コードを取得しました");
  console.log("🔄 トークンを取得中...\n");

  // トークンを取得
  const { tokens } = await oauth2Client.getToken(code);

  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("✅ トークンの取得に成功しました!");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  console.log("📝 以下の情報を設定ファイルに追加してください:\n");
  console.log("oauth2:");
  console.log(`  clientId: ${oauth2Client._clientId}`);
  console.log(`  clientSecret: ${oauth2Client._clientSecret}`);
  console.log(`  redirectUri: ${REDIRECT_URI}`);
  console.log(`  refreshToken: ${tokens.refresh_token}`);
  if (tokens.access_token) {
    console.log(
      `  accessToken: ${tokens.access_token}  # オプション（有効期限: ${tokens.expiry_date ? new Date(tokens.expiry_date).toLocaleString() : "不明"}）`
    );
  }

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  if (!tokens.refresh_token) {
    console.warn("⚠️  警告: リフレッシュトークンが取得できませんでした");
    console.warn("   これは、以前に同じクライアントIDで認証した可能性があります。");
    console.warn("   対処方法:");
    console.warn("   1. Google アカウントの「アプリのアクセス権」からアプリを削除");
    console.warn("   2. このスクリプトを再実行");
    console.warn("   または、generateAuthUrl の prompt オプションを確認してください。\n");
  }
}

/**
 * メイン処理
 */
async function main() {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🔑 OAuth2 トークン取得ヘルパー");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    // クライアントIDとシークレットを取得
    const { clientId, clientSecret } = await promptForCredentials();

    if (!clientId || !clientSecret) {
      console.error("\n❌ Client IDとClient Secretは必須です");
      process.exit(1);
    }

    // OAuth2クライアントを初期化
    const oauth2Client = new OAuth2Client({
      clientId,
      clientSecret,
      redirectUri: REDIRECT_URI,
    });

    // トークンを取得
    await getTokens(oauth2Client);

    console.log("💡 次のステップ:");
    console.log("   1. 上記の設定を test-oauth2.config.yaml にコピー");
    console.log("   2. path にスプレッドシートIDを設定");
    console.log("   3. node lib/cli.js --config test-oauth2.config.yaml\n");
  } catch (error) {
    console.error("\n❌ エラーが発生しました:", (error as Error).message);
    process.exit(1);
  }
}

// スクリプトとして実行された場合のみ実行
if (require.main === module) {
  main();
}
