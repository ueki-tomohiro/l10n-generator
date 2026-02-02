# テスト環境のセットアップガイド

このドキュメントでは、Google Sheets API を使用した l10n-generator のテスト環境を構築する方法を説明します。

## 📋 事前準備

### 1. テスト用の Google Spreadsheet を作成

1. [Google Sheets](https://sheets.google.com)で新しいスプレッドシートを作成
2. 以下の形式でデータを入力:

| key       | description        | ja                   | en              |
| --------- | ------------------ | -------------------- | --------------- |
| hello     | Greeting           | こんにちは           | Hello           |
| goodbye   | Farewell           | さようなら           | Goodbye         |
| welcome   | Welcome message    | ようこそ、{name}さん | Welcome, {name} |
| itemCount | Item count message | {count}個のアイテム  | {count} items   |

3. スプレッドシートの URL をコピー
   - 例: `https://docs.google.com/spreadsheets/d/1A2B3C4D5E6F7G8H9I0J/edit`
   - Sheet ID は `/d/` の後の文字列部分: `1A2B3C4D5E6F7G8H9I0J`

### 2. Google Cloud Console の設定

#### 方法 A: API Key 認証(推奨・最も簡単)

1. [Google Cloud Console](https://console.cloud.google.com/)にアクセス
2. プロジェクトを作成または選択
3. **API とサービス** → **ライブラリ** → 「Google Sheets API」を検索して有効化
4. **認証情報** → **認証情報を作成** → **API キー** を選択
5. 作成された API キーをコピー
6. **重要**: スプレッドシートの共有設定を「リンクを知っている全員に表示」に変更

#### 方法 B: Service Account (JWT)認証

1. [Google Cloud Console](https://console.cloud.google.com/)にアクセス
2. **IAM と管理** → **サービスアカウント** → **サービスアカウントを作成**
3. サービスアカウントを作成後、**キーを作成** → **JSON**を選択
4. ダウンロードした JSON ファイルを保存
5. スプレッドシートをサービスアカウントのメールアドレス(JSON 内の`client_email`)と共有

## 🔧 環境設定

### .env ファイルの作成

```bash
cp .env.example .env
```

`.env`ファイルに実際の値を設定:

```env
GOOGLE_API_KEY=AIza...your_actual_api_key
GOOGLE_SHEET_ID=1A2B3C4D5E6F7G8H9I0J
```

### テスト設定ファイルの編集

`test.config.yaml`を編集:

```yaml
fileType: sheet
path: 1A2B3C4D5E6F7G8H9I0J # あなたのSheet ID
credentialType: apiKey
apiKey: AIza...your_actual_api_key # あなたのAPIキー
localizePath: ./test-output/
outputType: both
```

## 🚀 テスト実行

### 1. ビルド

```bash
pnpm build
```

### 2. テスト実行

```bash
node lib/cli.js --config test.config.yaml
```

### 3. 出力の確認

成功すると `test-output/` ディレクトリに以下のファイルが生成されます:

#### Dart ARB ファイル

- `app_ja.arb`
- `app_en.arb`

#### TypeScript ファイル

- `translation.ts` - 型定義
- `translateFunction.ts` - ヘルパー関数
- `ja.ts` - 日本語翻訳
- `en.ts` - 英語翻訳

## 🐛 トラブルシューティング

### エラー: "Failed to fetch spreadsheet metadata"

- API キーが正しいか確認
- Google Sheets API が有効化されているか確認
- スプレッドシートの共有設定を確認(「リンクを知っている全員」)

### エラー: "API key is required"

- `test.config.yaml`の`apiKey`フィールドが設定されているか確認
- API キーに余分なスペースや改行が含まれていないか確認

### エラー: "Invalid Google Sheets URL"

- Sheet ID が正しいか確認
- `/d/`と`/edit`の間の文字列をコピーしているか確認

## 📝 他の認証方法のテスト

### JWT 認証のテスト

`test-jwt.config.yaml`を作成:

```yaml
fileType: sheet
path: https://docs.google.com/spreadsheets/d/YOUR_SHEET_ID/edit
credentialType: jwt
jwt:
  email: your-service-account@project-id.iam.gserviceaccount.com
  key: "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
  scopes:
    - https://www.googleapis.com/auth/spreadsheets.readonly
localizePath: ./test-output/
outputType: both
```

実行:

```bash
node lib/cli.js --config test-jwt.config.yaml
```

## 🔒 セキュリティに関する注意

- `.env`ファイルと`test.config.yaml`は`.gitignore`に追加済みです
- API キーやサービスアカウントキーを Git にコミットしないでください
- 本番環境では環境変数を使用することを推奨します

## 📚 参考リンク

- [Google Sheets API ドキュメント](https://developers.google.com/sheets/api)
- [Google Cloud Console](https://console.cloud.google.com/)
- [プロジェクトの README](../README.md)
