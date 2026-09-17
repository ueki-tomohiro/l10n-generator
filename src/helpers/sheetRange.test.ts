import { describe, it, expect } from "vitest";
import { toSheetRange } from "./sheetRange.js";

describe("toSheetRange", () => {
  it("シート名をシングルクォートで囲む", () => {
    expect(toSheetRange("Sheet1")).toBe("'Sheet1'");
    expect(toSheetRange("My Sheet")).toBe("'My Sheet'");
    expect(toSheetRange("Release!2026")).toBe("'Release!2026'");
  });

  it("アポストロフィを2つ重ねてエスケープする", () => {
    expect(toSheetRange("Bob's Data")).toBe("'Bob''s Data'");
  });
});
