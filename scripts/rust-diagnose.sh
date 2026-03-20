#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

args=("$@")
if [[ ${#args[@]} -gt 0 && "${args[0]}" == "--" ]]; then
  args=("${args[@]:1}")
fi

cargo run --manifest-path "$ROOT_DIR/rust/l10n-rust/Cargo.toml" -- diagnose "${args[@]}"
