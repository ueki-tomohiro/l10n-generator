import fs from "fs";
import path from "path";
import yaml from "js-yaml";
import { Config } from "./type.js";

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

  // 5. outputTypeのデフォルト値設定
  if (!config.outputType) {
    config.outputType = "dart";
  }

  // 6. localizePathの末尾にスラッシュがなければ追加
  if (!config.localizePath.endsWith("/")) {
    config.localizePath = config.localizePath + "/";
  }

  return config;
};
