import { importValues } from "./helpers/importer.js";
import { createL10nFile, createTypeScriptL10nFiles } from "./helpers/exporter.js";
import { loadConfig } from "./helpers/config-loader.js";
import { SupportLocale } from "./helpers/localization.js";

/**
 * ヘッダー行からロケールリストを抽出
 * @param values インポートされたデータ（破壊されない）
 * @returns ロケールの配列
 */
const extractLocales = (values: string[][]): string[] => {
  if (values.length === 0) {
    throw new Error("データが空です");
  }
  // ヘッダー行の3列目以降がロケール
  const header = values[0];
  return header.slice(2);
};

/**
 * Dart ARB形式でエクスポート
 */
const exportDart = async (localizePath: string, values: string[][]): Promise<void> => {
  const locales = extractLocales(values);

  console.log(`Dart ARB形式でエクスポート中: ${locales.join(", ")}`);

  // createL10nFileは破壊的操作（pop）を行うため、各ロケールごとにコピーを渡す
  for (const locale of locales) {
    const valuesCopy = JSON.parse(JSON.stringify(values));
    await createL10nFile(localizePath, valuesCopy, locale as SupportLocale);
    console.log(`  - app_${locale}.arb を生成しました`);
  }
};

/**
 * TypeScript形式でエクスポート
 */
const exportTypeScript = async (localizePath: string, values: string[][]): Promise<void> => {
  const locales = extractLocales(values);

  console.log(`TypeScript形式でエクスポート中: ${locales.join(", ")}`);

  await createTypeScriptL10nFiles(localizePath, values);

  console.log(`  - translation.ts を生成しました`);
  console.log(`  - translateFunction.ts を生成しました`);
  locales.forEach((locale) => {
    console.log(`  - ${locale}.ts を生成しました`);
  });
};

/**
 * CLIメイン関数
 * @param configPath 設定ファイルのパス
 */
export const cli = async (configPath: string = "l10n-generator.config.yaml"): Promise<void> => {
  try {
    console.log("=== l10n-generator ローカライゼーションファイル生成 ===\n");

    // 1. 設定ファイル読み込み
    console.log(`設定ファイルを読み込み中: ${configPath}`);
    const config = loadConfig(configPath);
    console.log(`  - ファイルタイプ: ${config.fileType}`);
    console.log(`  - 認証方式: ${config.credentialType}`);
    console.log(`  - 出力形式: ${config.outputType || "dart"}`);
    console.log(`  - 出力先: ${config.localizePath}\n`);

    // 2. データインポート
    console.log("データをインポート中...");
    const values = await importValues(config);
    console.log(`  - ${values.length}行のデータを読み込みました\n`);

    if (values.length < 2) {
      throw new Error("データが不足しています（ヘッダー行とデータ行が必要です）");
    }

    // 3. 出力タイプに応じてエクスポート
    const outputType = config.outputType || "dart";

    switch (outputType) {
      case "dart":
        await exportDart(config.localizePath, values);
        break;

      case "typescript":
        await exportTypeScript(config.localizePath, values);
        break;

      case "both":
        await exportDart(config.localizePath, values);
        console.log(); // 空行
        await exportTypeScript(config.localizePath, values);
        break;

      default:
        throw new Error(`未知の出力形式です: ${outputType}`);
    }

    console.log("\n✓ ローカライゼーションファイルの生成が完了しました");
  } catch (error) {
    console.error("\n✗ エラーが発生しました:");
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  }
};
