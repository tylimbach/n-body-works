use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/cuda");

    let dst = cmake::Config::new("src/cuda")
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();

    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=nbody");

    // copy dll to exe directory for runtime
    // OUT_DIR is a subdirectory inside target/{debug,release}/build/<crate>/out.
    let dll_src = dst.join("bin").join("nbody.dll");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).expect("Failed to locate target directory");
    let dll_dst = target_dir.join("nbody.dll");

    fs::copy(&dll_src, &dll_dst).unwrap_or_else(|_| panic!("Failed to copy {:?} to {:?}", dll_src, dll_dst));
}
