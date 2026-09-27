#!/usr/bin/env python3
"""Compile-only W03 target layout and rejected typestate misuse evidence.

Uses the repository-pinned rustc and exact production sources. It neither
executes target code nor proves stack fit. The canonical host test gate remains
cargo test --workspace --exclude hypervisor.
"""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
PREFIX = f'''#![no_std]
#[path = "{ROOT}/hypervisor/src/boot/address.rs"]
pub mod address;
pub mod boot {{ pub use crate::address; }}
#[path = "{ROOT}/hypervisor/src/platform/mod.rs"]
pub mod platform;
use platform::bootmap::{{BootMemoryMap, UnsealedMemoryMap, PageCount}};
'''
CASES = [
    ("target-layout", "aarch64-unknown-none-softfloat", "", '''
const _: () = assert!(core::mem::size_of::<UnsealedMemoryMap>() == 40768);
const _: () = assert!(core::mem::size_of::<BootMemoryMap>() == 40864);
const _: () = assert!(core::mem::size_of::<PageCount>() == 8);
'''),
    ("draft-is-not-authority", None, "E0308", '''
pub fn misuse(draft: UnsealedMemoryMap) { consume(&draft); }
fn consume(_: &BootMemoryMap) {}
'''),
    ("draft-cannot-seal-twice", None, "E0382", '''
pub fn misuse(draft: UnsealedMemoryMap) {
    let _ = draft.seal(&[]);
    let _ = draft.seal(&[]);
}
'''),
    ("sealed-cannot-reseal", None, "E0599", '''
pub fn misuse(sealed: BootMemoryMap) { let _ = sealed.seal(&[]); }
'''),
    ("sealed-summary-is-immutable", None, "E0594", '''
pub fn misuse(sealed: &mut BootMemoryMap) {
    sealed.summary().ram_frames = PageCount::new(0);
}
'''),
]
with tempfile.TemporaryDirectory(prefix="zelyr-w03-") as work:
    for name, target, expected, body in CASES:
        source = Path(work) / "probe.rs"
        source.write_text(PREFIX + body)
        cmd = ["rustc", "--edition=2024", "--crate-type=lib", "--emit=metadata",
               "--out-dir", work, str(source)]
        if target:
            cmd += ["--target", target, "-Dwarnings"]
        result = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)
        passed = (result.returncode == 0 if not expected else
                  result.returncode != 0 and f"error[{expected}]" in result.stderr)
        if not passed:
            raise SystemExit(f"FAIL {name}:\n{result.stdout}{result.stderr}")
        print(f"PASS {name}" + (f" ({expected})" if expected else ""))
