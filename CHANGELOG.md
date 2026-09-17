# Changelog

## 0.3.0 - 2026-08-29
- Add `fileType: xlsx` to read localization data from a local xlsx file.
- Add `columns` / `skipRows` config to map an arbitrary column layout onto the internal `[key, description, ...locales]` shape. Configs without `columns` keep the previous fixed layout.
- Accept both column letters (`"B"`) and 0-based numeric indexes (`1`) in `columns`.
- Add `sheetName` config to select a sheet for xlsx and Google Sheets sources.
- Emit unquoted properties and dot access from the TypeScript exporter when a key is a valid identifier; dot-separated keys keep quoted properties and bracket access.

## 0.2.1 - 2026-04-21
- Fix TypeScript exporter to support dot-separated translation keys.
- Generate quoted properties in `translation.ts` and bracket access in `translateFunction.ts`.
- Add regression tests for dot key CSV inputs.

## 0.2.0 - 2026-02-06
- Breaking: migrate package to ESM ("type": "module"); `require()` no longer supported.
- Align CLI/build output with ESM and update yargs usage.
- Migrate ESLint and Prettier configs to ESM.
