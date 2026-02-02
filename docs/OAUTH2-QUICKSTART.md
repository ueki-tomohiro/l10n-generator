# OAuth2 クイックスタート

OAuth2 認証を使って Google Sheets からローカライゼーションデータを取得する最短の手順です。

## 📋 前提条件

- Google アカウント
- アクセスしたい Google Spreadsheet

## 🚀 5 つのステップ

### ステップ 1: OAuth 2.0 クライアント ID を作成 (3 分)

1. [Google Cloud Console](https://console.cloud.google.com/)にアクセス
2. プロジェクトを作成または選択
3. **API とサービス** → **ライブラリ** → 「Google Sheets API」を有効化
4. **OAuth 同意画面** を設定:
   - ユーザータイプ: 外部
   - アプリ名: `l10n-generator`
   - サポートメール: あなたのメールアドレス
5. **スコープを追加**:
   - `https://www.googleapis.com/auth/spreadsheets.readonly`
6. **テストユーザーを追加**: あなたの Google アカウント
7. **認証情報** → **認証情報を作成** → **OAuth クライアント ID**
   - アプリケーションの種類: **デスクトップアプリ**
   - 名前: `l10n-generator-cli`
8. クライアント ID とシークレットをコピー

### ステップ 2: トークンを取得 (1 分)

```bash
# プロジェクトをビルド
pnpm build

# トークン取得ヘルパーを実行
node lib/helpers/oauth2-helper.js
```

プロンプトに従って:

1. Client ID を入力
2. Client Secret を入力
3. ブラウザが開くので、Google アカウントでログイン
4. アクセスを許可
5. ターミナルに戻るとトークンが表示される

### ステップ 3: 設定ファイルを作成 (30 秒)

`test-oauth2.config.yaml`を作成:

```yaml
fileType: sheet
path: YOUR_SPREADSHEET_ID
credentialType: oauth2
oauth2:
  clientId: YOUR_CLIENT_ID.apps.googleusercontent.com
  clientSecret: YOUR_CLIENT_SECRET
  redirectUri: http://localhost:3000/oauth2callback
  refreshToken: YOUR_REFRESH_TOKEN
localizePath: ./test-output/
outputType: both
```

ステップ 2 で表示されたトークン情報をコピーして貼り付けてください。

### ステップ 4: スプレッドシート ID を取得 (10 秒)

Google Sheets の URL から:

```
https://docs.google.com/spreadsheets/d/1A2B3C4D5E6F7G8H9I0J/edit
                                        ↑ この部分がSheet ID
```

設定ファイルの`path`に設定します。

### ステップ 5: 実行! (10 秒)

```bash
node lib/cli.js --config test-oauth2.config.yaml
```

## ✅ 成功!

`test-output/`ディレクトリにファイルが生成されます:

### Dart ARB

- `app_ja.arb`
- `app_en.arb`

### TypeScript

- `translation.ts`
- `translateFunction.ts`
- `ja.ts`
- `en.ts`

## 🎯 OAuth2 の利点

### API Key と比較

| 項目                 | API Key      | OAuth2 |
| -------------------- | ------------ | ------ |
| スプレッドシート共有 | 必要（公開） | 不要   |
| セキュリティ         | 低           | 高     |
| 複数ユーザー         | ❌           | ✅     |
| プライベートシート   | ❌           | ✅     |

### Service Account (JWT)と比較

| 項目          | JWT    | OAuth2 |
| ------------- | ------ | ------ |
| セットアップ  | 中程度 | 簡単   |
| JSON キー管理 | 必要   | 不要   |
| ユーザー認証  | ❌     | ✅     |
| サーバー環境  | ✅     | △      |

## 🔄 トークンの更新

リフレッシュトークンがあれば、アクセストークンは自動的に更新されます。

ツールが自動的に:

1. アクセストークンの有効期限をチェック
2. 期限切れの場合、リフレッシュトークンで更新
3. 新しいアクセストークンで API にアクセス

## 🐛 トラブルシューティング

### ブラウザが開かない

```bash
# 表示されたURLを手動でブラウザで開く
```

### "invalid_grant"エラー

```bash
# トークンを再取得
node lib/helpers/oauth2-helper.js
```

### リフレッシュトークンが取得できない

これは、以前に同じクライアント ID で認証した可能性があります。

**解決方法**:

1. [Google アカウントのセキュリティ](https://myaccount.google.com/permissions)を開く
2. `l10n-generator`アプリを削除
3. トークン取得ヘルパーを再実行

## 💡 ヒント

### プライベートスプレッドシート

OAuth2 を使えば、スプレッドシートを公開する必要がありません!

### 複数のスプレッドシート

同じトークンで、ログインユーザーがアクセス可能な全てのスプレッドシートにアクセスできます。

### 本番環境

本番環境では、環境変数でトークンを管理することをお勧めします:

```yaml
oauth2:
  clientId: ${OAUTH2_CLIENT_ID}
  clientSecret: ${OAUTH2_CLIENT_SECRET}
  refreshToken: ${OAUTH2_REFRESH_TOKEN}
```

## 📚 詳細情報

より詳しい情報は以下を参照:

- [OAUTH2-SETUP.md](./OAUTH2-SETUP.md) - 詳細なセットアップガイド
- [認証方式の比較](#-oauth2の利点) - どの認証方式を選ぶべきか

## 次のステップ

成功したら:

1. 設定ファイルを本番用にカスタマイズ
2. CI/CD パイプラインに統合
3. 環境変数で機密情報を管理
