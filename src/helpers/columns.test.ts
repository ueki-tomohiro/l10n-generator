import { describe, it, expect } from "vitest";
import { resolveColumnRef } from "./columns.js";

describe("resolveColumnRef", () => {
  it("数値インデックスをそのまま返す", () => {
    expect(resolveColumnRef(0, "columns.key")).toBe(0);
    expect(resolveColumnRef(4, "columns.key")).toBe(4);
  });

  it("列レターを0始まりのインデックスへ変換する", () => {
    expect(resolveColumnRef("A", "columns.key")).toBe(0);
    expect(resolveColumnRef("B", "columns.key")).toBe(1);
    expect(resolveColumnRef("E", "columns.key")).toBe(4);
    expect(resolveColumnRef("Z", "columns.key")).toBe(25);
  });

  it("2文字以上の列レターを変換する", () => {
    expect(resolveColumnRef("AA", "columns.key")).toBe(26);
    expect(resolveColumnRef("AB", "columns.key")).toBe(27);
    expect(resolveColumnRef("BA", "columns.key")).toBe(52);
  });

  it("小文字の列レターを受け付ける", () => {
    expect(resolveColumnRef("b", "columns.key")).toBe(1);
    expect(resolveColumnRef("aa", "columns.key")).toBe(26);
  });

  it("負の数値でエラー", () => {
    expect(() => resolveColumnRef(-1, "columns.key")).toThrow("0以上の整数");
  });

  it("小数でエラー", () => {
    expect(() => resolveColumnRef(1.5, "columns.key")).toThrow("0以上の整数");
  });

  it("列レター以外の文字列でエラー", () => {
    expect(() => resolveColumnRef("A1", "columns.key")).toThrow("columns.key");
    expect(() => resolveColumnRef("", "columns.key")).toThrow("columns.key");
    expect(() => resolveColumnRef("あ", "columns.key")).toThrow("columns.key");
  });

  it("エラーメッセージに設定項目名を含む", () => {
    expect(() => resolveColumnRef("A1", "columns.locales.ja")).toThrow("columns.locales.ja");
  });
});
