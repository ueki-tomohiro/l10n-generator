import { OAuth2ClientOptions, JWTOptions } from "google-auth-library";

const credentialTypes = ["apiKey", "oauth2", "jwt", "none"] as const;
export type CredentialType = typeof credentialTypes[number];

const fileTypes = ["csv", "sheet"] as const;
export type FileType = typeof fileTypes[number];

const outputTypes = ["dart", "typescript", "both"] as const;
export type OutputType = typeof outputTypes[number];

export type Config = {
  fileType: FileType;
  path: string;
  credentialType: CredentialType;
  apiKey?: string;
  oauth2?: OAuth2ClientOptions;
  jwt?: JWTOptions;
  localizePath: string;
  outputType?: OutputType;
};
