# liback-sys
[![Build Status](https://travis-ci.org/schoppmp/liback-sys.svg?branch=master)](https://travis-ci.org/schoppmp/liback-sys)

This library's purpose is to allow adding the [Absentminded Crypto Kit (ACK)][1] as a dependency
to Rust projects using the [`oblivc`][2] crate.
Since ACK's functions are written in [Obliv-C][3], this library does not have a public interface.
However, adding it as a dependency will make sure ACK compiles and gets linked to the project.

[1]: https://bitbucket.org/jackdoerner/absentminded-crypto-kit/
[2]: https://schoppmp.github.io/doc/oblivc-rust/oblivc/
[3]: https://github.com/samee/obliv-c
