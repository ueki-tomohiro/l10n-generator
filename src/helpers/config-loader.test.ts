import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { loadConfig } from "./config-loader.js";
import { createTempDir, cleanupTempDir, createTestConfig } from "../test-helpers/test-utils.js";
import fs from "fs";
import path from "path";

describe("loadConfig", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = createTempDir();
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  it("有効なYAML設定を読み込む", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      outputType: "dart",
    });

    const config = loadConfig(configPath);

    expect(config.fileType).toBe("csv");
    expect(config.path).toBe("/test/data.csv");
    expect(config.credentialType).toBe("none");
    expect(config.localizePath).toBe("./output/"); // スラッシュが追加される
    expect(config.outputType).toBe("dart");
  });

  it("localizePathへのスラッシュ追加", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
    });

    const config = loadConfig(configPath);

    expect(config.localizePath).toBe("./output/");
  });

  it("localizePathに既にスラッシュがある場合は追加しない", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output/",
    });

    const config = loadConfig(configPath);

    expect(config.localizePath).toBe("./output/");
  });

  it("outputTypeのデフォルト値を設定", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
    });

    const config = loadConfig(configPath);

    expect(config.outputType).toBe("dart");
  });

  it("outputTypeが指定されている場合はそのまま使用", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      outputType: "typescript",
    });

    const config = loadConfig(configPath);

    expect(config.outputType).toBe("typescript");
  });

  it("相対パスを絶対パスに解決", () => {
    const configPath = createTestConfig(
      tempDir,
      {
        fileType: "csv",
        path: "/test/data.csv",
        credentialType: "none",
        localizePath: "./output",
      },
      "relative-config.yaml"
    );

    const relativePath = path.relative(process.cwd(), configPath);
    const config = loadConfig(relativePath);

    expect(config.fileType).toBe("csv");
  });

  it("存在しない設定ファイルでエラー", () => {
    const nonExistentPath = path.join(tempDir, "nonexistent.yaml");

    expect(() => loadConfig(nonExistentPath)).toThrow("設定ファイルが見つかりません");
  });

  it("不正なYAML形式でエラー", () => {
    const invalidPath = path.join(tempDir, "invalid.yaml");
    fs.writeFileSync(invalidPath, "{ invalid yaml content [", "utf-8");

    expect(() => loadConfig(invalidPath)).toThrow("設定ファイルの読み込みに失敗しました");
  });

  it("fileType欠損でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
    } as any);

    expect(() => loadConfig(configPath)).toThrow("設定ファイルに必須フィールドが不足しています");
  });

  it("path欠損でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      credentialType: "none",
      localizePath: "./output",
    } as any);

    expect(() => loadConfig(configPath)).toThrow("設定ファイルに必須フィールドが不足しています");
  });

  it("localizePath欠損でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
    } as any);

    expect(() => loadConfig(configPath)).toThrow("設定ファイルに必須フィールドが不足しています");
  });

  it("Google Sheets設定を読み込む", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "sheet",
      path: "https://docs.google.com/spreadsheets/d/abc123/edit",
      credentialType: "apiKey",
      apiKey: "test-api-key",
      localizePath: "./output",
    });

    const config = loadConfig(configPath);

    expect(config.fileType).toBe("sheet");
    expect(config.credentialType).toBe("apiKey");
    expect(config.apiKey).toBe("test-api-key");
  });

  it("xlsx設定を読み込む", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "xlsx",
      path: "./sheet.xlsx",
      credentialType: "none",
      localizePath: "./output",
      sheetName: "sheet",
      skipRows: 2,
      columns: {
        key: "B",
        description: "E",
        locales: { ja: "C", en: "D" },
      },
    });

    const config = loadConfig(configPath);

    expect(config.fileType).toBe("xlsx");
    expect(config.sheetName).toBe("sheet");
    expect(config.skipRows).toBe(2);
    expect(config.columns).toEqual({
      key: "B",
      description: "E",
      locales: { ja: "C", en: "D" },
    });
  });

  it("skipRowsが負の値でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      skipRows: -1,
    });

    expect(() => loadConfig(configPath)).toThrow("skipRows は0以上の整数");
  });

  it("columns.key欠損でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      columns: { locales: { ja: "C" } } as any,
    });

    expect(() => loadConfig(configPath)).toThrow("columns.key が必要です");
  });

  it("columns.localesが空でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      columns: { key: "B", locales: {} },
    });

    expect(() => loadConfig(configPath)).toThrow("columns.locales を1件以上");
  });

  it("ロケール名に使用できない文字が含まれるとエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      columns: { key: "B", locales: { "../evil": "C" } },
    });

    expect(() => loadConfig(configPath)).toThrow("使用できない文字");
  });

  it("columns.localesがオブジェクトでないとエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      columns: { key: "B", locales: "en" } as any,
    });

    expect(() => loadConfig(configPath)).toThrow("columns.locales を1件以上");
  });

  it("数字のみのロケール名でエラー", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      columns: { key: "B", locales: { ja: "C", "10": "D" } },
    });

    expect(() => loadConfig(configPath)).toThrow("数字のみ");
  });

  it.each(["key", "description", "__proto__"])("予約語のロケール名 %s でエラー", (locale) => {
    const configPath = path.join(tempDir, "reserved.yaml");
    fs.writeFileSync(
      configPath,
      `fileType: csv\npath: /test/data.csv\ncredentialType: none\nlocalizePath: ./output\ncolumns:\n  key: B\n  locales:\n    ${locale}: C\n`
    );

    expect(() => loadConfig(configPath)).toThrow("予約語");
  });

  it("columns未指定でも従来どおり読み込める", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
    });

    const config = loadConfig(configPath);

    expect(config.columns).toBeUndefined();
    expect(config.skipRows).toBeUndefined();
  });

  it("both outputTypeを読み込む", () => {
    const configPath = createTestConfig(tempDir, {
      fileType: "csv",
      path: "/test/data.csv",
      credentialType: "none",
      localizePath: "./output",
      outputType: "both",
    });

    const config = loadConfig(configPath);

    expect(config.outputType).toBe("both");
  });
});
