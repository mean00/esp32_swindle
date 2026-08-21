// use crate::Config;
//use embuild::cmake::Config;
use std::env;
use std::process::Command;
//
use std::fs;
use std::path::{Path, PathBuf};
/**
 *
 */
/// Depth-limited recursive search for `full_name` under `root` (used to find a
/// tool binary inside a toolchain install directory tree).
fn find_tool_in_tree(root: &Path, full_name: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_tool_in_tree(&path, full_name, depth - 1) {
                return Some(found);
            }
        } else if path.file_name().map(|n| n == full_name).unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

/// Search a tools root (either `<workspace>/.embuild/espressif/tools` or
/// `$IDF_TOOLS_PATH/tools`) for `full_name`. First try the exact per-chip
/// layout `<root>/<prefix>/<version>/<prefix>/bin`, then fall back to a
/// depth-limited tree walk. IDF 6.0 installs the unified `xtensa-esp-elf`
/// toolchain (containing `xtensa-esp32s3-elf-*` binaries) rather than a
/// per-chip directory, so the exact-layout fast path misses it.
fn find_tool_in_tools_root(root: &Path, prefix: &str, full_name: &str) -> Option<PathBuf> {
    // Fast path: <root>/<prefix>/<version>/<prefix>/bin/<full_name>
    if let Ok(entries) = fs::read_dir(root.join(prefix)) {
        for entry in entries.flatten() {
            let bin = entry.path().join(prefix).join("bin").join(full_name);
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    // Generic path: any toolchain dir under <root>.
    find_tool_in_tree(root, full_name, 6)
}

fn get_tool_path(prefix: &str, tool_name: &str) -> String {
    let full_name = format!("{}{}", prefix, tool_name);

    // 1. PATH lookup (works when `$IDF_PATH/export.sh` has been sourced).
    let output = Command::new("which")
        .arg(&full_name)
        .output()
        .expect("Failed to execute 'which'");
    if output.status.success() {
        return String::from_utf8(output.stdout).unwrap().trim().to_string();
    }

    // 2. Tool install dirs: the project-local embuild dir and the global
    //    IDF_TOOLS_PATH dir. This makes a plain `cargo build` work without
    //    sourcing export.sh.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    // The workspace root is two levels above the target dir (same derivation
    // as find_sdkconfig_include above).
    let workspace = target_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(&manifest_dir);

    let mut tools_roots: Vec<PathBuf> = Vec::new();
    tools_roots.push(workspace.join(".embuild").join("espressif").join("tools"));
    if let Ok(idf_tools_path) = env::var("IDF_TOOLS_PATH") {
        if !idf_tools_path.trim().is_empty() {
            tools_roots.push(PathBuf::from(idf_tools_path).join("tools"));
        }
    }

    for root in &tools_roots {
        if let Some(bin) = find_tool_in_tools_root(root, prefix, &full_name) {
            println!(
                "cargo:warning=Found {} at {} (fallback from tools dir)",
                full_name,
                bin.display()
            );
            return bin.to_string_lossy().into_owned();
        }
    }

    panic!(
        "Could not find {}{} in PATH. Did you source export.sh?",
        prefix, tool_name
    );
}
/*
 *
 *
 */
fn find_sdkconfig_include() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));

    let triple = env::var("TARGET").unwrap(); // e.g., "riscv32imac-esp-espidf"
    let profile = env::var("PROFILE").unwrap(); // "debug" or "release"

    let build_root = target_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join(&triple)
        .join(&profile)
        .join("build");

    println!(
        "cargo:warning=Trying triplet = {} profile = {} build_root = {}",
        triple,  //.display(),
        profile, //.display(),
        build_root.display()
    );

    let mut candidate: Option<PathBuf> = None;
    if let Ok(entries) = fs::read_dir(&build_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("esp-idf-sys-") {
                let p = entry.path().join("out").join("build").join("config");
                println!("cargo:warning=Candidate  {}", p.display());
                if p.join("sdkconfig.h").exists() {
                    candidate = Some(p);
                    println!("cargo:warning=  ==> OK");
                    break;
                }
            }
        }
    }

    candidate.expect("Could not locate sdkconfig.h under target/<triple>/<profile>/build/esp-idf-sys-*/out/build/config")
}
//

