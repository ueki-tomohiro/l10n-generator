#!/usr/bin/env node

import yargs from "yargs/yargs";
import { hideBin } from "yargs/helpers";
import { cli } from "./index";
import { diagnose } from "./helpers/diagnose";

/**
 * 診断コマンドのハンドラ
 */
async function diagnoseHandler(argv: { config: string }): Promise<void> {
  await diagnose({ configFile: argv.config });
}

/**
 * メインコマンドのハンドラ
 */
async function mainHandler(argv: { config: string }): Promise<void> {
  try {
    await cli(argv.config);
  } catch (error) {
    console.error("予期しないエラーが発生しました:", error);
    process.exit(1);
  }
}

/**
 * CLIのメイン処理
 */
async function main() {
  const parser = yargs(hideBin(process.argv))
    .usage(
      `
Usage: $0 [options]

Generate localization files (Dart ARB, TypeScript) from CSV or Google Sheets.
Config file should be in YAML format.
If reading from Google Sheets, you need to configure credentials.
`
    )
    .command(
      "diagnose",
      "Diagnose Google Sheets API connection",
      (yargs) => {
        return yargs.option("config", {
          default: "test.config.yaml",
          describe: "Path to YAML config file",
          type: "string",
        });
      },
      (argv) => {
        diagnoseHandler(argv as { config: string }).catch((error) => {
          console.error(error);
          process.exit(1);
        });
      }
    )
    .options({
      config: {
        default: "l10n-generator.config.yaml",
        describe: "Path to YAML config file",
        type: "string",
      },
    })
    .help()
    .alias("h", "help")
    .example("$0", "Generate with default config file")
    .example("$0 --config custom.yaml", "Generate with custom config")
    .example("$0 diagnose", "Run diagnostics on test.config.yaml")
    .example("$0 diagnose --config custom.yaml", "Run diagnostics on custom config");

  const argv = await parser.argv;

  // サブコマンドが指定されていない場合はメインの処理を実行
  if (!argv._ || argv._.length === 0) {
    await mainHandler(argv as { config: string });
  }
}

// メイン処理を実行
main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
