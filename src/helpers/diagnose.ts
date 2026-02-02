import fs from "fs";
import yaml from "js-yaml";
import fetch from "node-fetch";
import { Config } from "./type";

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

    if (config.fileType === "sheet" && config.credentialType === "apiKey") {
      console.log(`  - Sheet ID: ${config.path}`);
      console.log(`  - API Key: ${config.apiKey ? config.apiKey.substring(0, 10) + "..." : "未設定"}\n`);
    } else if (config.fileType === "csv") {
      console.log(`  - CSV Path: ${config.path}\n`);
      console.log("✓ CSV形式の設定です。診断はGoogle Sheets専用です。");
      console.log("\n次のステップ:");
      console.log(`  node lib/cli.js --config ${configFile}`);
      process.exit(0);
    } else {
      console.log("\n⚠️  この診断ツールはGoogle Sheets + API Key認証専用です");
      process.exit(0);
    }
  } catch (error) {
    console.error(`❌ 設定ファイルの読み込みに失敗: ${(error as Error).message}`);
    process.exit(1);
  }

  if (!config.apiKey || config.apiKey.includes("YOUR_API_KEY")) {
    console.error("❌ APIキーが設定されていません");
    console.log("\n📝 Google Cloud ConsoleでAPIキーを取得してください:");
    console.log("   https://console.cloud.google.com/apis/credentials");
    process.exit(1);
  }

  if (!config.path || config.path.includes("YOUR_SHEET_ID")) {
    console.error("❌ Sheet IDが設定されていません");
    console.log("\n📝 Google SheetsのURLからSheet IDをコピーしてください");
    process.exit(1);
  }

  // 3. Google Sheets APIへの接続テスト
  console.log("🌐 ステップ3: Google Sheets APIへの接続テスト");
  const metadataUrl = `https://sheets.googleapis.com/v4/spreadsheets/${config.path}?key=${config.apiKey}`;

  try {
    console.log("  接続中...");
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

    const metadata = (await response.json()) as {
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
    };

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
    const sheetName = metadata.sheets?.[0]?.properties?.title || "Sheet1";
    const valuesUrl = `https://sheets.googleapis.com/v4/spreadsheets/${config.path}/values/${encodeURIComponent(sheetName)}?key=${config.apiKey}`;

    const valuesResponse = await fetch(valuesUrl);
    if (!valuesResponse.ok) {
      const error = (await valuesResponse.json()) as { error?: { message?: string } };
      console.error(`❌ データ取得エラー: ${error.error?.message || valuesResponse.statusText}`);
      process.exit(1);
    }

    const data = (await valuesResponse.json()) as { values?: string[][] };
    const rows = data.values || [];

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
    console.error(`\n❌ 予期しないエラー: ${(error as Error).message}`);
    process.exit(1);
  }
}
