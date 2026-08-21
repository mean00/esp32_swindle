#!/usr/bin/env bash
# build.sh — build the Swindle firmware from a named profile ("preset").
#
# A preset (see presets.toml) is a pair {mcu, board}:
#   - mcu   : target chip (esp32s3 / esp32c3 / esp32c6) — drives the Rust
#             target triple and, via esp-idf-sys, the chip cfgs.
#   - board : board type — selects the shared pinout header
#             modules/swindle_wrapper/include/lnBMP_pinout_external_<board>.h
#             on the C/C++ side and emits cfg(swindle_board_<board>) to Rust.
#
# Usage:
#   ./build.sh --preset <name> [--inverted] [--release] [--clean]
#
# The compiled binary is copied to target/swindle_<mcu>_<board>[_inverted].
set -u

ROOT="$(cd "$(dirname "$0")" && pwd)"
PRESETS_FILE="$ROOT/presets.toml"
DEFAULT_PRESET="esp32s3_dev"

usage() {
  cat <<'EOF'
Usage: ./build.sh --preset <name> [--inverted] [--release] [--clean]

Builds the Swindle firmware for the named preset (MCU + board type).

Options:
  --preset <name>  Preset from presets.toml, e.g. esp32s3_dev, esp32c6_alternatezero.
  --inverted       Drive NRST through a MOSFET (active-high). Default: straight.
  --release        Build in release mode (default: debug).
  --list           List available presets and exit.
  --clean          Remove target/ and .embuild/ before building.
  -h, --help       Show this help.

The binary is copied to target/swindle_<mcu>_<board>[_inverted].
EOF
}

# --- presets.toml helpers ---------------------------------------------------
# read_preset <name> prints "mcu board" for the preset, or nothing if unknown.
read_preset() {
  awk -v want="$1" '
    function val() { line=$0; sub(/^[^=]*=[ \t]*"?/, "", line); sub(/"?[ \t]*$/, "", line); return line }
    /^\[preset\.[a-zA-Z0-9_]+\]/ {
      name=$0; sub(/^\[preset\./, "", name); sub(/\]$/, "", name); in_preset=(name==want)
      if (!in_preset) { mcu=""; board="" }
    }
    in_preset && /^mcu[ \t]*=/   { mcu=val() }
    in_preset && /^board[ \t]*=/ { board=val() }
    in_preset && mcu!="" && board!="" { print mcu, board; exit }
  ' "$PRESETS_FILE"
}

list_presets() {
  awk '
    /^\[preset\.[a-zA-Z0-9_]+\]/ { name=$0; sub(/^\[preset\./, "", name); sub(/\]$/, "", name); print name }
  ' "$PRESETS_FILE" | sort
}

# --- CLI parsing ------------------------------------------------------------
PRESET=""
NRST="straight"
PROFILE="debug"
CLEAN=0
LIST=0

while [ $# -gt 0 ]; do
  case "$1" in
    --preset) PRESET="${2:-}"; shift 2 ;;
    --inverted) NRST="inverted"; shift ;;
    --release) PROFILE="release"; shift ;;
    --clean) CLEAN=1; shift ;;
    --list) LIST=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Error: unknown argument '$1'" >&2; usage >&2; exit 1 ;;
  esac
done

if [ "$LIST" -eq 1 ]; then
  echo "Available presets:"
  for p in $(list_presets); do
    read -r mcu board <<EOF
$(read_preset "$p")
EOF
    printf "  %-22s mcu=%-9s board=%s\n" "$p" "$mcu" "$board"
  done
  exit 0
fi

if [ -z "$PRESET" ]; then
  PRESET="$DEFAULT_PRESET"
  echo "No --preset given, using default preset '$PRESET'."
fi

if [ ! -f "$PRESETS_FILE" ]; then
  echo "Error: $PRESETS_FILE not found" >&2
  exit 1
fi

read -r MCU BOARD <<EOF
$(read_preset "$PRESET")
EOF
if [ -z "${MCU:-}" ] || [ -z "${BOARD:-}" ]; then
  echo "Error: preset '$PRESET' not found in $PRESETS_FILE" >&2
  exit 1
fi

# --- environment ------------------------------------------------------------
[ -n "${IDF_PATH:-}" ] || {
  echo "IDF_PATH is not set — source an ESP-IDF export script first." >&2
  exit 1
}

case "$MCU" in
  esp32s3) TARGET="xtensa-esp32s3-espidf" ;;
  esp32c6) TARGET="riscv32imac-esp-espidf" ;;
  esp32c3) TARGET="riscv32imc-esp-espidf" ;;
  *) echo "Error: preset '$PRESET' uses unsupported MCU '$MCU'" >&2; exit 1 ;;
esac

export MCU
export SWINDLE_SIZE="$BOARD"
export SWINDLE_NRST="$NRST"

# Optional sccache acceleration (see build_all.sh for the rationale).
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER="$(command -v sccache)"
  export CARGO_INCREMENTAL=0
  export ESP_IDF_SYS_C_COMPILER_LAUNCHER="$(command -v sccache)"
fi

if [ "$CLEAN" -eq 1 ]; then
  echo "Cleaning target/ and .embuild/ ..."
  rm -Rf "$ROOT/target" "$ROOT/.embuild"
fi

echo "============================================================"
echo " Preset    : $PRESET"
echo " MCU       : $MCU"
echo " Board     : $BOARD  (lnBMP_pinout_external_${BOARD}.h)"
echo " NRST      : $NRST"
echo " Target    : $TARGET"
echo " Profile   : $PROFILE"
echo "============================================================"

# Build all packages in ONE cargo invocation (rationale in build_all.sh:
# separate -p invocations give esp-idf-sys a different per-invocation
# context, which re-runs its build script per package and produces multiple
# out dirs that native_code/extra_code would pick arbitrarily).
BUILD_ARGS=(--target "$TARGET" -p native_code -p extra_code -p app)
[ "$PROFILE" = "release" ] && BUILD_ARGS+=(--release)

if ! (cd "$ROOT" && cargo build "${BUILD_ARGS[@]}"); then
  echo "Build failed!" >&2
  exit 1
fi

OUT_DIR="$ROOT/target"
mkdir -p "$OUT_DIR"
if [ "$NRST" = "inverted" ]; then
  BIN_SUFFIX="_inverted"
else
  BIN_SUFFIX=""
fi
BINARY="$OUT_DIR/swindle_${MCU}_${BOARD}${BIN_SUFFIX}"
cp "$ROOT/target/$TARGET/$PROFILE/swindle_s3" "$BINARY"
echo ""
echo "===================================================================="
echo "Build successful! Binary copied to: $BINARY"
echo "To flash: espflash flash -M $BINARY"
echo "===================================================================="
