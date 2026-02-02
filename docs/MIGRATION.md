# TypeScript CLI への移行完了

このドキュメントでは、CLI を JavaScript から TypeScript に移行した変更点をまとめています。

## 変更内容

### ✅ 実装された機能

1. **CLI の TypeScript 化**

   - `cli.js` → `src/cli.ts`
   - 型安全性の向上
   - ビルドプロセスへの統合

2. **診断機能の分離**

   - `src/helpers/diagnose.ts` として独立した関数に
   - 再利用可能な設計
   - 適切な型定義

3. **ビルドプロセスの統合**
   - TypeScript コンパイラによる自動ビルド
   - `lib/cli.js` に出力
   - shebang の自動保持

### 📁 ファイル構造

```
src/
├── cli.ts                    # CLIエントリーポイント (TypeScript)
├── index.ts                  # メインロジック
└── helpers/
    ├── diagnose.ts           # 診断機能 (新規)
    ├── importer.ts
    ├── exporter.ts
    └── ...

lib/                          # ビルド出力
├── cli.js                    # コンパイルされたCLI
├── cli.d.ts                  # 型定義
└── helpers/
    ├── diagnose.js           # コンパイルされた診断機能
    └── ...
```

### 🔧 package.json の変更

```json
{
  "bin": {
    "l10n-generator": "./lib/cli.js" // 変更: cli.js → lib/cli.js
  }
}
```

### 📝 使用方法の変更

#### 開発中

```bash
# ビルド
pnpm build

# 実行
node lib/cli.js --config test.config.yaml

# 診断
node lib/cli.js diagnose --config test.config.yaml
```

#### インストール後（変更なし）

```bash
# グローバルインストール後
l10n-generator --config test.config.yaml
l10n-generator diagnose

# npx経由
npx l10n-generator --config test.config.yaml
npx l10n-generator diagnose
```

## 移行の利点

### 1. 型安全性の向上

**Before (JavaScript)**:

```javascript
async function diagnoseCommand(argv) {
  const configFile = argv.config; // 型チェックなし
  // ...
}
```

**After (TypeScript)**:

```typescript
interface DiagnoseOptions {
  configFile: string;
}

async function diagnose(options: DiagnoseOptions): Promise<void> {
  const { configFile } = options; // 型チェックあり
  // ...
}
```

### 2. 保守性の向上

- IDE の補完機能が強化
- リファクタリングが安全に
- エラーをコンパイル時に検出

### 3. 一貫性の向上

- プロジェクト全体が TypeScript で統一
- 型定義の共有が容易
- ビルドプロセスの一元化

## 開発ワークフロー

### コードの変更

1. `src/cli.ts` または `src/helpers/diagnose.ts` を編集
2. `pnpm build` でビルド
3. `node lib/cli.js` でテスト

### 自動ビルド（開発時）

```bash
# TypeScriptのwatch mode
pnpm exec tsc --watch

# 別のターミナルでテスト
node lib/cli.js --help
```

## テスト

### ユニットテスト

診断機能は独立した関数なので、テストが容易です:

```typescript
import { diagnose } from "./helpers/diagnose";

describe("diagnose", () => {
  it("should validate config file", async () => {
    await expect(diagnose({ configFile: "nonexistent.yaml" })).rejects.toThrow();
  });
});
```

### 統合テスト

```bash
# ビルドしてテスト
pnpm build && node lib/cli.js diagnose --config test.config.yaml
```

## トラブルシューティング

### ビルドエラー

**問題**: TypeScript コンパイルエラー

**解決**:

```bash
# 型チェック
pnpm exec tsc --noEmit

# エラー詳細を確認
pnpm build
```

### 実行時エラー

**問題**: `Cannot find module` エラー

**解決**:

```bash
# 依存関係を再インストール
pnpm install

# ビルド
pnpm build
```

### shebang が消える

**問題**: `lib/cli.js` に shebang がない

**解決**:
TypeScript コンパイラは `src/cli.ts` の先頭にある `#!/usr/bin/env node` を自動的に保持します。もし消えている場合は:

1. `src/cli.ts` の最初の行が `#!/usr/bin/env node` であることを確認
2. `pnpm build` で再ビルド

## 後方互換性

### ドキュメントの更新

すべてのドキュメントで `node cli.js` を `node lib/cli.js` に更新済み:

- ✅ README.md
- ✅ QUICKSTART.md
- ✅ TESTING.md
- ✅ CLI-USAGE.md
- ✅ test-sheets.sh

### 既存のワークフロー

既存の使用方法は変更なし:

```bash
# package.jsonのscripts
pnpm run test:sheets        # 動作OK

# npmスクリプト経由
npm run i18n                # 動作OK

# npx経由（インストール後）
npx l10n-generator          # 動作OK
```

## 参考資料

- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Node.js TypeScript Support](https://nodejs.org/en/learn/getting-started/nodejs-with-typescript)
- [Yargs TypeScript Guide](https://yargs.js.org/docs/#typescript)

## 次のステップ

今後の改善案:

1. **テストの追加**

   - 診断機能のユニットテスト
   - CLI の統合テスト

2. **エラーハンドリングの強化**

   - カスタムエラークラス
   - より詳細なエラーメッセージ

3. **パフォーマンス最適化**
   - 非同期処理の最適化
   - キャッシュ機構の追加
