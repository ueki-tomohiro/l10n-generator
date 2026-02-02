# CLI 使用方法ガイド

このドキュメントでは、l10n-generator の CLI の使用方法を詳しく説明します。

## 基本的な使い方

### 1. 通常の実行

デフォルトの設定ファイル(`l10n-generator.config.yaml`)を使用:

```bash
l10n-generator
```

カスタム設定ファイルを指定:

```bash
l10n-generator --config custom.config.yaml
```

### 2. 診断コマンド

Google Sheets API の接続をテストし、問題を診断します:

```bash
l10n-generator diagnose
```

カスタム設定ファイルで診断:

```bash
l10n-generator diagnose --config test.config.yaml
```

### 3. ヘルプの表示

```bash
l10n-generator --help
```

### 4. バージョンの表示

```bash
l10n-generator --version
```

## 診断コマンドの詳細

診断コマンド(`diagnose`)は、Google Sheets API への接続に問題がある場合に使用します。

### 診断内容

1. **設定ファイルの確認**

   - ファイルの存在確認
   - YAML 形式の妥当性チェック

2. **設定の読み込み**

   - ファイルタイプ(CSV/Sheet)
   - 認証方式の確認
   - API キー/Sheet ID の設定確認

3. **Google Sheets API への接続テスト**

   - メタデータの取得
   - 権限エラーの検出
   - API キーの有効性確認

4. **スプレッドシート情報の表示**

   - タイトル
   - シート数
   - 行数・列数

5. **データの取得テスト**

   - 実際のデータ取得
   - ヘッダー行の確認

6. **データ形式の検証**
   - 最低限必要な列数のチェック
   - ロケール情報の抽出

### 診断結果の例

#### 成功時

```
🔍 Google Sheets API 接続診断ツール

📋 ステップ1: 設定ファイルの確認
✓ test.config.yaml を検出しました

📖 ステップ2: 設定の読み込み
✓ 設定を読み込みました
  - ファイルタイプ: sheet
  - 認証方式: apiKey
  - Sheet ID: 1A2B3C4D5E6F7G8H9I0J
  - API Key: AIzaSyBq8w...

🌐 ステップ3: Google Sheets APIへの接続テスト
  接続中...

✅ 接続成功!

📊 スプレッドシート情報:
  - タイトル: l10n-test-data
  - シート数: 1
  - 最初のシート名: Sheet1
  - 行数: 5
  - 列数: 4

📥 ステップ4: データの取得テスト
✅ データの取得に成功しました
  - 取得した行数: 5
  - ヘッダー行: key, description, ja, en
  - データ行数: 4

🔍 ステップ5: データ形式の検証
✅ データ形式は正常です
   - ロケール数: 2
   - ロケール: ja, en

🎉 すべての診断テストに合格しました!

次のステップ:
  node lib/cli.js --config test.config.yaml
```

#### エラー時(権限エラー)

```
❌ 接続エラー: The caller does not have permission

💡 解決方法:
   1. スプレッドシートを開く:
      https://docs.google.com/spreadsheets/d/1A2B3C4D5E6F7G8H9I0J/edit
   2. 右上の「共有」ボタンをクリック
   3. 「リンクを知っている全員」に変更
   4. 権限を「閲覧者」に設定
```

## トラブルシューティング

### エラー: "API key not valid"

**原因**: API キーが無効または期限切れ

**解決方法**:

1. Google Cloud Console で新しい API キーを作成
2. 設定ファイルの API キーを更新
3. Google Sheets API が有効化されているか確認

### エラー: "The caller does not have permission"

**原因**: スプレッドシートの共有設定が制限されている

**解決方法**:

1. スプレッドシートを開く
2. 「共有」ボタンをクリック
3. 「リンクを知っている全員」に変更
4. 権限を「閲覧者」に設定

### エラー: "not found" または "Not found"

**原因**: Sheet ID が間違っている

**解決方法**:

1. スプレッドシートの URL を確認
2. `/d/`と`/edit`の間の文字列をコピー
3. 設定ファイルの path を更新

### エラー: "データ行が不足しています"

**原因**: スプレッドシートにヘッダー行またはデータ行がない

**解決方法**:

1. スプレッドシートを開く
2. 最低限以下の 2 行を追加:
   - 1 行目: ヘッダー行(key, description, ja, en)
   - 2 行目: データ行

### エラー: "列数が不足しています"

**原因**: 必要な列数が足りない

**解決方法**:
最低限以下の 3 列が必要です:

- key: キー名
- description: 説明
- 言語 1: 最低 1 つの言語列(例: ja, en)

## 実行例

### CSV 形式の場合

```bash
# デフォルト設定で実行
l10n-generator

# カスタム設定で実行
l10n-generator --config csv.config.yaml
```

### Google Sheets + API Key 認証の場合

```bash
# まず診断を実行
l10n-generator diagnose --config sheet.config.yaml

# 診断が成功したら実行
l10n-generator --config sheet.config.yaml
```

### Google Sheets + JWT 認証の場合

```bash
# JWTの場合は診断コマンドは使えません
l10n-generator --config jwt.config.yaml
```

## npm スクリプトとの統合

`package.json`に以下を追加:

```json
{
  "scripts": {
    "i18n": "l10n-generator",
    "i18n:test": "l10n-generator --config test.config.yaml",
    "i18n:diagnose": "l10n-generator diagnose --config test.config.yaml"
  }
}
```

実行:

```bash
# 本番用
npm run i18n

# テスト用
npm run i18n:test

# 診断
npm run i18n:diagnose
```

## 参考資料

- [README.md](../README.md) - プロジェクト全体のドキュメント
- [QUICKSTART.md](./QUICKSTART.md) - 5 分でできるクイックスタート
- [TESTING.md](./TESTING.md) - テスト環境の詳細設定
- [examples/](../examples/) - 設定ファイルの例
