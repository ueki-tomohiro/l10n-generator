#!/bin/bash

# Google Sheets テストスクリプト
# このスクリプトは test.config.yaml を使用してGoogle Sheetsからデータを取得します

echo "🚀 l10n-generator Google Sheets テスト"
echo ""

# test.config.yamlが存在するかチェック
if [ ! -f "test.config.yaml" ]; then
    echo "❌ test.config.yaml が見つかりません"
    echo "📝 test.config.yaml を作成してください:"
    echo ""
    echo "fileType: sheet"
    echo "path: YOUR_SHEET_ID"
    echo "credentialType: apiKey"
    echo "apiKey: YOUR_API_KEY"
    echo "localizePath: ./test-output/"
    echo "outputType: both"
    echo ""
    exit 1
fi

# test-outputディレクトリが存在するかチェック
if [ ! -d "test-output" ]; then
    echo "📁 test-output ディレクトリを作成しています..."
    mkdir -p test-output
fi

# 古い出力ファイルをクリーンアップ
if [ -d "test-output" ] && [ "$(ls -A test-output)" ]; then
    echo "🧹 古い出力ファイルをクリーンアップしています..."
    rm -rf test-output/*
fi

# ビルド
echo "🔨 プロジェクトをビルドしています..."
pnpm build

if [ $? -ne 0 ]; then
    echo "❌ ビルドに失敗しました"
    exit 1
fi

echo ""
echo "✅ ビルド完了"
echo ""

# テスト実行
echo "📡 Google Sheetsからデータを取得しています..."
echo ""

node lib/cli.js --config test.config.yaml

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 成功! 出力ファイルを確認してください:"
    echo ""
    echo "📂 test-output/"
    if [ -d "test-output" ]; then
        ls -lh test-output/
    fi
else
    echo ""
    echo "❌ エラーが発生しました"
    echo ""
    echo "📚 トラブルシューティング:"
    echo "1. test.config.yaml の設定を確認してください"
    echo "2. Google Sheets APIが有効化されているか確認してください"
    echo "3. スプレッドシートの共有設定を確認してください"
    echo "4. 詳しくは TESTING.md を参照してください"
    exit 1
fi
