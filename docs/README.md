# l10n-generator ドキュメント

このディレクトリには、l10n-generator の詳細なドキュメントが含まれています。

## 📚 ドキュメント一覧

### 🚀 はじめに

- **[QUICKSTART.md](./QUICKSTART.md)** - 5 分でできるクイックスタート
  - 最速で Google Sheets からローカライゼーションファイルを生成
  - API Key 認証を使用

### 💻 使い方

- **[CLI-USAGE.md](./CLI-USAGE.md)** - CLI 使用方法の完全ガイド
  - コマンドラインオプションの詳細
  - 診断コマンドの使い方
  - トラブルシューティング
  - 実行例

### 🧪 テスト環境

- **[TESTING.md](./TESTING.md)** - テスト環境のセットアップ
  - Google Cloud Console の設定
  - API Key 認証の設定方法
  - Service Account (JWT)認証の設定方法
  - サンプルデータの作成

### 🔐 OAuth2 認証

- **[OAUTH2-QUICKSTART.md](./OAUTH2-QUICKSTART.md)** - OAuth2 クイックスタート

  - 5 ステップで完了
  - プライベートスプレッドシートに対応
  - トークン取得の自動化

- **[OAUTH2-SETUP.md](./OAUTH2-SETUP.md)** - OAuth2 詳細ガイド
  - 完全なセットアップ手順
  - OAuth 同意画面の設定
  - セキュリティのベストプラクティス
  - トラブルシューティング

## 📖 認証方式の選択

どの認証方式を使うべきか迷ったら:

### API Key (最も簡単)

- ✅ セットアップが簡単
- ✅ すぐに始められる
- ❌ スプレッドシートを公開する必要がある
- **推奨**: 開発・テスト環境、公開データ

👉 [QUICKSTART.md](./QUICKSTART.md)

### OAuth2 (プライベートデータ)

- ✅ プライベートスプレッドシートに対応
- ✅ 高いセキュリティ
- ✅ トークン自動更新
- ❌ セットアップがやや複雑
- **推奨**: 個人のプライベートデータ、デスクトップアプリ

👉 [OAUTH2-QUICKSTART.md](./OAUTH2-QUICKSTART.md)

### Service Account / JWT (本番環境)

- ✅ サーバー間通信に最適
- ✅ 高いセキュリティ
- ❌ JSON キーの管理が必要
- **推奨**: 本番環境、CI/CD、サーバー環境

👉 [TESTING.md](./TESTING.md)

## 🔗 関連リンク

- [メイン README](../README.md) - プロジェクト概要
- [設定例](../examples/) - YAML 設定ファイルの例
- [GitHub リポジトリ](https://github.com/ueki-tomohiro/l10n-generator)

## 📝 ドキュメントの読み方

### 初めての方

1. [QUICKSTART.md](./QUICKSTART.md)で API Key 認証を試す
2. [CLI-USAGE.md](./CLI-USAGE.md)でコマンドを理解する
3. 必要に応じて[OAUTH2-QUICKSTART.md](./OAUTH2-QUICKSTART.md)で OAuth2 に移行

### 本番環境の方

1. [TESTING.md](./TESTING.md)で Service Account 認証を設定
2. [CLI-USAGE.md](./CLI-USAGE.md)で CI/CD 統合を確認
3. セキュリティベストプラクティスに従う

## 💡 ヒント

- **診断コマンド**を活用しましょう:

  ```bash
  node lib/cli.js diagnose --config your-config.yaml
  ```

- **環境変数**で機密情報を管理しましょう:

  ```yaml
  apiKey: ${GOOGLE_API_KEY}
  ```

- **複数の設定ファイル**を使い分けましょう:
  - `dev.config.yaml` - 開発環境
  - `test.config.yaml` - テスト環境
  - `prod.config.yaml` - 本番環境

## 🤝 貢献

ドキュメントの改善案やバグ報告は、[GitHub Issues](https://github.com/ueki-tomohiro/l10n-generator/issues)でお願いします。
