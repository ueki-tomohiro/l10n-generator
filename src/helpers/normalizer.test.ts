import { describe, it, expect } from "vitest";
import { normalizeValues } from "./normalizer.js";
import { Config } from "./type.js";

const baseConfig: Config = {
  fileType: "xlsx",
  path: "./sheet.xlsx",
  credentialType: "none",
  localizePath: "./output/",
};

/** senkyaku のシート相当: 2行ヘッダ + [A列(無視), key, ja, en, description] */
const senkyakuLikeSheet = [
  ["カテゴリ", "", "", "", ""],
  ["", "key", "ja", "en", "description"],
  ["画面", "hello", "こんにちは", "Hello", "Greeting"],
  ["画面", "goodbye", "さようなら", "Goodbye", "Farewell"],
];

const senkyakuColumns = {
  key: "B",
  description: "E",
  locales: { ja: "C", en: "D" },
};

describe("normalizeValues", () => {
  it("columns 未指定なら入力をそのまま返す", () => {
    const values = [
      ["key", "description", "ja", "en"],
      ["hello", "Greeting", "こんにちは", "Hello"],
    ];

    expect(normalizeValues(baseConfig, values)).toEqual(values);
  });

  it("columns 指定で正規形へ変換する", () => {
    const config: Config = { ...baseConfig, skipRows: 2, columns: senkyakuColumns };

    expect(normalizeValues(config, senkyakuLikeSheet)).toEqual([
      ["key", "description", "ja", "en"],
      ["hello", "Greeting", "こんにちは", "Hello"],
      ["goodbye", "Farewell", "さようなら", "Goodbye"],
    ]);
  });

  it("数値インデックスでも同じ結果になる", () => {
    const config: Config = {
      ...baseConfig,
      skipRows: 2,
      columns: { key: 1, description: 4, locales: { ja: 2, en: 3 } },
    };

    expect(normalizeValues(config, senkyakuLikeSheet)).toEqual(
      normalizeValues({ ...baseConfig, skipRows: 2, columns: senkyakuColumns }, senkyakuLikeSheet)
    );
  });

  it("skipRows 未指定なら先頭行から読む", () => {
    const config: Config = { ...baseConfig, columns: senkyakuColumns };

    const result = normalizeValues(config, senkyakuLikeSheet);

    // 2行目のヘッダ行がデータとして残る
    expect(result[1]).toEqual(["key", "description", "ja", "en"]);
  });

  it("key が空の行を除外する", () => {
    const withBlank = [...senkyakuLikeSheet, ["", "", "", "", ""], ["画面", "", "空キー", "", ""]];
    const config: Config = { ...baseConfig, skipRows: 2, columns: senkyakuColumns };

    const result = normalizeValues(config, withBlank);

    expect(result).toHaveLength(3); // ヘッダ + データ2件
  });

  it("description 未指定なら空文字を埋める", () => {
    const config: Config = {
      ...baseConfig,
      skipRows: 2,
      columns: { key: "B", locales: { ja: "C" } },
    };

    expect(normalizeValues(config, senkyakuLikeSheet)).toEqual([
      ["key", "description", "ja"],
      ["hello", "", "こんにちは"],
      ["goodbye", "", "さようなら"],
    ]);
  });

  it("欠けている列は空文字になる", () => {
    const shortRows = [
      ["", "key", "ja", "en", "description"],
      ["", "hello", "こんにちは"],
    ];
    const config: Config = { ...baseConfig, skipRows: 1, columns: senkyakuColumns };

    expect(normalizeValues(config, shortRows)).toEqual([
      ["key", "description", "ja", "en"],
      ["hello", "", "こんにちは", ""],
    ]);
  });

  it("ロケールの並び順は columns.locales の定義順に従う", () => {
    const config: Config = {
      ...baseConfig,
      skipRows: 2,
      columns: { key: "B", description: "E", locales: { en: "D", ja: "C" } },
    };

    const result = normalizeValues(config, senkyakuLikeSheet);

    expect(result[0]).toEqual(["key", "description", "en", "ja"]);
    expect(result[1]).toEqual(["hello", "Greeting", "Hello", "こんにちは"]);
  });

  it("不正な列指定でエラー", () => {
    const config: Config = {
      ...baseConfig,
      columns: { key: "A1", locales: { ja: "C" } },
    };

    expect(() => normalizeValues(config, senkyakuLikeSheet)).toThrow("columns.key");
  });
});
