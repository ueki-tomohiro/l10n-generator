# Rust 移行ガイド

このドキュメントは、`l10n-generator` を TypeScript から Rust へ段階移行するための実装方針と、現在の進捗をまとめたものです。

## 現在の進捗（フェーズ4）

`rust/l10n-rust` に Rust CLI を追加済みです。

### 対応済み

- YAML 設定ファイル読み込み
- CSV 読み込み
- Google Sheets 読み込み（`credentialType: none | apiKey | jwt | oauth2`）
- Dart ARB 生成
- TypeScript 出力（`translation.ts` / `translateFunction.ts` / `*.ts`）
- `outputType: dart | typescript | both`
- `diagnose` サブコマンド
- `oauth2-helper` サブコマンド
- TS/Rust 生成結果の一致検証コマンド（`pnpm run rust:parity`）
- Rust 検証用 GitHub Actions（`.github/workflows/rust-validation.yaml`）

### 未対応

- TypeScript実装の縮退・削除（切り替え作業）

## 実行方法

```bash
# Rust版をビルド
pnpm run rust:build

# Rust版のテスト
pnpm run rust:test

# 実行
pnpm run rust:run -- --config l10n-generator.config.yaml

# 診断
pnpm run rust:diagnose -- --config test.config.yaml

# OAuth2ヘルパー
pnpm run rust:oauth2-helper

# TS版とRust版の生成結果比較
pnpm run rust:parity
```

`--config` には既存YAMLを利用できます。  
現時点の Rust 実装でサポートする入力:

- `fileType: csv`
- `fileType: sheet` + `credentialType: none | apiKey | jwt | oauth2`

## 推奨移行ステップ

1. CSV経路を Rust に切り替え、生成差分を比較する
2. TypeScript実装を残したまま CI で並行検証する
3. TS実装を段階的に縮退・最終削除する

## 検証ポイント

- ARB出力の `@@locale` / `@key` メタ情報の一致
- TypeScript生成物（型定義・置換関数・各ロケール）の互換性
- CSVの特殊文字（カンマ・改行・引用符）の処理
- 実行時間とメモリ使用量の改善効果
