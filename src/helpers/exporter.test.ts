import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { createL10nFile, createTypeScriptL10nFiles } from "./exporter.js";
import fs from "fs";
import path from "path";
import {
  createTempDir,
  cleanupTempDir,
  basicTestData,
  paramTestData,
  multiLocaleTestData,
} from "../test-helpers/test-utils.js";

describe("createL10nFile", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = createTempDir();
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  it("基本的なARBファイルを生成する", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));
    await createL10nFile(`${tempDir}/`, values, "ja");

    const arbPath = path.join(tempDir, "app_ja.arb");
    expect(fs.existsSync(arbPath)).toBe(true);

    const arb = JSON.parse(fs.readFileSync(arbPath, "utf-8"));

    // @@localeを検証
    expect(arb["@@locale"]).toBe("ja");

    // キーと値を検証
    expect(arb.hello).toBe("こんにちは");
    expect(arb.goodbye).toBe("さようなら");

    // メタデータを検証
    expect(arb["@hello"]).toEqual({
      description: "Greeting",
      placeholders: undefined,
    });
    expect(arb["@goodbye"]).toEqual({
      description: "Farewell",
      placeholders: undefined,
    });
  });

  it("英語のARBファイルを生成する", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));
    await createL10nFile(`${tempDir}/`, values, "en");

    const arbPath = path.join(tempDir, "app_en.arb");
    expect(fs.existsSync(arbPath)).toBe(true);

    const arb = JSON.parse(fs.readFileSync(arbPath, "utf-8"));

    expect(arb["@@locale"]).toBe("en");
    expect(arb.hello).toBe("Hello");
    expect(arb.goodbye).toBe("Goodbye");
  });

  it("パラメータ付きメッセージのplaceholdersを生成する", async () => {
    const values = JSON.parse(JSON.stringify(paramTestData));
    await createL10nFile(`${tempDir}/`, values, "ja");

    const arb = JSON.parse(fs.readFileSync(path.join(tempDir, "app_ja.arb"), "utf-8"));

    // 単一パラメータ
    expect(arb.welcome).toBe("ようこそ{name}さん");
    expect(arb["@welcome"]).toEqual({
      description: "Welcome message",
      placeholders: {
        name: {
          type: "String",
          example: "name",
        },
      },
    });

    // 複数パラメータ
    expect(arb.error_count).toBe("{count}件のエラー");
    expect(arb["@error_count"].placeholders).toHaveProperty("count");
    expect(arb["@error_count"].placeholders.count).toEqual({
      type: "String",
      example: "count",
    });
  });

  it("配列の破壊的操作（shift）を実行する", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));
    const originalLength = values.length;

    await createL10nFile(`${tempDir}/`, values, "ja");

    // shiftが呼ばれるので配列の長さが減る
    expect(values.length).toBe(originalLength - 1);
    // ヘッダー行が削除され、最初の要素がデータ行になる
    expect(values[0]).toEqual(["hello", "Greeting", "こんにちは", "Hello"]);
  });

  it("空のヘッダー配列を処理する", async () => {
    const values: string[][] = [];

    // エラーなく終了することを確認
    await expect(createL10nFile(`${tempDir}/`, values, "ja")).resolves.toBeUndefined();

    // ファイルが生成されないことを確認
    expect(fs.existsSync(path.join(tempDir, "app_ja.arb"))).toBe(false);
  });

  it("ヘッダーのみの配列を処理する", async () => {
    const values: string[][] = [["key", "description", "ja", "en"]];

    // ヘッダーのみの場合、reduceがエラーになるため、
    // 実際にはこのケースはサポートされていない動作
    // テストはエラーが発生することを確認
    await expect(async () => {
      await createL10nFile(`${tempDir}/`, values, "ja");
    }).rejects.toThrow();
  });

  it("複数ロケールの存在を確認（es）", async () => {
    const values = JSON.parse(JSON.stringify(multiLocaleTestData));
    await createL10nFile(`${tempDir}/`, values, "es");

    const arbPath = path.join(tempDir, "app_es.arb");
    expect(fs.existsSync(arbPath)).toBe(true);

    const arb = JSON.parse(fs.readFileSync(arbPath, "utf-8"));

    expect(arb["@@locale"]).toBe("es");
    expect(arb.hello).toBe("Hola");
    expect(arb.goodbye).toBe("Adiós");
  });
});

