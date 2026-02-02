import fetch from "node-fetch";

type ImportGoogleSpreadSheet = (documentId: string) => Promise<string[][]>;

export const importGoogleSpreadSheet: ImportGoogleSpreadSheet = async (documentId) => {
  // まずスプレッドシート情報を取得して最初のシート名を取得
  const metadataUrl = `https://sheets.googleapis.com/v4/spreadsheets/${documentId}`;
  const metadataResponse = await fetch(metadataUrl);

  if (!metadataResponse.ok) {
    const error = await metadataResponse.json();
    throw new Error(`Failed to fetch spreadsheet metadata: ${error.error?.message || metadataResponse.statusText}`);
  }

  const metadata = await metadataResponse.json();
  const sheetName = metadata.sheets?.[0]?.properties?.title || "Sheet1";

  // データを取得
  const url = `https://sheets.googleapis.com/v4/spreadsheets/${documentId}/values/${encodeURIComponent(sheetName)}`;
  const response = await fetch(url);

  if (!response.ok) {
    const error = await response.json();
    throw new Error(`Failed to fetch spreadsheet values: ${error.error?.message || response.statusText}`);
  }

  const document = await response.json();
  return document.values || [];
};
