import fs from "fs";
import { parse } from "csv-parse";
import { google } from "googleapis";
import isArray from "lodash/isArray";
import { Config } from "./type";
import fetch from "node-fetch";
import { JWT, OAuth2ClientOptions } from "google-auth-library";

type ImportCSV = (filePath: string) => Promise<string[][]>;
const importCSV: ImportCSV = async (filePath) => {
  const buffer = fs.readFileSync(filePath);

  const parser = parse(buffer);
  const lines: string[][] = [];
  for await (const record of parser) {
    if (isArray(record)) lines.push(record);
  }
  return lines;
};

type ImportGoogleSpreadSheet = (documentId: string) => Promise<string[][]>;
const importGoogleSpreadSheet: ImportGoogleSpreadSheet = async (documentId) => {
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

type ImportGoogleSpreadSheetWithAPIKey = (documentId: string, apiKey?: string) => Promise<string[][]>;
const importGoogleSpreadSheetWithAPIKey: ImportGoogleSpreadSheetWithAPIKey = async (documentId, apiKey) => {
  if (!apiKey) {
    throw new Error("API key is required");
  }

  // まずスプレッドシート情報を取得して最初のシート名を取得
  const metadataUrl = `https://sheets.googleapis.com/v4/spreadsheets/${documentId}?key=${apiKey}`;
  const metadataResponse = await fetch(metadataUrl);

  if (!metadataResponse.ok) {
    const error = await metadataResponse.json();
    throw new Error(`Failed to fetch spreadsheet metadata: ${error.error?.message || metadataResponse.statusText}`);
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

type ImportGoogleSpreadSheetWithJWT = (url: string, option?: JWT) => Promise<string[][]>;

const importGoogleSpreadSheetWithJWT: ImportGoogleSpreadSheetWithJWT = async (url, option) => {
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

type ImportGoogleSpreadSheetWithOAuth2 = (url: string, option?: OAuth2ClientOptions) => Promise<string[][]>;

const importGoogleSpreadSheetWithOAuth2: ImportGoogleSpreadSheetWithOAuth2 = async (url, option) => {
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

type ImportValues = (config: Config) => Promise<string[][]>;

export const importValues: ImportValues = async (config: Config) => {
  switch (config.fileType) {
    case "csv":
      return importCSV(config.path);
    case "sheet":
      switch (config.credentialType) {
        case "apiKey":
          return importGoogleSpreadSheetWithAPIKey(config.path, config.apiKey);
        case "oauth2":
          return importGoogleSpreadSheetWithOAuth2(config.path, config.oauth2);
        case "jwt":
          return importGoogleSpreadSheetWithJWT(config.path, config.jwt);
        case "none":
          return importGoogleSpreadSheet(config.path);
      }
  }
};
