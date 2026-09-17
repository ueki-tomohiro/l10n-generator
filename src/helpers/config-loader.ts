import fs from "fs";
import path from "path";
import yaml from "js-yaml";
import { Config } from "./type.js";

const RESERVED_LOCALE_NAMES = ["key", "description", "__proto__"];

export const loadConfig = (configPath: string): Config => {
  // 1. 絶対パス解決
  const absolutePath = path.isAbsolute(configPath) ? configPath : path.resolve(process.cwd(), configPath);

  // 2. ファイル存在チェック
  if (!fs.existsSync(absolutePath)) {
    throw new Error(`設定ファイルが見つかりません: ${absolutePath}`);
  }

  // 3. YAMLファイルを読み込み
  let config: Config;
  try {
    const fileContents = fs.readFileSync(absolutePath, "utf8");
    config = yaml.load(fileContents) as Config;
  } catch (error) {
    throw new Error(`設定ファイルの読み込みに失敗しました: ${error instanceof Error ? error.message : error}`);
  }

  // 4. 基本的な検証
  if (!config.fileType || !config.path || !config.localizePath) {
    throw new Error("設定ファイルに必須フィールドが不足しています（fileType, path, localizePathが必要です）");
  }

  // 5. skipRows / columns の検証
  if (config.skipRows !== undefined && (!Number.isInteger(config.skipRows) || config.skipRows < 0)) {
    throw new Error(`skipRows は0以上の整数で指定してください: ${config.skipRows}`);
  }

  if (config.columns) {
    if (config.columns.key === undefined) {
      throw new Error("columns を指定する場合は columns.key が必要です");
    }
    if (
      !config.columns.locales ||
      typeof config.columns.locales !== "object" ||
      Array.isArray(config.columns.locales) ||
      Object.keys(config.columns.locales).length === 0
    ) {
      throw new Error("columns を指定する場合は columns.locales を1件以上指定してください");
    }
    Object.keys(config.columns.locales).forEach((locale) => {
      // ロケール名はそのまま出力ファイル名になるため、パス区切りなどを弾く
      if (!/^[A-Za-z0-9_-]+$/.test(locale)) {
        throw new Error(`columns.locales のロケール名に使用できない文字が含まれています: ${locale}`);
      }
      // 整数形式のキーは Object.entries で先頭に並び替えられ、定義順が崩れる
      if (/^\d+$/.test(locale)) {
        throw new Error(`columns.locales のロケール名に数字のみの名前は使用できません: ${locale}`);
      }
      // 正規化後のヘッダー名や JavaScript の特殊プロパティと衝突する
      if (RESERVED_LOCALE_NAMES.includes(locale)) {
        throw new Error(`columns.locales のロケール名に予約語は使用できません: ${locale}`);
      }
    });
  }

  // 6. outputTypeのデフォルト値設定
  if (!config.outputType) {
    config.outputType = "dart";
  }

  // 7. localizePathの末尾にスラッシュがなければ追加
  if (!config.localizePath.endsWith("/")) {
    config.localizePath = config.localizePath + "/";
  }

  return config;
};
