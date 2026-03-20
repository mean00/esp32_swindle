// use crate::Config;
//use embuild::cmake::Config;
use std::env;
use std::process::Command;
//
use std::fs;
use std::path::PathBuf;
/**
 *
 */
fn get_tool_path(prefix: &str, tool_name: &str) -> String {
    let full_name = format!("{}{}", prefix, tool_name);
    let output = Command::new("which")
        .arg(full_name)
        .output()
        .expect("Failed to execute 'which'");

    if !output.status.success() {
        panic!(
            "Could not find {}{} in PATH. Did you source export.sh?",
            prefix, tool_name
        );
    }

    String::from_utf8(output.stdout).unwrap().trim().to_string()
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
    //let ln_esp_board = "mini";
    let board = env::var("ln_board").unwrap_or("default".to_string());
    let ln_esp_board = match board.as_str() {
        "mini" => "mini",
        _ => "wroom",
    };

    println!("cargo:warning=Collecting paths ");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_path = out_dir.join("../../native");
    let _dst = cmake::Config::new("../modules/swindle_wrapper")
        .define("CMAKE_C_COMPILER", &cc)
        .define("CMAKE_CXX_COMPILER", &cxx)
        .define("CMAKE_AR", &ar)
        .define("CMAKE_RANLIB", &ranlib)
        .define("CMAKE_SYSTEM_NAME", "Generic")
        .define("CMAKE_C_COMPILER_WORKS", "ON")
        .define("CMAKE_CXX_COMPILER_WORKS", "ON")
        .define("LN_ESP_MCU", &ln_esp_mcu)
        .define("LN_ESP_BOARD", &ln_esp_board)
        .define("CMAKE_INSTALL_PREFIX", &install_path)
        .cflag(format!("-I{}", config.display()))
        .cxxflag(format!("-I{}", config.display()))
        .build();
}
// EOF
