//! Wires the member's boot-image layout script into the link step.
//!
//! The script, the load address, and the entry layout are owned by the
//! P1-W02 detailed implementation design; this wiring is the P0 build-target
//! baseline's reserved linking extension position. Host-side build tooling
//! only: no hypervisor runtime code runs from here.

use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|error| panic!("cargo always sets CARGO_MANIFEST_DIR: {error}"));
    println!("cargo::rustc-link-arg=-T{manifest_dir}/link.ld");
    println!("cargo::rerun-if-changed=link.ld");
}
