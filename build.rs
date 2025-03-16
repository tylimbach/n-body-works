use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let dst = cmake::Config::new("src/cuda")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .build();

    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=nbody");

    // Copy compile_commands.json to the project root if it's not there
    let compile_commands_path = Path::new(&dst).join("build/compile_commands.json");
    let root_compile_commands = Path::new("compile_commands.json");

    if compile_commands_path.exists() {
        if let Err(err) = fs::copy(&compile_commands_path, &root_compile_commands) {
            eprintln!("Failed to copy compile_commands.json: {}", err);
        }
    } else {
        eprintln!(
            "Warning: compile_commands.json was not found at {:?}",
            compile_commands_path
        );
    }

    let output_file = if cfg!(target_os = "windows") {
        "nbody.dll"
    } else {
        "libnbody.so"
    };

    let output_dir = if cfg!(target_os = "windows") {
        "bin"
    } else {
        "lib"
    };

    // copy dll to exe directory for runtime
    // OUT_DIR is a subdirectory inside target/{debug,release}/build/<crate>/out.
    let dll_src = dst.join(output_dir).join(output_file);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).expect("Failed to locate target directory");
    let dll_dst = target_dir.join(output_file);

    fs::copy(&dll_src, &dll_dst).unwrap_or_else(|_| panic!("Failed to copy {:?} to {:?}", dll_src, dll_dst));

    println!("cargo:rerun-if-changed=src/cuda");
}
