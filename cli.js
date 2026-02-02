#!/usr/bin/env node
/* eslint-env es6 */
const yargs = require("yargs")
  .usage(
    `
Usage: $0 [--config config_filename]

Generate localization files (Dart ARB, TypeScript) from CSV or Google Sheets.
Config file should be in YAML format.
If reading from Google Sheets, you need to configure credentials.
`
  )
  .options({
    config: {
      default: "l10n-generator.config.yaml",
    },
  })
  .describe({
    config: "Path to YAML config file",
  })
  .help()
  .alias("h", "help");

const argv = yargs.argv;

// helpオプションが指定されている場合は表示して終了
if (argv.help) {
  yargs.showHelp();
  process.exit(0);
}

// メインロジックを実行
const { cli } = require("./lib/index.js");

cli(argv.config).catch((error) => {
  console.error("予期しないエラーが発生しました:", error);
  process.exit(1);
});