describe("createTypeScriptL10nFiles", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = createTempDir();
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  it("translation.tsの型定義を生成する", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const tsPath = path.join(tempDir, "translation.ts");
    expect(fs.existsSync(tsPath)).toBe(true);

    const content = fs.readFileSync(tsPath, "utf-8");

    expect(content).toContain("export interface Translation");
    expect(content).toContain("hello: string;");
    expect(content).toContain("goodbye: string;");
    expect(content).toContain("* こんにちは: Greeting");
    expect(content).toContain("* さようなら: Farewell");
  });

  it("パラメータ付きメッセージのヘルパー関数を生成する", async () => {
    const values = JSON.parse(JSON.stringify(paramTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const funcPath = path.join(tempDir, "translateFunction.ts");
    const content = fs.readFileSync(funcPath, "utf-8");

    // インポート文
    expect(content).toContain('import { Translation } from "./translation"');

    // welcome関数
    expect(content).toContain("export const welcome");
    expect(content).toContain("params: { name: string; }");
    expect(content).toContain('.replaceAll("{name}", params.name)');

    // error_count関数（camelCase変換）
    expect(content).toContain("export const errorCount");
    expect(content).toContain("params: { count: string; }");
    expect(content).toContain('.replaceAll("{count}", params.count)');
  });

  it("パラメータなしのキーにはヘルパー関数を生成しない", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const funcPath = path.join(tempDir, "translateFunction.ts");
    const content = fs.readFileSync(funcPath, "utf-8");

    // helloとgoodbyeの関数は生成されない
    expect(content).not.toContain("export const hello");
    expect(content).not.toContain("export const goodbye");
  });

  it("各ロケールのファイルを生成する", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const jaPath = path.join(tempDir, "ja.ts");
    const enPath = path.join(tempDir, "en.ts");

    expect(fs.existsSync(jaPath)).toBe(true);
    expect(fs.existsSync(enPath)).toBe(true);

    const jaContent = fs.readFileSync(jaPath, "utf-8");
    expect(jaContent).toContain('import { Translation } from "./translation"');
    expect(jaContent).toContain("export const translation: Translation =");
    expect(jaContent).toContain('"hello": "こんにちは"');
    expect(jaContent).toContain('"goodbye": "さようなら"');

    const enContent = fs.readFileSync(enPath, "utf-8");
    expect(enContent).toContain('"hello": "Hello"');
    expect(enContent).toContain('"goodbye": "Goodbye"');
  });

  it("複数ロケールのファイルを生成する", async () => {
    const values = JSON.parse(JSON.stringify(multiLocaleTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const jaPath = path.join(tempDir, "ja.ts");
    const enPath = path.join(tempDir, "en.ts");
    const esPath = path.join(tempDir, "es.ts");

    expect(fs.existsSync(jaPath)).toBe(true);
    expect(fs.existsSync(enPath)).toBe(true);
    expect(fs.existsSync(esPath)).toBe(true);

    const esContent = fs.readFileSync(esPath, "utf-8");
    expect(esContent).toContain('"hello": "Hola"');
    expect(esContent).toContain('"goodbye": "Adiós"');
  });

  it("JSONフォーマットが正しい", async () => {
    const values = JSON.parse(JSON.stringify(basicTestData));

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const jaPath = path.join(tempDir, "ja.ts");
    const jaContent = fs.readFileSync(jaPath, "utf-8");

    // JSONがインデント付きでフォーマットされていることを確認
    expect(jaContent).toContain('"hello": "こんにちは"');
    expect(jaContent).toContain('"goodbye": "さようなら"');
    // インデントがあることを確認
    expect(jaContent).toMatch(/\{\s+"/);
  });

  it("空のtranslationを処理する", async () => {
    const values: string[][] = [["key", "description", "ja", "en"]];

    await createTypeScriptL10nFiles(`${tempDir}/`, values);

    const tsPath = path.join(tempDir, "translation.ts");
    expect(fs.existsSync(tsPath)).toBe(true);

    const content = fs.readFileSync(tsPath, "utf-8");
    expect(content).toContain("export interface Translation {");
    expect(content).toContain("}");
  });
});
