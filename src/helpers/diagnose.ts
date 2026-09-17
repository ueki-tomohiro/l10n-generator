import fs from "fs";
import yaml from "js-yaml";
import fetch from "node-fetch";
import { google } from "googleapis";
import { Config } from "./type.js";
import { toSheetRange } from "./sheetRange.js";

interface DiagnoseOptions {
  configFile: string;
}

/**
 * Google Sheets API接続の診断を実行
 */
export async function diagnose(options: DiagnoseOptions): Promise<void> {
  const { configFile } = options;

  console.log("🔍 Google Sheets API 接続診断ツール\n");

  // 1. 設定ファイルの確認
  console.log("📋 ステップ1: 設定ファイルの確認");
  if (!fs.existsSync(configFile)) {
    console.error(`❌ ${configFile} が見つかりません`);
    process.exit(1);
  }
  console.log(`✓ ${configFile} を検出しました\n`);

  // 2. 設定の読み込み
  console.log("📖 ステップ2: 設定の読み込み");
  let config: Config;
  try {
    const fileContents = fs.readFileSync(configFile, "utf8");
    config = yaml.load(fileContents) as Config;
    console.log(`✓ 設定を読み込みました`);
    console.log(`  - ファイルタイプ: ${config.fileType}`);
    console.log(`  - 認証方式: ${config.credentialType}`);

    if (config.fileType === "csv" || config.fileType === "xlsx") {
      console.log(`  - ${config.fileType === "csv" ? "CSV" : "xlsx"} Path: ${config.path}\n`);
      console.log("✓ ローカルファイルの設定です。診断はGoogle Sheets専用です。");
      console.log("\n次のステップ:");
      console.log(`  node lib/cli.js --config ${configFile}`);
      process.exit(0);
    }

    if (config.fileType === "sheet") {
      console.log(`  - Sheet ID: ${config.path}`);

      if (config.credentialType === "apiKey") {
        console.log(`  - API Key: ${config.apiKey ? config.apiKey.substring(0, 10) + "..." : "未設定"}\n`);
      } else if (config.credentialType === "oauth2") {
        console.log(
          `  - Client ID: ${config.oauth2?.clientId ? config.oauth2.clientId.substring(0, 20) + "..." : "未設定"}`
        );
        const hasRefreshToken = config.oauth2?.credentials?.refresh_token || (config.oauth2 as any)?.refreshToken;
        console.log(`  - Refresh Token: ${hasRefreshToken ? "設定済み" : "未設定"}\n`);
      } else if (config.credentialType === "jwt") {
        if (typeof config.jwt === "string") {
          console.log(`  - JWT File: ${config.jwt}\n`);
        } else {
          console.log(`  - Service Account Email: ${config.jwt?.email || "未設定"}\n`);
        }
      } else {
        console.log("\n⚠️  認証方式が指定されていません");
        process.exit(0);
      }
    }
  } catch (error) {
    console.error(`❌ 設定ファイルの読み込みに失敗: ${(error as Error).message}`);
    process.exit(1);
  }

  // 認証情報の検証
  if (config.credentialType === "apiKey") {
    if (!config.apiKey || config.apiKey.includes("YOUR_API_KEY")) {
      console.error("❌ APIキーが設定されていません");
      console.log("\n📝 Google Cloud ConsoleでAPIキーを取得してください:");
      console.log("   https://console.cloud.google.com/apis/credentials");
      process.exit(1);
    }
  } else if (config.credentialType === "oauth2") {
    if (!config.oauth2?.clientId || !config.oauth2?.clientSecret) {
      console.error("❌ OAuth2のクライアントIDまたはクライアントシークレットが設定されていません");
      console.log("\n📝 Google Cloud ConsoleでOAuth2クライアントを作成してください:");
      console.log("   https://console.cloud.google.com/apis/credentials");
      process.exit(1);
    }
    // refreshTokenとaccessTokenは、credentials以下にもトップレベルにも配置される可能性がある
    const hasRefreshToken = config.oauth2?.credentials?.refresh_token || (config.oauth2 as any)?.refreshToken;
    const hasAccessToken = config.oauth2?.credentials?.access_token || (config.oauth2 as any)?.accessToken;
    if (!hasRefreshToken && !hasAccessToken) {
      console.error("❌ OAuth2のトークンが設定されていません");
      console.log("\n📝 トークン取得ヘルパーを実行してください:");
      console.log("   node lib/helpers/oauth2-helper.js");
      process.exit(1);
    }
  } else if (config.credentialType === "jwt") {
    if (typeof config.jwt === "string") {
      // ファイルパスの場合、ファイルの存在を確認
      if (!fs.existsSync(config.jwt)) {
        console.error(`❌ JWTファイルが見つかりません: ${config.jwt}`);
        console.log("\n📝 正しいJSONファイルのパスを指定してください");
        process.exit(1);
      }
    } else if (!config.jwt?.email || !config.jwt?.key) {
      console.error("❌ Service Accountの情報が設定されていません");
      console.log("\n📝 Google Cloud ConsoleでService Accountを作成し、JSONキーをダウンロードしてください:");
      console.log("   https://console.cloud.google.com/iam-admin/serviceaccounts");
      process.exit(1);
    }
  }

  if (!config.path || config.path.includes("YOUR_SHEET_ID")) {
    console.error("❌ Sheet IDが設定されていません");
    console.log("\n📝 Google SheetsのURLからSheet IDをコピーしてください");
    process.exit(1);
  }

  // 3. Google Sheets APIへの接続テスト
  console.log("🌐 ステップ3: Google Sheets APIへの接続テスト");

  try {
    console.log("  接続中...");

    let metadata: {
      properties?: { title?: string };
      sheets?: Array<{
        properties?: {
          title?: string;
          gridProperties?: {
            rowCount?: number;
            columnCount?: number;
          };
        };
      }>;
    } = {};
    let rows: string[][] = [];

    if (config.credentialType === "apiKey") {
      // API Key認証
      const metadataUrl = `https://sheets.googleapis.com/v4/spreadsheets/${config.path}?key=${config.apiKey}`;
      const response = await fetch(metadataUrl);

      if (!response.ok) {
        const error = (await response.json()) as { error?: { message?: string } };
        console.error(`\n❌ 接続エラー: ${error.error?.message || response.statusText}\n`);

        if (error.error?.message?.includes("API key not valid")) {
          console.log("💡 解決方法:");
          console.log("   1. Google Cloud Consoleで正しいAPIキーを確認");
          console.log("   2. APIキーが正しくコピーされているか確認");
          console.log("   3. Google Sheets APIが有効化されているか確認");
          console.log("      https://console.cloud.google.com/apis/library/sheets.googleapis.com");
        } else if (error.error?.message?.includes("permission") || error.error?.message?.includes("Permission")) {
          console.log("💡 解決方法:");
          console.log("   1. スプレッドシートを開く:");
          console.log(`      https://docs.google.com/spreadsheets/d/${config.path}/edit`);
          console.log("   2. 右上の「共有」ボタンをクリック");
          console.log("   3. 「リンクを知っている全員」に変更");
          console.log("   4. 権限を「閲覧者」に設定");
        } else if (error.error?.message?.includes("not found") || error.error?.message?.includes("Not found")) {
          console.log("💡 解決方法:");
          console.log("   1. Sheet IDが正しいか確認");
          console.log("   2. URLの /d/ と /edit の間の文字列をコピー");
        } else {
          console.log("💡 その他のエラー:");
          console.log("   詳しくは TESTING.md を参照してください");
        }

        process.exit(1);
      }

      metadata = (await response.json()) as typeof metadata;

      // データの取得
      const sheetName = config.sheetName ?? (metadata.sheets?.[0]?.properties?.title || "Sheet1");
      const valuesUrl = `https://sheets.googleapis.com/v4/spreadsheets/${config.path}/values/${encodeURIComponent(toSheetRange(sheetName))}?key=${config.apiKey}`;

      const valuesResponse = await fetch(valuesUrl);
      if (!valuesResponse.ok) {
        const error = (await valuesResponse.json()) as { error?: { message?: string } };
        console.error(`❌ データ取得エラー: ${error.error?.message || valuesResponse.statusText}`);
        process.exit(1);
      }

      const data = (await valuesResponse.json()) as { values?: string[][] };
      rows = data.values || [];
    } else if (config.credentialType === "oauth2") {
      // OAuth2認証
      const auth = new google.auth.OAuth2(config.oauth2);

      // refreshToken/accessTokenがトップレベルにある場合、credentialsに変換
      const oauth2Any = config.oauth2 as any;
      if (oauth2Any?.refreshToken || oauth2Any?.accessToken) {
        auth.setCredentials({
          refresh_token: oauth2Any.refreshToken,
          access_token: oauth2Any.accessToken,
        });
      }

      const sheets = google.sheets({ version: "v4", auth });

      const spreadsheet = await sheets.spreadsheets.get({
        spreadsheetId: config.path,
      });

      metadata = {
        properties: { title: spreadsheet.data.properties?.title || undefined },
        sheets: spreadsheet.data.sheets?.map((sheet) => ({
          properties: {
            title: sheet.properties?.title || undefined,
            gridProperties: {
              rowCount: sheet.properties?.gridProperties?.rowCount || undefined,
              columnCount: sheet.properties?.gridProperties?.columnCount || undefined,
            },
          },
        })),
      };

      const sheetName = config.sheetName ?? (spreadsheet.data.sheets?.[0]?.properties?.title || "Sheet1");
      const valuesResponse = await sheets.spreadsheets.values.get({
        spreadsheetId: config.path,
        range: toSheetRange(sheetName),
      });

      rows = (valuesResponse.data.values as string[][]) || [];
    } else if (config.credentialType === "jwt") {
      // JWT認証
      // config.jwtが文字列の場合、JSONファイルから読み込む
      let jwtOptions: any;
      if (typeof config.jwt === "string") {
        const fileContent = fs.readFileSync(config.jwt, "utf8");
        const jsonData = JSON.parse(fileContent);

        // Service Account JSONファイルのフィールド名をJWTOptionsに変換
        jwtOptions = {
          email: jsonData.client_email,
          key: jsonData.private_key,
          keyId: jsonData.private_key_id,
          subject: jsonData.subject,
          additionalClaims: jsonData.additionalClaims,
          scopes: ["https://www.googleapis.com/auth/spreadsheets.readonly"],
        };
      } else {
        jwtOptions = { ...config.jwt };
        // スコープが設定されていない場合は追加
        if (!jwtOptions.scopes) {
          jwtOptions.scopes = ["https://www.googleapis.com/auth/spreadsheets.readonly"];
        }
      }

      const auth = new google.auth.JWT(jwtOptions);
      const sheets = google.sheets({ version: "v4", auth });

      const spreadsheet = await sheets.spreadsheets.get({
        spreadsheetId: config.path,
      });

      metadata = {
        properties: { title: spreadsheet.data.properties?.title || undefined },
        sheets: spreadsheet.data.sheets?.map((sheet) => ({
          properties: {
            title: sheet.properties?.title || undefined,
            gridProperties: {
              rowCount: sheet.properties?.gridProperties?.rowCount || undefined,
              columnCount: sheet.properties?.gridProperties?.columnCount || undefined,
            },
          },
        })),
      };

      const sheetName = config.sheetName ?? (spreadsheet.data.sheets?.[0]?.properties?.title || "Sheet1");
      const valuesResponse = await sheets.spreadsheets.values.get({
        spreadsheetId: config.path,
        range: toSheetRange(sheetName),
      });

      rows = (valuesResponse.data.values as string[][]) || [];
    }

    console.log(`\n✅ 接続成功!\n`);

    // 4. スプレッドシート情報の表示
    console.log("📊 スプレッドシート情報:");
    console.log(`  - タイトル: ${metadata.properties?.title || "不明"}`);
    console.log(`  - シート数: ${metadata.sheets?.length || 0}`);

    if (metadata.sheets && metadata.sheets.length > 0) {
      console.log(`  - 最初のシート名: ${metadata.sheets[0].properties?.title || "不明"}`);
      console.log(`  - 行数: ${metadata.sheets[0].properties?.gridProperties?.rowCount || "不明"}`);
      console.log(`  - 列数: ${metadata.sheets[0].properties?.gridProperties?.columnCount || "不明"}`);
    }

    // 5. データの取得テスト
    console.log("\n📥 ステップ4: データの取得テスト");

    console.log(`✅ データの取得に成功しました`);
    console.log(`  - 取得した行数: ${rows.length}`);

    if (rows.length > 0) {
      console.log(`  - ヘッダー行: ${rows[0].join(", ")}`);
      console.log(`  - データ行数: ${rows.length - 1}`);
    }

    // 6. データ形式の検証
    console.log("\n🔍 ステップ5: データ形式の検証");
    if (rows.length < 2) {
      console.warn("⚠️  データ行が不足しています");
      console.log("   ヘッダー行とデータ行が必要です");
    } else if (rows[0].length < 3) {
      console.warn("⚠️  列数が不足しています");
      console.log("   最低でも key, description, 言語1 の3列が必要です");
      console.log(`   現在の列数: ${rows[0].length}`);
    } else {
      console.log("✅ データ形式は正常です");
      console.log(`   - ロケール数: ${rows[0].length - 2}`);
      console.log(`   - ロケール: ${rows[0].slice(2).join(", ")}`);
    }

    console.log("\n🎉 すべての診断テストに合格しました!");
    console.log("\n次のステップ:");
    console.log(`  node lib/cli.js --config ${configFile}`);
    console.log("\nまたは:");
    console.log("  pnpm run test:sheets");
  } catch (error) {
    const errorMessage = (error as Error).message;
    console.error(`\n❌ 予期しないエラー: ${errorMessage}`);

    // OAuth2固有のエラーメッセージ
    if (config.credentialType === "oauth2") {
      if (errorMessage.includes("invalid_grant") || errorMessage.includes("Token has been expired")) {
        console.log("\n💡 解決方法:");
        console.log("   OAuth2トークンの有効期限が切れている可能性があります。");
        console.log("   トークン取得ヘルパーを再実行してください:");
        console.log("   node lib/helpers/oauth2-helper.js");
      } else if (errorMessage.includes("invalid_client")) {
        console.log("\n💡 解決方法:");
        console.log("   クライアントIDまたはクライアントシークレットが正しくありません。");
        console.log("   Google Cloud Consoleで確認してください:");
        console.log("   https://console.cloud.google.com/apis/credentials");
      }
    }

    // JWT固有のエラーメッセージ
    if (config.credentialType === "jwt") {
      if (errorMessage.includes("permission") || errorMessage.includes("Permission denied")) {
        console.log("\n💡 解決方法:");
        console.log("   スプレッドシートをService Accountと共有してください:");
        console.log(`   1. スプレッドシートを開く: https://docs.google.com/spreadsheets/d/${config.path}/edit`);
        console.log("   2. 右上の「共有」ボタンをクリック");

        // config.jwtが文字列の場合は、ファイルから読み込んでemailを取得
        let email = "SERVICE_ACCOUNT_EMAIL";
        if (typeof config.jwt === "string") {
          try {
            const fileContent = fs.readFileSync(config.jwt, "utf8");
            const jwtData = JSON.parse(fileContent);
            email = jwtData.client_email || email;
          } catch {
            // ファイル読み込みエラーは無視
          }
        } else if (config.jwt?.email) {
          email = config.jwt.email;
        }

        console.log(`   3. Service Accountのメールアドレスを追加: ${email}`);
        console.log("   4. 権限を「閲覧者」に設定");
      }
    }

    process.exit(1);
  }
}
