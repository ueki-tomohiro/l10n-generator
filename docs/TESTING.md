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

**概要**: Service Account認証は、サーバー間通信や本番環境に最適です。ユーザーの操作なしで自動的に認証できます。

##### ステップ1: Service Accountの作成

1. [Google Cloud Console](https://console.cloud.google.com/)にアクセス
2. プロジェクトを選択（または新規作成）
3. **IAM と管理** → **サービスアカウント**を開く
4. **サービスアカウントを作成**をクリック

##### ステップ2: サービスアカウントの設定

1. **サービスアカウント名**を入力（例: `l10n-generator`）
2. **説明**を入力（例: `Localization file generator`）
3. **作成して続行**をクリック
4. 役割は設定不要（スキップして**完了**をクリック）

##### ステップ3: JSONキーの作成とダウンロード

1. 作成したサービスアカウントをクリック
2. **キー**タブを開く
3. **鍵を追加** → **新しい鍵を作成**
4. **JSON**を選択して**作成**をクリック
5. JSONファイルが自動的にダウンロードされます

**ダウンロードされたJSONファイルの例**:

```json
{
  "type": "service_account",
  "project_id": "your-project-id",
  "private_key_id": "abc123...",
  "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBA...\n-----END PRIVATE KEY-----\n",
  "client_email": "l10n-generator@your-project-id.iam.gserviceaccount.com",
  "client_id": "123456789",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/..."
}
```

##### ステップ4: スプレッドシートの共有

JSONファイル内の`client_email`の値をコピーして、スプレッドシートと共有します。

1. Google Sheetsでスプレッドシートを開く
2. 右上の**共有**ボタンをクリック
3. `client_email`のアドレス（例: `l10n-generator@your-project-id.iam.gserviceaccount.com`）を入力
4. 権限を**閲覧者**に設定
5. **送信**をクリック

##### ステップ5: 設定ファイルの作成

ダウンロードしたJSONファイルを、設定ファイルの`jwt`フィールドに**ファイルパス**として指定します（推奨）。

**test-jwt.config.yaml** (推奨):

```yaml
fileType: sheet
path: YOUR_SPREADSHEET_ID_HERE
credentialType: jwt
jwt: ./path/to/service-account-key.json # JSONファイルのパス
localizePath: ./test-output/
outputType: both
```

**重要な注意事項**:

- JSONファイルには機密情報が含まれているため、**.gitignore**に追加して公開しないようにしてください
- ファイルパスは、設定ファイルからの相対パスまたは絶対パスで指定できます
- JSONファイルを直接編集する必要はありません

##### 代替方法: JSONの内容を直接記載

JSONファイルの内容を、設定ファイルに直接記載することもできます。

**test-jwt.config.yaml** (代替):

```yaml
fileType: sheet
path: YOUR_SPREADSHEET_ID_HERE
credentialType: jwt
jwt:
  type: service_account
  project_id: your-project-id
  private_key_id: abc123...
  private_key: "-----BEGIN PRIVATE KEY-----\nMIIEvQIBA...\n-----END PRIVATE KEY-----\n"
  client_email: l10n-generator@your-project-id.iam.gserviceaccount.com
  client_id: "123456789"
  auth_uri: https://accounts.google.com/o/oauth2/auth
  token_uri: https://oauth2.googleapis.com/token
  auth_provider_x509_cert_url: https://www.googleapis.com/oauth2/v1/certs
  client_x509_cert_url: https://www.googleapis.com/robot/v1/metadata/x509/...
localizePath: ./test-output/
outputType: both
```

**注意**: `private_key`は改行を含む文字列なので、YAMLでは引用符で囲む必要があります

##### ステップ6: 接続テスト

診断コマンドで接続をテストします:

```bash
pnpm build
node lib/cli.js diagnose --config test-jwt.config.yaml
```

成功すると以下のように表示されます:

```text
🔍 Google Sheets API 接続診断ツール

📋 ステップ1: 設定ファイルの確認
✓ test-jwt.config.yaml を検出しました

📖 ステップ2: 設定の読み込み
✓ 設定を読み込みました
  - ファイルタイプ: sheet
  - 認証方式: jwt
  - Sheet ID: YOUR_SPREADSHEET_ID_HERE
  - Service Account Email: l10n-generator@your-project-id.iam.gserviceaccount.com

🌐 ステップ3: Google Sheets APIへの接続テスト
  接続中...

✅ 接続成功!
```

##### トラブルシューティング

###### エラー: Permission denied

```text
❌ 予期しないエラー: permission denied

💡 解決方法:
   スプレッドシートをService Accountと共有してください:
   1. スプレッドシートを開く: https://docs.google.com/spreadsheets/d/YOUR_SPREADSHEET_ID/edit
   2. 右上の「共有」ボタンをクリック
   3. Service Accountのメールアドレスを追加: l10n-generator@your-project-id.iam.gserviceaccount.com
   4. 権限を「閲覧者」に設定
```

→ スプレッドシートがService Accountと共有されていません。ステップ4を確認してください。

###### エラー: JWT credentials are required

→ 設定ファイルの`jwt`フィールドが正しく設定されていません。JSONファイルの内容を確認してください。

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

Rust版の検証:

```bash
pnpm run rust:test
pnpm run rust:diagnose -- --config test.config.yaml
pnpm run rust:parity
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
