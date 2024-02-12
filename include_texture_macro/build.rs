//! This build script mainly exists just to ensure `OUT_DIR` is set for the macro,
//! but we can also use it to force a re-evaluation if `DEVKITPRO` changes.

use std::env::var;

fn main() {
    for var in ["OUT_DIR", "DEVKITPRO"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    if var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo:rustc-cfg=win");
    }
}
