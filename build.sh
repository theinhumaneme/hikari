#!/usr/bin/env bash
set -euo pipefail
export SQLX_OFFLINE=true

BIN_BASE="hikari"

# Pass one or more targets to override the default release matrix.
if [[ "$#" -gt 0 ]]; then
  TARGETS=("$@")
else
  TARGETS=(
      aarch64-apple-darwin
      x86_64-apple-darwin
      aarch64-unknown-linux-gnu
      x86_64-unknown-linux-gnu
      aarch64-unknown-linux-musl
      x86_64-unknown-linux-musl
      x86_64-pc-windows-gnu
  )
fi

# Base output directory in your current working directory
OUTDIR="$(pwd)/releases"
mkdir -p "$OUTDIR"

echo "Building for targets: ${TARGETS[*]}"
echo "All artifacts will be collected under $OUTDIR"
echo

run_build() {
  local tgt="$1"

  case "$tgt" in
    *-apple-darwin)
      cargo build --release --target "$tgt"
      ;;
    *)
      cross build --release --target "$tgt"
      ;;
  esac
}

for tgt in "${TARGETS[@]}"; do
  echo "→ Building $tgt"
  run_build "$tgt"

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
