import { google } from "googleapis";
import { JWTOptions } from "google-auth-library";

type ImportGoogleSpreadSheetWithJWT = (url: string, option?: JWTOptions) => Promise<string[][]>;

export const importGoogleSpreadSheetWithJWT: ImportGoogleSpreadSheetWithJWT = async (url, option) => {
  if (!option) {
    throw new Error("JWT credentials are required");
  }

  const auth = new google.auth.JWT(option);
  await auth.authorize();

  const sheets = google.sheets({ version: "v4", auth });

  // URLからスプレッドシートIDとシート名を抽出
  const match = url.match(/\/spreadsheets\/d\/([a-zA-Z0-9-_]+)/);
  if (!match) {
    throw new Error("Invalid Google Sheets URL");
  }
  const spreadsheetId = match[1];

  // シート名をURLから取得するか、最初のシートを使用
  const spreadsheet = await sheets.spreadsheets.get({
    spreadsheetId,
  });

  let range: string;
  const sheetNameMatch = url.match(/[#&]gid=(\d+)/);

  if (sheetNameMatch) {
    // gidがある場合は、対応するシート名を取得
    const sheet = spreadsheet.data.sheets?.find((s) => s.properties?.sheetId?.toString() === sheetNameMatch[1]);
    range = sheet?.properties?.title || spreadsheet.data.sheets?.[0]?.properties?.title || "Sheet1";
  } else {
    // gidがない場合は最初のシートを使用
    range = spreadsheet.data.sheets?.[0]?.properties?.title || "Sheet1";
  }

  const response = await sheets.spreadsheets.values.get({
    spreadsheetId,
    range,
  });

  return response.data.values as string[][];
};
