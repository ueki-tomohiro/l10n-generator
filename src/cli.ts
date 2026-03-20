#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";
import yargs from "yargs/yargs";
import { hideBin } from "yargs/helpers";
import { cli } from "./index.js";
import { diagnose } from "./helpers/diagnose.js";
import { runOauth2Helper } from "./helpers/oauth2-helper.js";

type Engine = "auto" | "ts" | "rust";

interface EngineArgv {
  engine: Engine;
}

const currentFilePath = fileURLToPath(import.meta.url);
const projectRoot = path.resolve(path.dirname(currentFilePath), "..");
const rustManifestPath = path.join(projectRoot, "rust", "l10n-rust", "Cargo.toml");

function hasRustCli(): boolean {
  if (!fs.existsSync(rustManifestPath)) {
    return false;
  }
  const result = spawnSync("cargo", ["--version"], { stdio: "ignore" });
  return result.status === 0;
}

function resolveEngine(engine: Engine): Exclude<Engine, "auto"> {
  if (engine === "ts") {
    return "ts";
  }

  if (engine === "rust") {
    if (!hasRustCli()) {
      throw new Error("Rust CLIが利用できません。cargoとrust/l10n-rust/Cargo.tomlを確認してください。");
    }
    return "rust";
  }

  return hasRustCli() ? "rust" : "ts";
}

function runRustCli(args: string[]): never {
  const result = spawnSync("cargo", ["run", "--manifest-path", rustManifestPath, "--", ...args], {
    stdio: "inherit",
  });
  if (result.error) {
    throw result.error;
  }
  process.exit(result.status ?? 1);
}

/**
 * 診断コマンドのハンドラ
 */
async function diagnoseHandler(argv: { config: string } & EngineArgv): Promise<void> {
  const engine = resolveEngine(argv.engine);
  if (engine === "rust") {
    runRustCli(["diagnose", "--config", argv.config]);
  }
  await diagnose({ configFile: argv.config });
}

/**
 * メインコマンドのハンドラ
 */
async function mainHandler(argv: { config: string } & EngineArgv): Promise<void> {
  try {
    const engine = resolveEngine(argv.engine);
    if (engine === "rust") {
      runRustCli(["--config", argv.config]);
    }
    await cli(argv.config);
  } catch (error) {
    console.error("予期しないエラーが発生しました:", error);
    process.exit(1);
  }
}

/**
 * OAuth2ヘルパーのハンドラ
 */
async function oauth2HelperHandler(argv: EngineArgv): Promise<void> {
  try {
    const engine = resolveEngine(argv.engine);
    if (engine === "rust") {
      runRustCli(["oauth2-helper"]);
    }
    await runOauth2Helper();
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
    .options({
      config: {
        default: "l10n-generator.config.yaml",
        describe: "Path to YAML config file",
        type: "string",
      },
      engine: {
        choices: ["auto", "ts", "rust"] as const,
        default: "auto" as const,
        describe: "Execution engine (auto prefers Rust when available)",
        type: "string",
      },
    })
    .command(
      "$0",
      "Generate localization files",
      () => {},
      (argv) => {
        mainHandler(argv as { config: string } & EngineArgv).catch((error) => {
          console.error(error);
          process.exit(1);
        });
      }
    )
    .command(
      "diagnose",
      "Diagnose Google Sheets API connection",
      (yargs) => {
        return yargs.options({
          config: {
            default: "test.config.yaml",
            describe: "Path to YAML config file",
            type: "string",
          },
          engine: {
            choices: ["auto", "ts", "rust"] as const,
            default: "auto" as const,
            describe: "Execution engine (auto prefers Rust when available)",
            type: "string",
          },
        });
      },
      (argv) => {
        diagnoseHandler(argv as { config: string } & EngineArgv).catch((error) => {
          console.error(error);
          process.exit(1);
        });
      }
    )
    .command(
      "oauth2-helper",
      "Generate OAuth2 tokens for Google Sheets",
      (yargs) => {
        return yargs.option("engine", {
          choices: ["auto", "ts", "rust"] as const,
          default: "auto" as const,
          describe: "Execution engine (auto prefers Rust when available)",
          type: "string",
        });
      },
      (argv) => {
        oauth2HelperHandler(argv as EngineArgv).catch((error) => {
          console.error(error);
          process.exit(1);
        });
      }
    )
    .help()
    .alias("h", "help")
    .example("$0", "Generate with default config file")
    .example("$0 --config custom.yaml", "Generate with custom config")
    .example("$0 diagnose", "Run diagnostics on test.config.yaml")
    .example("$0 diagnose --config custom.yaml", "Run diagnostics on custom config")
    .example("$0 oauth2-helper", "Run OAuth2 helper")
    .example("$0 --engine ts --config custom.yaml", "Force TypeScript engine")
    .strict();

  await parser.parseAsync();
}

// メイン処理を実行
main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
