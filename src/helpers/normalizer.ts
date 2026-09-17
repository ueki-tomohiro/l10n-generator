import { resolveColumnRef } from "./columns.js";
import { Config } from "./type.js";

/**
 * インポートした生の2次元配列を、exporter が期待する正規形へ変換する。
 *
 * 正規形: 1行目がヘッダー ["key", "description", ...locales]、2行目以降がデータ。
 *
 * config.columns が未指定の場合は完全な空行だけを除いて入力をそのまま返すため、
 * 既存の [key, description, locale...] 形式の入力は従来どおり動作する。
 */
export const normalizeValues = (config: Config, rawValues: string[][]): string[][] => {
  const { columns } = config;
  // xlsx は skipRows を物理行で扱うため空行を残して読み込むので、ここで取り除く
  if (!columns) return rawValues.filter((row) => row.some((cell) => cell !== ""));

  const keyIndex = resolveColumnRef(columns.key, "columns.key");
  const descriptionIndex =
    columns.description === undefined ? undefined : resolveColumnRef(columns.description, "columns.description");
  const localeEntries = Object.entries(columns.locales).map(([locale, ref]) => ({
    locale,
    index: resolveColumnRef(ref, `columns.locales.${locale}`),
  }));

  const header = ["key", "description", ...localeEntries.map(({ locale }) => locale)];

  const dataRows = rawValues
    .slice(config.skipRows ?? 0)
    .map((row) => [
      row[keyIndex] ?? "",
      descriptionIndex === undefined ? "" : (row[descriptionIndex] ?? ""),
      ...localeEntries.map(({ index }) => row[index] ?? ""),
    ])
    // シート末尾の空行や区切り行を除く
    .filter((row) => row[0] !== "");

  return [header, ...dataRows];
};