fn main() {
    // Watch the swindle submodule sources + the wrapper: editing them (e.g.
    // toolchain.cmake, esprit mcu sources, platform files) must re-run this
    // build script so the C/C++ static libs are rebuilt.
    println!("cargo:rerun-if-changed=../modules/swindle_wrapper");
    println!("cargo:rerun-if-changed=../modules/swindle");
    println!("cargo:rerun-if-changed=../modules/swindle/esprit");
    println!("cargo:rerun-if-env-changed=MCU");
    println!("cargo:rerun-if-env-changed=SWINDLE_SIZE");
    println!("cargo:rerun-if-env-changed=SWINDLE_NRST");
    let _idf_path = env::var("IDF_PATH").expect("IDF_PATH must be set to your ESP-IDF checkout");
    let config = find_sdkconfig_include();

    let env_mcu = std::env::var("MCU").unwrap_or_else(|_| "unknown".to_string());

    let triplet: &str;
    let ln_esp_mcu: &str;
    let _mcu: &str;
    let _arch: &str;

    (triplet, _mcu, ln_esp_mcu, _arch) = match env_mcu.as_str() {
        "esp32c6" => {
            println!("cargo:warning=Configuring for ESP32-C6 (RISC-V)");
            ("riscv32-esp-elf", "esp32c6", "ESP32C6", "riscv")
        }
        "esp32s3" => {
            println!("cargo:warning=Configuring for ESP32-S3 (Xtensa)");
            ("xtensa-esp32s3-elf", "esp32s3", "ESP32S3", "xtensa")
        }
        "esp32c3" => {
            println!("cargo:warning=Configuring for ESP32-C3 (riscv)");
            ("riscv32-esp-elf", "esp32c3", "ESP32C3", "riscv")
        }
        _ => {
            // Handle cases like "esp32" (original) or "unknown"
            println!("cargo:warning=Unknown or generic MCU detected: {}", env_mcu);
            panic!("ops");
            //("xxx", "xxx", "xxx", "xxx")
        }
    };
    println!("cargo:warning=Collecting tools ");
    let ar = get_tool_path(triplet, "-ar");
    let ranlib = get_tool_path(triplet, "-ranlib");
    let cc = get_tool_path(triplet, "-gcc");
    let cxx = get_tool_path(triplet, "-g++");
    // Board type -> pinout selection. `SWINDLE_SIZE` carries the raw board id
    // (dev / zero / alternatezero ...), forwarded verbatim to CMake as
    // LN_ESP_BOARD and as a -DLN_BOARD_SIZE_<UPPER> define. The latter is what
    // lnBMP_pinout_external.h dispatches on to pick the matching
    // lnBMP_pinout_external_<board>.h. "mini" and "full" are legacy aliases of
    // "zero" and "dev" (same pinout headers).
    let board = env::var("SWINDLE_SIZE").unwrap_or("dev".to_string());
    let ln_esp_board = match board.as_str() {
        "mini" => "zero",
        "full" => "dev",
        _ => board.as_str(),
    };
    println!(
        "cargo:warning=Board: {} (SWINDLE_SIZE={}, LN_BOARD_SIZE_{})",
        ln_esp_board,
        board,
        ln_esp_board.to_uppercase()
    );

    // Reset polarity: "inverted" drives NRST through a MOSFET (active-high),
    // "straight" uses the default open-drain (active-low) reset pin. Forwarded
    // to CMake as USE_INVERTED_NRST, which selects bmp_reset_inv.cpp vs
    // bmp_reset.cpp in modules/swindle/swindle/swindle_target.cmake.
    let nrst = env::var("SWINDLE_NRST").unwrap_or("straight".to_string());
    let use_inverted_nrst = match nrst.as_str() {
        "inverted" => "ON",
        _ => "OFF",
    };
    println!(
        "cargo:warning=Reset polarity: {} (USE_INVERTED_NRST={})",
        nrst, use_inverted_nrst
    );

    println!("cargo:warning=Collecting paths ");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_path = out_dir.join("../../native");
    let mut cfg = cmake::Config::new("../modules/swindle_wrapper");
    cfg.pic(false) // bare-metal firmware: -fPIC creates a .got.plt section that IDF's sections.ld discards
        .define("CMAKE_C_COMPILER", &cc)
        .define("CMAKE_CXX_COMPILER", &cxx)
        .define("CMAKE_ASM_COMPILER", &cc)
        .define("CMAKE_AR", &ar)
        .define("CMAKE_RANLIB", &ranlib)
        .define("CMAKE_SYSTEM_NAME", "Generic")
        .define("CMAKE_C_COMPILER_WORKS", "ON")
        .define("CMAKE_CXX_COMPILER_WORKS", "ON")
        .define("CMAKE_ASM_COMPILER_WORKS", "ON")
        .define("LN_ESP_MCU", &ln_esp_mcu)
        .define("LN_ESP_BOARD", &ln_esp_board)
        .define("USE_INVERTED_NRST", &use_inverted_nrst)
        .define("CMAKE_INSTALL_PREFIX", &install_path)
        .cflag(format!("-DLN_BOARD_SIZE_{}=1", ln_esp_board.to_uppercase()))
        .cxxflag(format!("-DLN_BOARD_SIZE_{}=1", ln_esp_board.to_uppercase()))
        .cflag(format!("-I{}", config.display()))
        .cxxflag(format!("-I{}", config.display()));
    // The esprit C/C++ is compiled against the toolchain's NEWLIB sysroot by
    // default. When the SDK is built with picolibc (CONFIG_LIBC_PICOLIBC=y,
    // the IDF 6.0 default), the IDF's esp_libc/platform_include headers take
    // their picolibc code paths, so the C/C++ must also compile against the
    // picolibc headers - otherwise newlib's stdio.h breaks on the missing
    // '__FILE' typedef (platform_include/sys/reent.h skips newlib's reent.h
    // whenever CONFIG_LIBC_NEWLIB is unset). Inject the toolchain's
    // picolibc.specs (the file IDF resolves itself via
    // `--print-file-name=picolibc.specs`); its -isystem entries point the
    // includes at picolibc, which is exactly what esp_libc's platform
    // `#include_next` wrappers expect.
    let sdkconfig_h = config.join("sdkconfig.h");
    let uses_picolibc = std::fs::read_to_string(&sdkconfig_h)
        .map(|s| s.contains("#define CONFIG_LIBC_PICOLIBC 1"))
        .unwrap_or(false);
    if uses_picolibc {
        println!(
            "cargo:warning=CONFIG_LIBC_PICOLIBC=y detected - compiling esprit C/C++ against picolibc headers"
        );
        cfg.cflag("-specs=picolibc.specs")
            .cxxflag("-specs=picolibc.specs");
    }
    // Optional sccache/ccache support: when ESP_IDF_SYS_C_COMPILER_LAUNCHER is
    // set, route every C/C++ compile through the launcher (see build_mini.sh).
    if let Ok(launcher) = env::var("ESP_IDF_SYS_C_COMPILER_LAUNCHER") {
        if !launcher.trim().is_empty() {
            cfg.define("CMAKE_C_COMPILER_LAUNCHER", &launcher)
                .define("CMAKE_CXX_COMPILER_LAUNCHER", &launcher);
        }
    }
    let _dst = cfg.build();
}
// EOF
