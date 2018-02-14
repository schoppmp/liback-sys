extern crate make_cmd;
extern crate glob;
extern crate ar;
extern crate libobliv_sys;
extern crate walkdir;

use std::path::Path;
use std::env;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use glob::glob;
use make_cmd::make;
use ar::Builder;
use walkdir::WalkDir;

fn main() {
    let ack_path = env::current_dir().unwrap().join("absentminded-crypto-kit");
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
    let ack_src_path =  ack_path.join("src");
    let ack_lib_path= ack_path.join("build/lib");
    println!("cargo:root={}", ack_path.display());
    println!("cargo:include={}", ack_src_path.display());

    // tell cargo to link ack and OpenMP
    println!("cargo:rustc-link-search=native={}", ack_lib_path.display());
    println!("cargo:rustc-link-lib=static=ack");
    println!("cargo:rustc-link-lib=dylib=gomp");

    // register to rebuild when ACK sources change
    let register_dir_rebuild = |dir: &AsRef<Path>| {
        for file in WalkDir::new(dir)
                            .into_iter()
                            .filter_map(|e| e.ok()) {
            println!("cargo:rerun-if-changed={}", file.path().display());
        }
    };
    let ack_tests_path = ack_path.join("tests");
    register_dir_rebuild(&ack_src_path);
    register_dir_rebuild(&ack_tests_path);

    // generate archive files and rust wrappers for tests
    let glob_pattern = ack_tests_path.join("test_*.oo");
    for test_path in glob(glob_pattern.to_str().unwrap()).unwrap_or_else(
        |e| panic!("Could not read glob pattern: {}: {}", glob_pattern.display(), e)
    ) {
        let test_path = test_path.unwrap_or_else(
            |e| panic!("Error traversing pattern: {}: {}", glob_pattern.display(), e)
        );
        let test_name = test_path.file_stem().and_then(|s| s.to_str()).unwrap_or_else(
            || panic!("Invalid path: {}", test_path.display()
        ));
        // pack test object into library
        let test_lib_name = format!("lib{}.a", test_name);
        Builder::new(File::create(ack_lib_path.join(&test_lib_name)).unwrap_or_else(
            |e| panic!("Could not create output file: {}: {}", test_lib_name, e)
        )).append_path(&test_path).unwrap_or_else(
            |e| panic!("Could not archive file: {}: {}", test_path.display(), e)
        );
        // generate rust test case
        let mut test_wrapper_file = File::create(format!("tests/{}.rs", test_name))
            .expect("Error creating test wrapper file");
        test_wrapper_file.write_all(format!("
mod common;
#[link(name = \"{test_name}\", kind = \"static\")]
extern \"C\" {{ fn test_main(varg: *mut ::std::os::raw::c_void); }}
#[test]
fn {test_name}() {{ unsafe {{ common::run_test(test_main); }} }}
        ", test_name=test_name).as_bytes())
            .expect("Error writing to test wrapper file");
    }
}
