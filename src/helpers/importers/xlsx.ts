import fs from "fs";
import { createRequire } from "module";

// xlsx の ESM ビルド(xlsx.mjs)は fs を自動で読み込まず、
// バンドラ経由だと set_fs すら named export として解決できないことがある。
// CLI からの利用のみを想定しているため、fs を自前で require する CJS ビルドを直接読み込む。
const xlsx = createRequire(import.meta.url)("xlsx") as typeof import("xlsx");

type ImportXlsx = (filePath: string, sheetName?: string) => Promise<string[][]>;

/**
 * ローカルの xlsx ファイルを生の2次元配列として読み込む。
 *
 * `header: 1` を指定することで1行目をヘッダーとして推論させず、
 * すべての行をそのまま配列で受け取る (列レイアウトは columns 設定で解決する)。
 */
export const importXlsx: ImportXlsx = async (filePath, sheetName) => {
  if (!fs.existsSync(filePath)) {
    throw new Error(`xlsxファイルが見つかりません: ${filePath}`);
  }

  const workbook = xlsx.readFile(filePath);
  const targetSheet = sheetName ?? workbook.SheetNames[0];

  if (!targetSheet) {
    throw new Error(`xlsxファイルにシートがありません: ${filePath}`);
  }

  const worksheet = workbook.Sheets[targetSheet];
  if (!worksheet) {
    throw new Error(`シートが見つかりません: ${targetSheet} (利用可能なシート: ${workbook.SheetNames.join(", ")})`);
  }

  return xlsx.utils.sheet_to_json<string[]>(worksheet, {
    header: 1,
    // skipRows を物理的な行位置で適用するため、空行も詰めずに残す
    blankrows: true,
    defval: "",
    raw: false,
  });
};
