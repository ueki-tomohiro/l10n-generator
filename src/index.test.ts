import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { cli } from "./index";
import fs from "fs";
import path from "path";
import {
  createTempDir,
  cleanupTempDir,
  createTestCSV,
  createTestConfig,
  basicTestData,
  paramTestData,
  multiLocaleTestData,
} from "./test-helpers/test-utils";

describe("cli - End-to-End", () => {
  let tempDir: string;
  let outputDir: string;

  beforeEach(() => {
    tempDir = createTempDir();
    outputDir = path.join(tempDir, "output");
    fs.mkdirSync(outputDir);
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  it("CSV → Dart ARBの完全フロー", async () => {
    const csvPath = createTestCSV(tempDir, basicTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "dart",
    });

    await cli(configPath);

    // 出力ファイルが存在することを確認
    expect(fs.existsSync(path.join(outputDir, "app_ja.arb"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "app_en.arb"))).toBe(true);

    // 内容を検証
    const jaArb = JSON.parse(fs.readFileSync(path.join(outputDir, "app_ja.arb"), "utf-8"));
    expect(jaArb["@@locale"]).toBe("ja");
    expect(jaArb.hello).toBe("こんにちは");
    expect(jaArb.goodbye).toBe("さようなら");
    expect(jaArb["@hello"]).toEqual({
      description: "Greeting",
      placeholders: undefined,
    });

    const enArb = JSON.parse(fs.readFileSync(path.join(outputDir, "app_en.arb"), "utf-8"));
    expect(enArb["@@locale"]).toBe("en");
    expect(enArb.hello).toBe("Hello");
    expect(enArb.goodbye).toBe("Goodbye");
  });

  it("CSV → TypeScriptの完全フロー", async () => {
    const csvPath = createTestCSV(tempDir, basicTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "typescript",
    });

    await cli(configPath);

    // 出力ファイルが存在することを確認
    expect(fs.existsSync(path.join(outputDir, "translation.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "translateFunction.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "ja.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "en.ts"))).toBe(true);

    // 内容を検証
    const translationContent = fs.readFileSync(path.join(outputDir, "translation.ts"), "utf-8");
    expect(translationContent).toContain("export interface Translation");
    expect(translationContent).toContain("hello: string;");
    expect(translationContent).toContain("goodbye: string;");

    const jaContent = fs.readFileSync(path.join(outputDir, "ja.ts"), "utf-8");
    expect(jaContent).toContain("export const translation: Translation =");
    expect(jaContent).toContain('"hello": "こんにちは"');
    expect(jaContent).toContain('"goodbye": "さようなら"');

    const enContent = fs.readFileSync(path.join(outputDir, "en.ts"), "utf-8");
    expect(enContent).toContain('"hello": "Hello"');
    expect(enContent).toContain('"goodbye": "Goodbye"');
  });

  it("CSV → Bothの完全フロー", async () => {
    const csvPath = createTestCSV(tempDir, basicTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "both",
    });

    await cli(configPath);

    // Dart ARBファイル
    expect(fs.existsSync(path.join(outputDir, "app_ja.arb"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "app_en.arb"))).toBe(true);

    // TypeScriptファイル
    expect(fs.existsSync(path.join(outputDir, "translation.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "translateFunction.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "ja.ts"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "en.ts"))).toBe(true);
  });

  it("パラメータ付きメッセージの完全フロー", async () => {
    const csvPath = createTestCSV(tempDir, paramTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "both",
    });

    await cli(configPath);

    // Dart ARB - placeholders確認
    const jaArb = JSON.parse(fs.readFileSync(path.join(outputDir, "app_ja.arb"), "utf-8"));
    expect(jaArb.welcome).toBe("ようこそ{name}さん");
    expect(jaArb["@welcome"].placeholders).toHaveProperty("name");
    expect(jaArb["@welcome"].placeholders.name).toEqual({
      type: "String",
      example: "name",
    });

    // TypeScript - translateFunction確認
    const funcContent = fs.readFileSync(path.join(outputDir, "translateFunction.ts"), "utf-8");
    expect(funcContent).toContain("export const welcome");
    expect(funcContent).toContain("params: { name: string; }");
    expect(funcContent).toContain('.replaceAll("{name}", params.name)');

    expect(funcContent).toContain("export const errorCount");
    expect(funcContent).toContain("params: { count: string; }");
  });

  it("複数ロケール（3言語）を処理", async () => {
    const csvPath = createTestCSV(tempDir, multiLocaleTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "dart",
    });

    await cli(configPath);

    // 3つのARBファイルが生成される
    expect(fs.existsSync(path.join(outputDir, "app_ja.arb"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "app_en.arb"))).toBe(true);
    expect(fs.existsSync(path.join(outputDir, "app_es.arb"))).toBe(true);

    const esArb = JSON.parse(fs.readFileSync(path.join(outputDir, "app_es.arb"), "utf-8"));
    expect(esArb["@@locale"]).toBe("es");
    expect(esArb.hello).toBe("Hola");
    expect(esArb.goodbye).toBe("Adiós");
  });

  it("設定ファイルが存在しない場合にエラー終了", async () => {
    const exitSpy = vi.spyOn(process, "exit").mockImplementation((code) => {
      throw new Error(`process.exit: ${code}`);
    });

    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    await expect(cli("/nonexistent/config.yaml")).rejects.toThrow("process.exit: 1");

    expect(consoleSpy).toHaveBeenCalled();

    exitSpy.mockRestore();
    consoleSpy.mockRestore();
  });

  it("データが不足している場合にエラー", async () => {
    const csvPath = createTestCSV(tempDir, [["key", "description", "ja", "en"]]); // ヘッダーのみ
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "dart",
    });

    const exitSpy = vi.spyOn(process, "exit").mockImplementation((code) => {
      throw new Error(`process.exit: ${code}`);
    });

    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    await expect(cli(configPath)).rejects.toThrow("process.exit: 1");

    exitSpy.mockRestore();
    consoleSpy.mockRestore();
  });

  it("未知の出力形式でエラー", async () => {
    const csvPath = createTestCSV(tempDir, basicTestData);
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: csvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "invalid" as any,
    });

    const exitSpy = vi.spyOn(process, "exit").mockImplementation((code) => {
      throw new Error(`process.exit: ${code}`);
    });

    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    await expect(cli(configPath)).rejects.toThrow("process.exit: 1");

    expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining("エラーが発生しました"));

    exitSpy.mockRestore();
    consoleSpy.mockRestore();
  });

  it("既存のsample-data.csvを使用した実際のフロー", async () => {
    const sampleCsvPath = path.join(process.cwd(), "examples", "sample-data.csv");

    // sample-data.csvが存在しない場合はスキップ
    if (!fs.existsSync(sampleCsvPath)) {
      return;
    }

    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: sampleCsvPath,
      credentialType: "none",
      localizePath: outputDir,
      outputType: "both",
    });

    await cli(configPath);

    // ファイルが生成されることを確認
    const files = fs.readdirSync(outputDir);
    expect(files.length).toBeGreaterThan(0);

    // ARBファイルが存在することを確認
    const arbFiles = files.filter((f) => f.endsWith(".arb"));
    expect(arbFiles.length).toBeGreaterThan(0);
  });
});
