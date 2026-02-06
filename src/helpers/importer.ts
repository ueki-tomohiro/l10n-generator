import { importCSV } from "./importers/csv.js";
import { importGoogleSpreadSheetWithAPIKey } from "./importers/googleSheetsApiKey.js";
import { importGoogleSpreadSheetWithJWT } from "./importers/googleSheetsJwt.js";
import { importGoogleSpreadSheetWithOAuth2 } from "./importers/googleSheetsOAuth2.js";
import { importGoogleSpreadSheet } from "./importers/googleSheetsPublic.js";
import { Config } from "./type.js";

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
