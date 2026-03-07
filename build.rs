// use crate::Config;
use embuild::cmake::Config;
use std::env;
use std::process::Command;
//
use std::fs;
use std::path::PathBuf;
//

fn main() {
    embuild::espidf::sysenv::output();
   // Link your library name (adjust to actual target the C++ project produces)
    // // If the library target is "mylib", CMake will produce "libmylib.a"
    //println!("cargo:rustc-link-lib=static=swindle_interface");
    //println!("cargo:rustc-link-lib=static=libswindle");
    //println!("cargo:rustc-link-lib=static=swindle");
    // Link the C++ runtime
    //println!("cargo:rustc-link-lib=dylib=stdc++");
    //println!("cargo:rerun-if-env-changed=IDF_PATH");
    //println!("cargo:rerun-if-changed=modules/swindle_wrapper");
}
