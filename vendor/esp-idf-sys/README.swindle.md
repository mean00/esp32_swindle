# esp-idf-sys (vendored + patched)

This is the `esp-rs/esp-idf-sys` `master` branch at commit `8369f610`
(as referenced by `esp-idf-sys#408`, ESP-IDF v6.0 support), vendored locally so
that the following two local patches can never be lost by a git checkout
refresh.

## Patches applied (relative to `8369f610`)

1. **`src/include/esp-idf/bindings.h`**
   - Added `#include "network_provisioning/network_prov_mgr.h"` (and the
     `esp_netif`/`esp_wifi` includes it requires) to the `--blocklist-file`
     section, so bindgen generates the `network_prov_*` bindings that the app's
     provisioning code uses. ESP-IDF v6.0 removed the old `wifi_provisioning`
     component in favour of `network_provisioning` (an extra component,
     vendored in the workspace `components/` dir), which is why the crates.io
     and stock-master `bindings.h` do not cover it.

2. **`build/build.rs`** (in the `build_native` code path, function emitting the
   `cargo:rustc-link-arg` lines)
   - Wrapped the component `-l` link arguments in
     `-Wl,--start-group` … `-Wl,--end-group`. ESP-IDF v6.0 split the HAL into
     per-peripheral components (`esp_hal_*`), which breaks the single-pass
     static link order and caused undefined references (e.g. `esp_hal_rmt` /
     `esp_hal_ana_conv`). The link group makes the ordering irrelevant.
   - Note: the same treatment is applied in `app/build.rs` for the manual
     `-l` libs (blackmagic/esprit/native_code/extra_code).

3. **`build/native/cargo_driver.rs`**
   - Added opt-in compiler-launcher support: when the
     `ESP_IDF_SYS_C_COMPILER_LAUNCHER` env var is set, the ESP-IDF CMake build
     is configured with `CMAKE_C_COMPILER_LAUNCHER`/`CMAKE_CXX_COMPILER_LAUNCHER`
     pointing at it. `build_mini.sh` sets it to `sccache` when available, so the
     ~1200-object ESP-IDF C build is cached in `~/.cache/sccache` (which
     survives the `rm -Rf target .embuild` clean). Strict no-op when unset;
     `native_code/build.rs`/`extra_code/build.rs` honour the same var.

4. **`build/common.rs`** (`setup_clang_env`)
   - The ESP-IDF esp-clang's `libclang` (via the `~/.espup/esp-clang` symlink)
     is now preferred over any `clang` discovered on the toolchain `$PATH`.
     Without this, an activated ESP-IDF shell (PATH includes `/usr/bin`) makes
     the native driver find the generic system clang — whose `lib/` contains a
     `libclang.so` — and pass its dir as `LIBCLANG_PATH`, skipping the espup
     policy entirely. The system clang (e.g. clang 22) parses the newlib
     headers with a different set of `__riscv_*` macros for `-target riscv32`,
     so bindgen emits opaque placeholder types (`_address: u8`, size 1) for
     `timeval`, `itimerval`, `_reent`, ... which fail the crate's `checks::libc`
     compile-time type assertions on any cold (non-sccache) build
     (`libc/esp-idf-sys type mismatch for timeval size: esp-idf=1 libc=16`,
     `no field _stdin/_stdout/_stderr on type _reent`). Preferring the espup
     esp-clang libclang restores the correct ESP-IDF parsing for both riscv
     and xtensa targets, making the build independent of whether the caller's
     shell has the IDF environment activated. A discovered clang's own
     libclang is still used as a fallback when the symlink is absent.

5. **`build/native/cargo_driver.rs`** (native cmake flow, before the
   `cmake::Query::new` call)
   - Pre-emptively clears a stale cmake build dir whose `CMakeCache.txt`
     records a `CMAKE_HOME_DIRECTORY` that differs from the current `out` dir.
     The `cmake` crate's `Config::build()` does exactly this check internally
     (`maybe_clear`) and responds by deleting the *entire* build dir — but it
     runs *after* this file has already written the cmake file-api query
     (`.cmake/api/v1/query/client-cargo/*`) into that very dir, so the wipe
     silently deletes the query, the re-configure runs without a client query,
     no file-api replies are generated and `query.get_replies()` fails with
     `Failed to list cmake-file-api reply directory`. Triggered whenever
     `target/` is shared between environments whose absolute paths differ
     (host `/home/fx/...` vs devcontainer `/home/ubuntu/...`). Mirroring the
     check *before* `Query::new` keeps the query alive across the (now no-op)
     `maybe_clear`.


## Why not a git dependency?

With a `[patch.crates-io]` git dependency, `cargo update` (or a re-fetch of
`~/.cargo/git`) would restore the pristine source and silently drop both
patches, breaking the build (missing `network_prov_*` symbols / link cycle).
A local path dependency keeps them permanent and reviewable.

The workspace `Cargo.toml` references this crate with
`esp-idf-sys = { path = "vendor/esp-idf-sys" }`.
