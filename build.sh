#!/usr/bin/env bash
set -euo pipefail
export SQLX_OFFLINE=true
# List all the targets you want to build
TARGETS=(
    aarch64-unknown-linux-gnu
    x86_64-pc-windows-gnu
    x86_64-unknown-linux-gnu
    aarch64-unknown-linux-musl
    x86_64-unknown-linux-musl

)

# Base output directory in your current working directory
OUTDIR="$(pwd)/releases"
mkdir -p "$OUTDIR"

echo "Building for targets: ${TARGETS[*]}"
echo "All artifacts will be collected under $OUTDIR"
echo

# Derive your binary base name (without extension)
BIN_BASE="$(basename "$(pwd)")"

for tgt in "${TARGETS[@]}"; do
  echo "→ Building $tgt"
  cross build --release --target "$tgt"

  # Determine built file(s)
  SRC_DIR="target/$tgt/release"
  EXT=""
  # for Windows target, append .exe
  if [[ "$tgt" == *"windows-gnu"* ]]; then
    EXT=".exe"
  fi

  # Compose new filename: e.g. myapp-x86_64-pc-windows-gnu.exe
  NEW_NAME="$BIN_BASE-$tgt$EXT"

  # Copy & rename
  cp "$SRC_DIR/$BIN_BASE$EXT" "$OUTDIR/$NEW_NAME"

  echo "  → $SRC_DIR/$BIN_BASE$EXT  →  $OUTDIR/$NEW_NAME"
  echo
done

echo "All done!"
