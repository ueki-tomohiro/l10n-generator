import fetch from "node-fetch";

type ImportGoogleSpreadSheetWithAPIKey = (documentId: string, apiKey?: string) => Promise<string[][]>;

export const importGoogleSpreadSheetWithAPIKey: ImportGoogleSpreadSheetWithAPIKey = async (documentId, apiKey) => {
  if (!apiKey) {
    throw new Error("API key is required");
  }

  // まずスプレッドシート情報を取得して最初のシート名を取得
  const metadataUrl = `https://sheets.googleapis.com/v4/spreadsheets/${documentId}?key=${apiKey}`;
  const metadataResponse = await fetch(metadataUrl);

  if (!metadataResponse.ok) {
    const error = await metadataResponse.json();
    const errorMessage = error.error?.message || metadataResponse.statusText;

    // より詳細なエラーメッセージを提供
    let detailedMessage = `Failed to fetch spreadsheet metadata: ${errorMessage}\n`;

    if (errorMessage.includes("permission") || errorMessage.includes("Permission")) {
      detailedMessage += `\n解決方法:\n`;
      detailedMessage += `1. スプレッドシートの共有設定を確認してください\n`;
      detailedMessage += `   - スプレッドシートを開く\n`;
      detailedMessage += `   - 右上の「共有」ボタンをクリック\n`;
      detailedMessage += `   - 「リンクを知っている全員」に変更\n`;
      detailedMessage += `   - 権限を「閲覧者」に設定\n`;
      detailedMessage += `2. Google Cloud ConsoleでAPIキーの制限を確認してください\n`;
      detailedMessage += `3. Google Sheets APIが有効化されているか確認してください`;
    }

    throw new Error(detailedMessage);
  }

  const metadata = await metadataResponse.json();
  const sheetName = metadata.sheets?.[0]?.properties?.title || "Sheet1";

  // データを取得
  const url = `https://sheets.googleapis.com/v4/spreadsheets/${documentId}/values/${encodeURIComponent(sheetName)}?key=${apiKey}`;
  const response = await fetch(url);

  if (!response.ok) {
    const error = await response.json();
    throw new Error(`Failed to fetch spreadsheet values: ${error.error?.message || response.statusText}`);
  }

  const document = await response.json();
  return document.values || [];
};
