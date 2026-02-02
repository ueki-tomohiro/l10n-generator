# 🚀 クイックスタート - Google Sheets テスト

このガイドでは、5 分で Google Sheets を使った l10n-generator のテスト環境を構築できます。

## ステップ 1: テスト用スプレッドシートを作成 (1 分)

1. [Google Sheets](https://sheets.google.com)で新しいスプレッドシートを作成
2. 以下のサンプルデータをコピー&ペースト:

```
key	description	ja	en
hello	Greeting	こんにちは	Hello
goodbye	Farewell	さようなら	Goodbye
welcome	Welcome message	ようこそ、{name}さん	Welcome, {name}
itemCount	Item count message	{count}個のアイテム	{count} items
```

3. スプレッドシートの URL から **Sheet ID** をコピー
   - URL 例: `https://docs.google.com/spreadsheets/d/`**`1A2B3C4D5E6F7G8H9I0J`**`/edit`
   - Sheet ID は太字の部分です

## ステップ 2: Google Cloud Console で API キーを取得 (2 分)

1. [Google Cloud Console](https://console.cloud.google.com/)を開く
2. 新しいプロジェクトを作成(または既存のものを選択)
3. 左メニューから **API とサービス** → **ライブラリ**
4. 「Google Sheets API」を検索 → **有効にする**
5. **認証情報** → **認証情報を作成** → **API キー**
6. 生成された API キーをコピー

## ステップ 3: スプレッドシートを共有 (30 秒)

1. 作成したスプレッドシートに戻る
2. 右上の **共有** ボタンをクリック
3. **リンクを知っている全員** に変更
4. 権限を **閲覧者** に設定
5. **完了** をクリック

## ステップ 4: 設定ファイルを編集 (30 秒)

プロジェクトルートの `test.config.yaml` を開いて編集:

```yaml
fileType: sheet
path: 1A2B3C4D5E6F7G8H9I0J # ← ステップ1でコピーしたSheet ID
credentialType: apiKey
apiKey: AIza... # ← ステップ2でコピーしたAPIキー
localizePath: ./test-output/
outputType: both
```

## ステップ 5: 診断を実行(推奨) (30秒)

まず診断コマンドで設定が正しいか確認します:

```bash
node lib/cli.js diagnose --config test.config.yaml
```

診断が成功したら、次のステップへ進みます。

## ステップ 6: 実行! (1 分)

ターミナルで以下のコマンドを実行:

```bash
pnpm run test:sheets
```

または:

```bash
./test-sheets.sh
```

または直接実行:

```bash
node lib/cli.js --config test.config.yaml
```

## ✅ 成功!

`test-output/` ディレクトリに以下のファイルが生成されているはずです:

### Dart ARB ファイル

- `app_ja.arb` - 日本語の翻訳
- `app_en.arb` - 英語の翻訳

### TypeScript ファイル

- `translation.ts` - TypeScript 型定義
- `translateFunction.ts` - ヘルパー関数
- `ja.ts` - 日本語翻訳オブジェクト
- `en.ts` - 英語翻訳オブジェクト

## 🎉 次のステップ

成功したら、実際のプロジェクトで使用してみましょう:

1. プロジェクトルートに設定ファイルを作成:

```bash
cp test.config.yaml l10n-generator.config.yaml
```

2. `localizePath` を実際の出力先に変更:

```yaml
localizePath: ./src/i18n/ # または ./lib/l10n/ など
```

3. 実行:

```bash
npx l10n-generator
```

## 🐛 うまくいかない場合

### まず診断コマンドを実行

問題が発生した場合、まず診断コマンドで原因を特定しましょう:

```bash
node lib/cli.js diagnose --config test.config.yaml
```

診断コマンドが自動的に問題を検出し、解決方法を提示します。

### エラー: "Failed to fetch spreadsheet metadata"

- [ ] API キーが正しいか確認
- [ ] Google Sheets API が有効化されているか確認
- [ ] スプレッドシートの共有設定が「リンクを知っている全員」になっているか確認

### エラー: "API key is required"

- [ ] `test.config.yaml`の`apiKey`フィールドを確認
- [ ] API キーに余分なスペースがないか確認

### その他のエラー

詳しいトラブルシューティングは[TESTING.md](./TESTING.md)を参照してください。

## 📚 参考資料

- [README.md](../README.md) - 詳細なドキュメント
- [TESTING.md](./TESTING.md) - テスト環境の詳細
- [examples/](../examples/) - 設定ファイルの例
