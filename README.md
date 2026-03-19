# l10n-generator

> Google SheetsまたはCSVファイルから、Dart ARBファイルとTypeScriptのローカライゼーションファイルを自動生成するCLIツール

[![Issues](https://img.shields.io/github/issues/ueki-tomohiro/l10n-generator?style=flat&color=336791)](https://github.com/ueki-tomohiro/l10n-generator/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/ueki-tomohiro/l10n-generator?style=flat&color=336791)](https://github.com/ueki-tomohiro/l10n-generator/pulls)
[![GitHub release](https://img.shields.io/github/release/ueki-tomohiro/l10n-generator.svg?style=flat&color=336791)](https://github.com/ueki-tomohiro/l10n-generator)

## 📚 ドキュメント

詳細なドキュメントは[docsディレクトリ](./docs/)を参照してください:

- 🚀 [クイックスタート](./docs/QUICKSTART.md) - 5分で始める
- 💻 [CLI使用方法](./docs/CLI-USAGE.md) - 完全ガイド
- 🧪 [テスト環境](./docs/TESTING.md) - 環境構築
- 🔐 [OAuth2認証](./docs/OAUTH2-QUICKSTART.md) - プライベートシート対応
- 🦀 [Rust移行ガイド](./docs/RUST-MIGRATION.md) - Rust版への段階移行

## ✨ 特徴

- 📝 **複数のデータソース対応**: CSV、Google Sheets（API Key、OAuth2、JWT認証）
- 🎯 **複数の出力形式**: Dart ARB、TypeScript型定義 + 各言語ファイル
- 🔧 **YAML設定ファイル**: シンプルで読みやすい設定
- 🚀 **npxで即実行**: インストール不要で実行可能
- 🌍 **多言語サポート**: 任意の数の言語に対応

## 📦 インストール

### グローバルインストール

```bash
npm install -g l10n-generator
```

### プロジェクトにインストール

```bash
npm install --save-dev l10n-generator
# or
pnpm add -D l10n-generator
```

### npxで直接実行（インストール不要）

```bash
npx l10n-generator --config your-config.yaml
```

## 🚀 クイックスタート

### 1. 設定ファイルを作成

プロジェクトルートに`l10n-generator.config.yaml`を作成します。

```yaml
fileType: csv
path: ./localization.csv
credentialType: none
localizePath: ./src/i18n/
outputType: both # dart | typescript | both
```

### 2. ローカライゼーションデータを準備

CSVファイルの形式:

```csv
key,description,ja,en
hello,Greeting,こんにちは,Hello
goodbye,Farewell,さようなら,Goodbye
welcome,Welcome message,ようこそ、{name}さん,"Welcome, {name}"
```

- 1列目: キー（変数名）
- 2列目: 説明
- 3列目以降: 各言語の翻訳テキスト

### 3. 実行

```bash
# デフォルト設定ファイルを使用
l10n-generator

# カスタム設定ファイルを指定
l10n-generator --config custom.config.yaml

# npxで実行
npx l10n-generator --config your-config.yaml
```

## 🦀 Rust移行フェーズ2a（CSV + Google Sheets API Key/Public対応）

Rust版CLIは `rust/l10n-rust` に追加されています。  
現時点では `fileType: csv` と `fileType: sheet` の全認証方式（`credentialType: none | apiKey | jwt | oauth2`）に対応し、`dart/typescript/both` の生成を実行できます。

```bash
# ビルド
pnpm run rust:build

# テスト
pnpm run rust:test

# 実行（--config をそのまま渡せます）
pnpm run rust:run -- --config l10n-generator.config.yaml
```

## 📤 出力形式

### Dart ARB形式 (`outputType: dart`)

```text
output/
├── app_ja.arb
└── app_en.arb
```

`app_ja.arb`の内容例:

```json
{
  "@@locale": "ja",
  "hello": "こんにちは",
  "@hello": {
    "description": "Greeting"
  },
  "welcome": "ようこそ、{name}さん",
  "@welcome": {
    "description": "Welcome message",
    "placeholders": {
      "name": {
        "type": "String",
        "example": "name"
      }
    }
  }
}
```

### TypeScript形式 (`outputType: typescript`)

```text
output/
├── translation.ts         # 型定義
├── translateFunction.ts   # ヘルパー関数
├── ja.ts                  # 日本語翻訳
└── en.ts                  # 英語翻訳
```

`translation.ts`の内容例:

```typescript
export interface Translation {
  /**
   * こんにちは: Greeting
   */
  hello: string;
  /**
   * ようこそ、{name}さん: Welcome message
   */
  welcome: string;
}
```

`ja.ts`の内容例:

```typescript
import { Translation } from "./translation";

export const translation: Translation = {
  hello: "こんにちは",
  welcome: "ようこそ、{name}さん",
};
```

## ⚙️ 設定ファイルの詳細

### 基本設定

| フィールド       | 型                                        | 必須 | 説明                                  |
| ---------------- | ----------------------------------------- | ---- | ------------------------------------- |
| `fileType`       | `"csv" \| "sheet"`                        | ✅   | データソースの種類                    |
| `path`           | `string`                                  | ✅   | ファイルパスまたはGoogle Sheet ID/URL |
| `credentialType` | `"none" \| "apiKey" \| "oauth2" \| "jwt"` | ✅   | 認証方式                              |
| `localizePath`   | `string`                                  | ✅   | 出力先ディレクトリ                    |
| `outputType`     | `"dart" \| "typescript" \| "both"`        | -    | 出力形式（デフォルト: `dart`）        |

### 設定例

詳細な設定例は[examples](./examples)ディレクトリを参照してください。

- [CSV + Dart](./examples/csv-dart.config.yaml)
- [CSV + TypeScript](./examples/csv-typescript.config.yaml)
- [Google Sheets + API Key](./examples/sheet-apikey.config.yaml)
- [Google Sheets + OAuth2](./examples/sheet-oauth2.config.yaml)
- [Google Sheets + JWT](./examples/sheet-jwt.config.yaml)

## 🔧 Google Sheets の設定

### API Keyを使用する場合

1. [Google Cloud Console](https://console.cloud.google.com/)でプロジェクトを作成
2. Google Sheets APIを有効化
3. APIキーを作成
4. スプレッドシートを「リンクを知っている全員」に共有設定

### OAuth2を使用する場合

1. [Google Cloud Console](https://console.cloud.google.com/)でOAuth 2.0クライアントIDを作成
2. トークン取得ヘルパーを実行してトークンを取得

   ```bash
   node lib/helpers/oauth2-helper.js
   ```

3. 取得したトークンを設定ファイルに追加

詳しくは[OAUTH2-SETUP.md](./docs/OAUTH2-SETUP.md)を参照してください。

### Service Account（JWT）を使用する場合

1. [Google Cloud Console](https://console.cloud.google.com/)でサービスアカウントを作成
2. JSONキーファイルをダウンロード
3. スプレッドシートをサービスアカウントのメールアドレスと共有
4. JSONの内容を設定ファイルの`jwt`フィールドに記載

## 💻 CLIオプション

```bash
l10n-generator [オプション]

コマンド:
  l10n-generator diagnose  Google Sheets API接続の診断

オプション:
  --config     設定ファイルのパス (デフォルト: l10n-generator.config.yaml)
  --diagnose   接続診断を実行
  -h, --help   ヘルプを表示
  --version    バージョンを表示

例:
  l10n-generator                          デフォルト設定ファイルで生成
  l10n-generator --config custom.yaml     カスタム設定で生成
  l10n-generator diagnose                 test.config.yamlで診断実行
  l10n-generator diagnose --config custom.yaml  カスタム設定で診断
```

### 診断コマンド

Google Sheets APIの接続に問題がある場合、診断コマンドで原因を特定できます:

```bash
l10n-generator diagnose --config test.config.yaml
```

診断コマンドは以下をチェックします:

- 設定ファイルの形式
- APIキーの有効性
- Google Sheets APIの有効化状態
- スプレッドシートの共有設定
- データ形式の妥当性

## 📝 スクリプトに組み込む

`package.json`にスクリプトを追加:

```json
{
  "scripts": {
    "i18n": "l10n-generator",
    "i18n:watch": "nodemon --watch localization.csv --exec l10n-generator"
  }
}
```

実行:

```bash
npm run i18n
```

## 🐛 トラブルシューティング

### 出力ディレクトリが見つからない

出力先ディレクトリは自動作成されません。事前に作成してください:

```bash
mkdir -p src/i18n
```

### Google Sheets APIエラー

- API Keyが正しいか確認
- Google Sheets APIが有効化されているか確認
- スプレッドシートの共有設定を確認

詳しいテスト環境のセットアップ手順は[TESTING.md](./docs/TESTING.md)を参照してください。

## 🤝 Contributing

Contributions, issues and feature requests are welcome!
Feel free to check [issues page](https://github.com/ueki-tomohiro/l10n-generator/issues).

## 📝 License

Copyright © 2022-2026 [Tomohiro Ueki](https://github.com/ueki-tomohiro).
This project is [MIT](LICENSE) licensed.
