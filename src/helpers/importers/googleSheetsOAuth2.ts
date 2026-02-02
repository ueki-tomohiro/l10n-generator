import { google } from "googleapis";
import { OAuth2ClientOptions } from "google-auth-library";

type ImportGoogleSpreadSheetWithOAuth2 = (url: string, option?: OAuth2ClientOptions) => Promise<string[][]>;

export const importGoogleSpreadSheetWithOAuth2: ImportGoogleSpreadSheetWithOAuth2 = async (url, option) => {
  if (!option) {
    throw new Error("OAuth2 credentials are required");
  }

  const client = new google.auth.OAuth2(option);

  // OAuth2トークンが設定されているか確認
  const credentials = client.credentials;
  if (!credentials.access_token && !credentials.refresh_token) {
    // トークンがない場合は認証URLを生成してエラーを投げる
    const authUrl = client.generateAuthUrl({
      access_type: "offline",
      scope: "https://www.googleapis.com/auth/spreadsheets.readonly",
    });
    throw new Error(`OAuth2 authentication required. Please visit: ${authUrl}`);
  }

  const sheets = google.sheets({ version: "v4", auth: client });

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
    const sheet = spreadsheet.data.sheets?.find(
      (s) => s.properties?.sheetId?.toString() === sheetNameMatch[1]
    );
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
