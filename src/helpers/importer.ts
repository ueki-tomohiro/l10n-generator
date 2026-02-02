import { importCSV } from "./importers/csv";
import { importGoogleSpreadSheetWithAPIKey } from "./importers/googleSheetsApiKey";
import { importGoogleSpreadSheetWithJWT } from "./importers/googleSheetsJwt";
import { importGoogleSpreadSheetWithOAuth2 } from "./importers/googleSheetsOAuth2";
import { importGoogleSpreadSheet } from "./importers/googleSheetsPublic";
import { Config } from "./type";

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
