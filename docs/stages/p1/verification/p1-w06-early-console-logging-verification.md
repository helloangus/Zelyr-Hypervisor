# P1-W06 Early Console — Verification Record

**Status:** Mechanism review and local gates; integrated execution pending.  
**Date:** 2026-09-25 (Asia/Shanghai).  
**Environment:** pinned Rust 1.98.1, AArch64 bare-metal target, branch `p1/w06-early-console`.  
**Design:** [W06 detailed design](../implementation/p1-w06-early-console-logging/README.md).  
**Implementation:** [W06 record](../implementation/p1-w06-early-console-logging-record.md).

| ID | Parent | Result and evidence | Proof boundary |
|---|---|---|---|
| W06-DV01 | W06 closure | Passed review of P0 channel semantics, W02 identity and P0 failure classification against the reconciliation | Classification is coherent; no serial output observed |
| W06-DV02 | P1-V10 | Passed target Clippy and code review of readback, one-time flag, fixed prefix and bounded framing | Mechanism compiles; W09 has not invoked it |
| W06-DV03 | P1-V10 | Partially reviewed W03/W05/W07/W08/W09 seams; executable consumers pending | No guarded exception or post-MMU proof |
| W06-DV04 | P1-V10/P1-V15 | Passed source review of fixed reference address, output-path inventory and no platform-name branch | Does not prove PL011 behavior |
| W06-DV05 | P1-V10 | Not run; W10 owns entry-to-stable and post-MMU marker execution | P1-V10 executed half remains unproven |
| W06-DV06 | W06 closure | Passed handoff review of W07 transport, W08 region, W09 marker format and W10 tokens | Downstream delivery remains separate |

Local execution so far: `cargo fmt --all`,
`cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings`,
and `cargo test --workspace --exclude hypervisor` (six host tests passed).
The host suite exercises W03 logic only and proves no PL011 behavior.
The full integration gate set, second unsafe review and QEMU execution
are required before W06 is merged or its executed acceptance is claimed.
