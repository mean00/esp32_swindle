// use crate::Config;
use std::env;
//
use std::path::PathBuf;
//

fn main() {
    embuild::espidf::sysenv::output();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dst = match env::var("CARGO_MANIFEST_DIR") {
        Ok(x) => x,
        _ => panic!("cannot get CARGO_MANIFEST_DIR"),
    };
    let build = PathBuf::from(dst);
    let triple = env::var("TARGET").unwrap(); // e.g., "riscv32imac-esp-espidf"
    let profile = env::var("PROFILE").unwrap(); // "debug" or "release"
    let _target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let native = build
        .parent()
        .unwrap()
        .join("target")
        .join(&triple)
        .join(&profile)
        .join("build");

    //
    //println!("cargo:rustc-link-arg=-Wl,--whole-archive");
    println!("cargo:rustc-link-arg=-lswindle_single");
    println!("cargo:rustc-link-arg=-lesprit_single_lib");
    println!("cargo:rustc-link-arg=-lesp32_ws2812");
    println!("cargo:rustc-link-arg=-lesp_driver_gpio");
    println!("cargo:rustc-link-arg=-lesp_driver_rmt");
    println!("cargo:rustc-link-arg=-lesp_pm");
    println!("cargo:rustc-link-arg=-lbt");
    println!("cargo:rustc-link-arg=-lprotocomm");
    println!("cargo:rustc-link-arg=-lhal");
    println!("cargo:rustc-link-arg=-lsoc");
    //println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    println!("cargo:rustc-link-arg=-Lesp-idf/esp_driver_gpio");
    println!("cargo:rustc-link-arg=-Lesp-idf/esp_driver_rmt");
    println!("cargo:rustc-link-arg=-Lesp-idf/esp_pm");
    println!("cargo:rustc-link-arg=-Lesp-idf/hal");
    println!("cargo:rustc-link-arg=-Lesp-idf/soc");
    println!("cargo:rustc-link-arg=-Lesp-idf/bt");
    println!("cargo:rustc-link-arg=-Lesp-idf/protocomm");
    //
    println!("cargo:rustc-link-arg=-Wl,-u,dedic_gpio_periph_signals");
    //
    println!("cargo:rustc-link-lib=static=stdc++");
    println!(
        "cargo:rustc-link-search=native={}/native/lib",
        native.display()
    );
    //println!("cargo:rerun-if-env-changed=IDF_PATH");
    //println!("cargo:rerun-if-changed=modules/swindle_wrapper");
}
