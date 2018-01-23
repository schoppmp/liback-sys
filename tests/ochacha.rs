mod common;
use std::env::args_os;
use std::os::raw::c_void;

// replace the value of `name` by the library containing this particular test
#[link(name = "test_ochacha", kind = "static")]
extern "C" {
    fn test_main(varg: *mut c_void);
}

#[test]
fn test_ochacha() {
    let mut args = Vec::new();
    // copy first argument from args_os
    let name = args_os().next().expect("argc == 0").into_string().unwrap();
    args.push(name);
    // optional: push additional arguments
    // args.push(...);
    // run the test
    unsafe { common::run_test(args, test_main); }
}
