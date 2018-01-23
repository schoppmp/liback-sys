extern crate oblivc;
extern crate liback_sys;

use std::thread;
use std::os::raw::{c_int, c_char};
use std::ffi::CString;
use self::oblivc::{ProtocolFn};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CArgs {
    pub argc: c_int,
    pub argv: *mut *mut c_char,
}

unsafe fn run_party(args: Vec<String>, p: c_int, run_fn: ProtocolFn) {
    // convert Vec<String> to C-style array of pointers
    let mut ptrs : Vec<_> = args.into_iter().map(
        |s| CString::new(s).unwrap().into_raw()
    ).collect();
    let mut cargs = CArgs {
        argc: ptrs.len() as c_int,
        argv: ptrs.as_mut_ptr()
    };
    let pd = oblivc::protocol_desc().party(p);
    let pd = if p == 1 {
        pd.accept("34512").expect("Accept failed")
    } else {
        pd.connect("localhost", "34512").expect("Connect failed")
    };
    pd.exec_yao_protocol(run_fn, &mut cargs);
    // reconstruct CStrings for them to be properly freed
    let _args : Vec<_> = ptrs.into_iter().map(|p| CString::from_raw(p)).collect();
}

pub unsafe fn run_test(args: Vec<String>, run_fn: ProtocolFn) {
    let server_args = args.clone();
    // pass one to the server, use the other as client
    let server = thread::spawn(move || run_party(server_args, 1, run_fn));
    run_party(args, 2, run_fn);
    server.join().unwrap();
}
