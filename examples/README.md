# 設定ファイルのサンプル

このディレクトリには、l10n-generatorの設定ファイルのサンプルが含まれています。

## ファイル一覧

### 設定ファイル

- **csv-dart.config.yaml** - CSV + Dart ARB出力
- **csv-typescript.config.yaml** - CSV + TypeScript出力
- **csv-both.config.yaml** - CSV + Dart & TypeScript両方出力
- **sheet-apikey.config.yaml** - Google Sheets + API Key認証
- **sheet-jwt.config.yaml** - Google Sheets + JWT認証（Service Account）

### サンプルデータ

- **sample-data.csv** - テスト用のCSVデータ

## 使用方法

1. 使用したい設定ファイルをプロジェクトルートにコピー:

   ```bash
   cp examples/csv-typescript.config.yaml l10n-generator.config.yaml
   ```

2. 設定ファイルを編集して、プロジェクトに合わせて設定を変更:
   - `path`: データファイルのパス
   - `localizePath`: 出力先ディレクトリ
   - `outputType`: 出力形式（dart / typescript / both）

3. l10n-generatorを実行:

   ```bash
   l10n-generator
   ```

## CSVファイルを使う場合

1. `sample-data.csv`をコピーまたは参考にしてデータファイルを作成
2. CSV形式の設定ファイルを使用

## Google Sheetsを使う場合

### API Key認証

1. [Google Cloud Console](https://console.cloud.google.com/)でAPIキーを作成
2. `sheet-apikey.config.yaml`の`apiKey`にAPIキーを設定
3. スプレッドシートを「リンクを知っている全員」に共有

### JWT認証（推奨）

1. サービスアカウントを作成してJSONキーをダウンロード
2. `sheet-jwt.config.yaml`の`jwt`フィールドにJSON内容をコピー
3. スプレッドシートをサービスアカウントのメールアドレスと共有

## 環境変数の使用

機密情報は環境変数から読み込むことを推奨します:

```yaml
apiKey: ${GOOGLE_API_KEY}
```

または:

```yaml
jwt:
  private_key: ${GOOGLE_SERVICE_ACCOUNT_KEY}
```

## トラブルシューティング

問題が発生した場合は、[メインのREADME](../README.md#トラブルシューティング)を参照してください。
