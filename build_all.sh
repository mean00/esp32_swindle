#!/bin/sh
echo "ESP_IDF_PATH is {${IDF_PATH}}"

[[ -v ESP_IDF_PATH ]] || {
  echo "ESP_IDF_PATH not  set" >&2
  exit 1
}

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
  echo "Usage: ./build_all.sh [MCU] [SIZE] [RESET]"
  echo ""
  echo "Builds the Swindle firmware for the specified MCU, board size and reset polarity."
  echo ""
  echo "Arguments:"
  echo "  MCU   The target MCU (e.g., esp32c3, esp32s3, esp32c6). Default: esp32c3"
  echo "  SIZE  The board size profile (e.g., full, mini, zero). Default: full"
  echo "                               (zero and mini is the same thing)"
  echo "  RESET The reset pin polarity:"
  echo "          straight - default open-drain (active-low) reset pin"
  echo "          inverted - reset driven through a MOSFET (active-high)"
  echo "        Default: straight"
  echo ""
  echo "Examples:"
  echo "  ./build_all.sh esp32c3 full"
  echo "  ./build_all.sh esp32s3 mini"
  echo "  ./build_all.sh esp32s3 zero inverted"
  echo ""
  echo "The compiled binary will be copied to the target/ directory with the name: swindle_MCU_SIZE[_inverted]"
  exit 0
fi

MCU_ARG=${1:-esp32c3}
PROFILE_ARG=${2:-full}
RESET_ARG=${3:-straight}

case "$RESET_ARG" in
  inverted | straight) ;;
  *)
    echo "Error: RESET must be 'inverted' or 'straight' (got '$RESET_ARG')" >&2
    exit 1
    ;;
esac

export MCU=$MCU_ARG
export SWINDLE_SIZE=$PROFILE_ARG
export SWINDLE_NRST=$RESET_ARG

if [ "$MCU" == "esp32s3" ]; then
  TARGET="xtensa-esp32s3-espidf"
elif [ "$MCU" == "esp32c6" ]; then
  TARGET="riscv32imac-esp-espidf"
else
  TARGET="riscv32imc-esp-espidf"
fi

echo "Building for $MCU with size $SWINDLE_SIZE and reset $SWINDLE_NRST (target $TARGET)..."

# --- optional sccache acceleration -------------------------------------------
# If sccache is available, cache BOTH the Rust crates (RUSTC_WRAPPER) and the
# C/C++ builds (CMAKE_*_COMPILER_LAUNCHER via ESP_IDF_SYS_C_COMPILER_LAUNCHER,
# honoured by the vendored esp-idf-sys and native_code/extra_code build.rs).
# The cache lives in ~/.cache/sccache - outside target/ and .embuild/ - so it
# survives the `rm -Rf target .embuild` below: a "clean" rebuild re-downloads
# the IDF tools but reuses every cached compilation. CARGO_INCREMENTAL=0 is
# required because sccache refuses to cache incremental rustc output; for this
# clean-build workflow sccache is the better cache anyway.
if command -v sccache >/dev/null 2>&1; then
  echo "sccache found at $(command -v sccache) - enabling Rust + C/C++ caching"
  export RUSTC_WRAPPER="$(command -v sccache)"
  export CARGO_INCREMENTAL=0
  export ESP_IDF_SYS_C_COMPILER_LAUNCHER="$(command -v sccache)"
else
  echo "sccache not found - building without caching"
fi
# rm -Rf target .embuild
# Build all packages in ONE cargo invocation (same rationale as build_mini.sh):
# separate `-p` invocations give the esp-idf-sys build script a different
# per-invocation context (config/env), so cargo re-runs it per package, creating
# multiple esp-idf-sys out dirs. native_code/extra_code then pick one of them by
# arbitrary readdir order, and when a stale build dir is reused the cmake
# configure loses the `-I<esp-idf-sys>/out/build/config` flag, so the swindle
# C++ fails to find sdkconfig.h. A single invocation builds esp-idf-sys exactly
# once and keeps that include path stable.
if cargo build --target $TARGET -p native_code -p extra_code -p app; then
  OUT_DIR="target"
  mkdir -p "$OUT_DIR"
  if [ "$SWINDLE_NRST" = "inverted" ]; then
    BIN_SUFFIX="_inverted"
  else
    BIN_SUFFIX=""
  fi
  BINARY="$OUT_DIR/swindle_${MCU}_${SWINDLE_SIZE}${BIN_SUFFIX}"
  cp "target/$TARGET/debug/swindle_s3" "$BINARY"
  echo ""
  echo "===================================================================="
  echo "Build successful! Binary copied to: $BINARY"
  echo "To flash: espflash flash -M $BINARY"
  echo "===================================================================="
else
  echo "Build failed!"
  exit 1
fi
