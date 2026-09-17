import { ColumnRef } from "./type.js";

const COLUMN_LETTER_PATTERN = /^[A-Za-z]+$/;
const ALPHABET_LENGTH = 26;
const CHAR_CODE_A = "A".charCodeAt(0);

/**
 * 列の指定を0始まりのインデックスへ解決する。
 *
 * - 数値はそのままインデックスとして扱う (0以上の整数のみ)
 * - 文字列は列レターとして26進変換する (A=0, B=1, Z=25, AA=26)
 *
 * @param ref 列レター("B") または数値インデックス(1)
 * @param label エラーメッセージに含める設定項目名 (例: "columns.key")
 */
export const resolveColumnRef = (ref: ColumnRef, label: string): number => {
  if (typeof ref === "number") {
    if (!Number.isInteger(ref) || ref < 0) {
      throw new Error(`${label} の列インデックスは0以上の整数で指定してください: ${ref}`);
    }
    return ref;
  }

  if (typeof ref === "string" && COLUMN_LETTER_PATTERN.test(ref)) {
    // A=1 として累積し、最後に0始まりへ変換する
    const oneBased = ref
      .toUpperCase()
      .split("")
      .reduce((total, char) => total * ALPHABET_LENGTH + (char.charCodeAt(0) - CHAR_CODE_A + 1), 0);
    return oneBased - 1;
  }

  throw new Error(
    `${label} は列レター("B") または0始まりの数値インデックス(1) で指定してください: ${JSON.stringify(ref)}`
  );
};
