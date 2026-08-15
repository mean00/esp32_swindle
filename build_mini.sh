#!/bin/sh
echo "ESP_IDF_PATH is {${IDF_PATH}}"

[[ -v ESP_IDF_PATH ]] || {
  echo "ESP_IDF_PATH not  set" >&2
  exit 1
}

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
rm -Rf target .embuild
# Build all packages in ONE cargo invocation: separate `-p` invocations give the
# esp-idf-sys build script a different per-invocation context (config/env), so
# cargo re-runs it per package and the ESP-IDF C build is repeated in a second
# out dir. A single invocation builds esp-idf-sys exactly once.
cargo build -p native_code -p extra_code -p app --config 'env.ln_board="mini"' && bash flashme_c6.sh
