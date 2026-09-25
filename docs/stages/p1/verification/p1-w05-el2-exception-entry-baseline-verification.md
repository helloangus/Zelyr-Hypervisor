# P1-W05 exception-entry verification record

**Status:** Static mechanism review and host tests passed; executed faults pending.
**Date:** 2026-09-25 (Asia/Shanghai).
**Environment:** pinned Rust 1.98.1, AArch64 bare-metal target, W05 branch rebased on W04 merge `e47f4ac`.
**Implementation:** [W05 record](../implementation/p1-w05-el2-exception-entry-baseline-record.md).

| Validation | Result | Evidence boundary |
|---|---|---|
| W05-DV01/DV02 | Passed source/link inspection: 16 branch-only 128-byte slots, table 0x40081000–0x40082000, normal text from 0x40082000 | Does not prove an exception reaches its destination |
| W05-DV03 | Passed four host tests of all origin/category pairs, syndrome classes, FAR validity edges and stale asynchronous ESR rejection; frame layout assertions compile | Synthetic inputs, not hardware faults |
| W05-DV04 | Passed W04 C1–C4 API compilation and VBAR write–ISB–readback source review | No executing VBAR proof |
| W05-DV05 | Passed independent ordering/soundness review of guard, static frame, original x0–x30/SP, terminal stack reset and second-entry stop | Recursive physical fault injection belongs to W11 |
| W05-DV06 | Not run: NC3/NC6 and known-register injections belong to W11 after W09 integration | P1-V09 executed half unproven |
| W05-DV07 | Passed W06/W07/W08/W09/W11 interface handoff review, with explicit deferred post-arm/phase wiring | Consumers have not yet executed |

`cargo build --target aarch64-unknown-none-softfloat -p hypervisor` passed.
`cargo test --workspace --exclude hypervisor` passed 15 host tests (including
four W05 tests). `cargo clippy --target aarch64-unknown-none-softfloat -p
hypervisor -- -D warnings` passed. `llvm-objdump -h` and `llvm-nm -n` confirmed
the linked addresses above. Other local/online gates are recorded when run.
No QEMU boot, intentional exception, post-arm report, MMU-transition or real
hardware validation has run for W05. P1-V08/P1-V09 are not yet fully proven.
