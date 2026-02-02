import fs from 'fs';
import path from 'path';
import os from 'os';
import yaml from 'js-yaml';
import { Config } from '../helpers/type';

/**
 * 一時ディレクトリを作成
 */
export const createTempDir = (): string => {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'l10n-test-'));
};

/**
 * 一時ディレクトリをクリーンアップ
 */
export const cleanupTempDir = (dir: string): void => {
  if (fs.existsSync(dir)) {
    fs.rmSync(dir, { recursive: true, force: true });
  }
};

/**
 * テスト用のCSVファイルを作成
 * 特殊文字を含むフィールドは引用符で囲む
 */
export const createTestCSV = (dir: string, data: string[][], filename: string = 'test.csv'): string => {
  const csvPath = path.join(dir, filename);
  const csvContent = data.map(row =>
    row.map(field => {
      // カンマや引用符を含む場合は引用符で囲む
      if (field.includes(',') || field.includes('"') || field.includes('\n')) {
        return `"${field.replace(/"/g, '""')}"`;
      }
      return field;
    }).join(',')
  ).join('\n');
  fs.writeFileSync(csvPath, csvContent, 'utf-8');
  return csvPath;
};

/**
 * テスト用の設定ファイルを作成
 */
export const createTestConfig = (dir: string, config: Partial<Config>, filename: string = 'config.yaml'): string => {
  const configPath = path.join(dir, filename);
  fs.writeFileSync(configPath, yaml.dump(config), 'utf-8');
  return configPath;
};

/**
 * 生成されたARBファイルを読み込み
 */
export const readGeneratedARB = (filePath: string): any => {
  const content = fs.readFileSync(filePath, 'utf-8');
  return JSON.parse(content);
};

/**
 * 生成されたTypeScriptファイルを読み込み
 */
export const readGeneratedTS = (filePath: string): string => {
  return fs.readFileSync(filePath, 'utf-8');
};

/**
 * 実際の内容と期待される内容を比較
 */
export const compareContent = (actual: string, expected: string): void => {
  expect(actual.trim()).toBe(expected.trim());
};

/**
 * 基本的なテストデータ
 */
export const basicTestData = [
  ['key', 'description', 'ja', 'en'],
  ['hello', 'Greeting', 'こんにちは', 'Hello'],
  ['goodbye', 'Farewell', 'さようなら', 'Goodbye']
];

/**
 * パラメータ付きテストデータ
 */
export const paramTestData = [
  ['key', 'description', 'ja', 'en'],
  ['welcome', 'Welcome message', 'ようこそ{name}さん', 'Welcome {name}'],
  ['error_count', 'Error count', '{count}件のエラー', '{count} errors']
];

/**
 * 複数ロケールテストデータ
 */
export const multiLocaleTestData = [
  ['key', 'description', 'ja', 'en', 'es'],
  ['hello', 'Greeting', 'こんにちは', 'Hello', 'Hola'],
  ['goodbye', 'Farewell', 'さようなら', 'Goodbye', 'Adiós']
];
