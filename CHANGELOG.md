# Changelog

## 0.2.1 - 2026-04-21
- Fix TypeScript exporter to support dot-separated translation keys.
- Generate quoted properties in `translation.ts` and bracket access in `translateFunction.ts`.
- Add regression tests for dot key CSV inputs.

## 0.2.0 - 2026-02-06
- Breaking: migrate package to ESM ("type": "module"); `require()` no longer supported.
- Align CLI/build output with ESM and update yargs usage.
- Migrate ESLint and Prettier configs to ESM.
