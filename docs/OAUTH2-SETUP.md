# OAuth2 セットアップガイド

このガイドでは、Google Sheets API で OAuth2 認証を使用する方法を説明します。

## 📋 OAuth2 とは

OAuth2 は、ユーザーがアプリケーションに権限を委譲する標準的な認証プロトコルです。

### API Key との違い

| 項目                 | API Key                        | OAuth2                                             |
| -------------------- | ------------------------------ | -------------------------------------------------- |
| 認証方法             | API キーのみ                   | ユーザーログイン                                   |
| スプレッドシート共有 | 「リンクを知っている全員」必要 | 不要（ログインユーザーがアクセス可能なものすべて） |
| セキュリティ         | 低（キーが漏れると危険）       | 高（ユーザーごとに認証）                           |
| 使用ケース           | 公開スプレッドシート           | プライベートスプレッドシート                       |

## 🚀 セットアップ手順

### ステップ 1: Google Cloud Console でプロジェクトを作成

1. [Google Cloud Console](https://console.cloud.google.com/)にアクセス
2. 新しいプロジェクトを作成（または既存のものを選択）

### ステップ 2: Google Sheets API を有効化

1. **API とサービス** → **ライブラリ**
2. 「Google Sheets API」を検索
3. **有効にする** をクリック

### ステップ 3: OAuth 同意画面を設定

1. **API とサービス** → **OAuth 同意画面**
2. ユーザータイプを選択:
   - **外部**: 誰でもアクセス可能（テストユーザーを追加可能）
   - **内部**: Google Workspace の組織内のみ
3. アプリ情報を入力:
   - **アプリ名**: l10n-generator（任意）
   - **ユーザーサポートメール**: あなたのメールアドレス
   - **デベロッパーの連絡先情報**: あなたのメールアドレス
4. **保存して次へ**

### ステップ 4: スコープを追加

1. **スコープを追加または削除** をクリック
2. 以下のスコープを追加:
   ```
   https://www.googleapis.com/auth/spreadsheets.readonly
   ```
3. **保存して次へ**

### ステップ 5: テストユーザーを追加（外部を選択した場合）

1. **テストユーザーを追加** をクリック
2. 使用する Google アカウントのメールアドレスを入力
3. **保存して次へ**

### ステップ 6: OAuth 2.0 クライアント ID を作成

1. **API とサービス** → **認証情報**
2. **認証情報を作成** → **OAuth クライアント ID**
3. アプリケーションの種類を選択:
   - **デスクトップアプリ**（推奨）
   - 名前: l10n-generator-cli（任意）
4. **作成** をクリック
5. 表示されるクライアント ID とクライアントシークレットをコピー

## 🔑 認証トークンの取得

OAuth2 を使用するには、最初に認証フローを実行してトークンを取得する必要があります。

### 方法 1: 自動化スクリプトを使用（推奨）

専用のヘルパースクリプトを用意しています:

```bash
node lib/helpers/oauth2-helper.js
```

このスクリプトは:

1. ブラウザで認証 URL を開く
2. ユーザーが Google アカウントでログイン
3. 認証コードを取得
4. トークンを取得して表示

### 方法 2: 手動で取得

#### 2-1. 認証 URL を生成

```javascript
const { OAuth2Client } = require("google-auth-library");

const oauth2Client = new OAuth2Client({
  clientId: "YOUR_CLIENT_ID",
  clientSecret: "YOUR_CLIENT_SECRET",
  redirectUri: "http://localhost:3000/oauth2callback",
});

const authUrl = oauth2Client.generateAuthUrl({
  access_type: "offline",
  scope: ["https://www.googleapis.com/auth/spreadsheets.readonly"],
});

console.log("Authorize this app by visiting this url:", authUrl);
```

#### 2-2. ブラウザで認証

1. 生成された URL をブラウザで開く
2. Google アカウントでログイン
3. アクセスを許可
4. リダイレクト URL の`code`パラメータをコピー

#### 2-3. トークンを取得

```javascript
const { tokens } = await oauth2Client.getToken(code);
console.log("Tokens:", tokens);
```

## ⚙️ 設定ファイルの作成

トークンを取得したら、設定ファイルを作成します。

### test-oauth2.config.yaml

```yaml
fileType: sheet
path: YOUR_SPREADSHEET_ID
credentialType: oauth2
oauth2:
  clientId: YOUR_CLIENT_ID.apps.googleusercontent.com
  clientSecret: YOUR_CLIENT_SECRET
  redirectUri: http://localhost:3000/oauth2callback
  # 取得したトークン
  refreshToken: YOUR_REFRESH_TOKEN
  accessToken: YOUR_ACCESS_TOKEN # オプション（有効期限あり）
localizePath: ./test-output/
outputType: both
```

### トークンについて

- **Access Token**: 短期間有効（通常 1 時間）
- **Refresh Token**: 長期間有効（アクセストークンの更新に使用）

リフレッシュトークンがあれば、アクセストークンは自動的に更新されます。

## 🧪 テスト

### 診断を実行

```bash
node lib/cli.js diagnose --config test-oauth2.config.yaml
```

### ローカライゼーションファイルを生成

```bash
node lib/cli.js --config test-oauth2.config.yaml
```

## 🔒 セキュリティのベストプラクティス

### 1. 設定ファイルを保護

```bash
# .gitignoreに追加済み
test-oauth2.config.yaml
*.oauth2.yaml
```

### 2. トークンを環境変数で管理

```bash
# .envファイル
OAUTH2_CLIENT_ID=your_client_id
OAUTH2_CLIENT_SECRET=your_client_secret
OAUTH2_REFRESH_TOKEN=your_refresh_token
```

設定ファイルで環境変数を参照:

```yaml
oauth2:
  clientId: ${OAUTH2_CLIENT_ID}
  clientSecret: ${OAUTH2_CLIENT_SECRET}
  refreshToken: ${OAUTH2_REFRESH_TOKEN}
```

### 3. トークンのローテーション

定期的にトークンを再生成することをお勧めします。

## ❗ トラブルシューティング

### エラー: "invalid_grant"

**原因**: リフレッシュトークンが無効または期限切れ

**解決方法**:

1. 認証フローを再実行
2. 新しいトークンを取得
3. 設定ファイルを更新

### エラー: "redirect_uri_mismatch"

**原因**: リダイレクト URI が一致しない

**解決方法**:

1. Google Cloud Console で設定したリダイレクト URI を確認
2. 設定ファイルの`redirectUri`を一致させる

### エラー: "access_denied"

**原因**: ユーザーがアクセスを拒否した

**解決方法**:
認証フローを再実行し、アクセスを許可してください

## 📚 参考リンク

- [Google OAuth 2.0 ドキュメント](https://developers.google.com/identity/protocols/oauth2)
- [Google Sheets API スコープ](https://developers.google.com/sheets/api/guides/authorizing)
- [google-auth-library](https://github.com/googleapis/google-auth-library-nodejs)

## 🆚 認証方式の比較表

| 項目                 | API Key     | Service Account (JWT) | OAuth2             |
| -------------------- | ----------- | --------------------- | ------------------ |
| セットアップの複雑さ | 簡単        | 中程度                | 複雑               |
| セキュリティレベル   | 低          | 高                    | 最高               |
| スプレッドシート共有 | 必要        | 必要                  | 不要               |
| 使用ケース           | 公開データ  | サーバー間通信        | ユーザー代理       |
| トークン管理         | 不要        | 不要                  | 必要               |
| 推奨環境             | 開発/テスト | 本番環境              | デスクトップアプリ |

## 次のステップ

1. OAuth2 クライアント ID を作成
2. ヘルパースクリプトでトークンを取得
3. 設定ファイルを作成
4. テスト実行

より簡単な方法が必要な場合は、[API Key 認証](./TESTING.md)または[JWT 認証](../examples/sheet-jwt.config.yaml)の使用を検討してください。
