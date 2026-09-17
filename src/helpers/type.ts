import { OAuth2ClientOptions, JWTOptions } from "google-auth-library";

const credentialTypes = ["apiKey", "oauth2", "jwt", "none"] as const;
export type CredentialType = (typeof credentialTypes)[number];

const fileTypes = ["csv", "sheet", "xlsx"] as const;
export type FileType = (typeof fileTypes)[number];

const outputTypes = ["dart", "typescript", "both"] as const;
export type OutputType = (typeof outputTypes)[number];

/** 列の指定。列レター("B")または0始まりの数値インデックス(1) */
export type ColumnRef = string | number;

/**
 * 入力データの列レイアウト。
 * 未指定の場合は [key, description, locale...] の固定レイアウトとして扱う。
 */
export type ColumnMapping = {
  key: ColumnRef;
  description?: ColumnRef;
  /** 出力ファイル名(ロケール) と 列の対応。例: { ja: "C", en: "D" } */
  locales: Record<string, ColumnRef>;
};

export type Config = {
  fileType: FileType;
  path: string;
  credentialType: CredentialType;
  apiKey?: string;
  oauth2?: OAuth2ClientOptions;
  jwt?: JWTOptions | string; // JWTOptionsオブジェクトまたはJSONファイルパス
  localizePath: string;
  outputType?: OutputType;
  /** xlsx / sheet のシート名。未指定は先頭シート */
  sheetName?: string;
  /** 先頭から読み飛ばす行数 (既定 0)。columns 指定時のみ有効 */
  skipRows?: number;
  columns?: ColumnMapping;
};
