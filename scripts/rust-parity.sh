#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t l10n-rust-parity)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

TS_OUT="$TMP_DIR/ts-out"
RS_OUT="$TMP_DIR/rs-out"
TS_CONFIG="$TMP_DIR/ts.config.yaml"
RS_CONFIG="$TMP_DIR/rs.config.yaml"
SAMPLE_CSV="$ROOT_DIR/examples/sample-data.csv"

mkdir -p "$TS_OUT" "$RS_OUT"

cat > "$TS_CONFIG" <<YAML
fileType: csv
path: $SAMPLE_CSV
credentialType: none
localizePath: $TS_OUT
outputType: both
YAML

cat > "$RS_CONFIG" <<YAML
fileType: csv
path: $SAMPLE_CSV
credentialType: none
localizePath: $RS_OUT
outputType: both
YAML

pnpm -s build
node "$ROOT_DIR/lib/cli.js" --config "$TS_CONFIG" >/dev/null
cargo run --manifest-path "$ROOT_DIR/rust/l10n-rust/Cargo.toml" -- --config "$RS_CONFIG" >/dev/null

ts_listing="$(cd "$TS_OUT" && ls -1 | sort)"
rs_listing="$(cd "$RS_OUT" && ls -1 | sort)"

if [[ "$ts_listing" != "$rs_listing" ]]; then
  echo "Generated file sets differ between TypeScript and Rust." >&2
  echo "[TypeScript]" >&2
  echo "$ts_listing" >&2
  echo "[Rust]" >&2
  echo "$rs_listing" >&2
  exit 1
fi

status=0
while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  if ! cmp -s "$TS_OUT/$file" "$RS_OUT/$file"; then
    echo "Content mismatch: $file" >&2
    diff -u "$TS_OUT/$file" "$RS_OUT/$file" || true
    status=1
  fi
done <<< "$ts_listing"

if [[ $status -ne 0 ]]; then
  exit $status
fi

echo "Rust/TypeScript parity check passed (${ts_listing//$'\n'/, })."
