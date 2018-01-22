extern crate make_cmd;
extern crate glob;
extern crate ar;
extern crate oblivc;

use std::path::Path;
use std::env;
use std::process::Command;
use std::fs::File;
use glob::glob;
use make_cmd::make;
use ar::Builder;

fn main() {
    let ack_path = Path::new("absentminded-crypto-kit");
    // update submodule, but ignore if that fails
    if !ack_path.join(".git").exists() {
        let _status = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    // tell the ack makefile where to find obliv-c
    env::set_var("OBLIVC_PATH", env::var("DEP_OBLIV_ROOT").unwrap());

    // the following is a hack to allow compilation with glibc >= 2.26
    // see https://github.com/samee/obliv-c/issues/48 and remove if the issue gets fixed
    env::set_var("CFLAGS", format!("-D_Float128=double {}",
        env::var("CFLAGS").unwrap_or("".to_string())));

    // start build
    make().current_dir(&ack_path).status().ok().and_then(
        |status| if status.success() { Some(()) } else { None }
    ).expect("Building liback failed");

    // export root and include paths
    let ack_include_path =  ack_path.join("src");
    let ack_lib_path= ack_path.join("build/lib");
    println!("cargo:root={}", ack_path.display());
    println!("cargo:include={}", ack_include_path.display());

    // tell cargo to link ack
    println!("cargo:rustc-link-search=native={}", ack_lib_path.display());
    println!("cargo:rustc-link-lib=static=ack");

    // archive all objects in the "tests" directory (will be linked by rust tests)
    let glob_pattern = ack_path.join("tests/*.oo");
    for test_path in glob(glob_pattern.to_str().unwrap()).unwrap_or_else(
        |e| panic!("Could not read glob pattern: {}: {}", glob_pattern.display(), e)
    ) {
        let test_path = test_path.unwrap_or_else(
            |e| panic!("Error traversing pattern: {}: {}", glob_pattern.display(), e)
        );
        let test_lib_name = format!("lib{}.a", test_path.file_stem().and_then(
            |s| s.to_str()).unwrap_or_else(
                || panic!("Invalid path: {}", test_path.display())
            )
        );
        Builder::new(File::create(ack_lib_path.join(&test_lib_name)).unwrap_or_else(
            |e| panic!("Could not create output file: {}: {}", test_lib_name, e)
        )).append_path(&test_path).unwrap_or_else(
            |e| panic!("Could not archive file: {}: {}", test_path.display(), e)
        );
    }
}
