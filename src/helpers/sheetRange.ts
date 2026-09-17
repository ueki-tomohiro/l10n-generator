/**
 * シート名を Google Sheets API の A1 記法の range に変換する。
 *
 * 空白や記号を含むシート名、同名の named range との衝突に備えて常にシングルクォートで囲み、
 * シート名中のアポストロフィは2つ重ねてエスケープする。
 */
export const toSheetRange = (sheetName: string): string => `'${sheetName.replace(/'/g, "''")}'`;
