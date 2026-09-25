# P1-W07 fatal-report verification record

**Status:** Host/target checks and one real QEMU panic-path probe passed; fault execution pending.
**Date:** 2026-09-25 (Asia/Shanghai).
**Implementation:** [W07 record](../implementation/p1-w07-fatal-crash-diagnostics-record.md).

`cargo test --workspace --exclude hypervisor` passed 18 host tests, including
three source-shared W07 line-buffer tests for normal formatting, UTF-8-safe
truncation and later-fragment overflow. `cargo build --target
aarch64-unknown-none-softfloat -p hypervisor`, target and host Clippy with
`-D warnings`, `cargo fmt --all -- --check`, and `git diff --check` passed
after linking the merged W09 lifecycle foundation. The target build now
compiles W07 and the W09 tracker. Static checks alone do not prove report
field ordering on a running image or guard behavior under faults.

On merged `main` commit `4596074`, the target ELF was built and converted to
an ARM64 Image with the W10 draft image converter (`b76a6b4`). Image SHA-256:
`0e2bb3992b510dbc1ba6976514676c1fa3aeb2911a3ef5fbac3b2212a7eb6ca6`.
The W10 draft runner launched real `qemu-system-aarch64` (`virt`,
`virtualization=on`, `cortex-a57`, one CPU) with a three-second hard timeout.
Its full local capture is at
`target/p1-diagnostics/pre-w09-panic-evidence/` (build-tree evidence, not a
checked-in artifact). It returned status 4, reason `forbidden-marker`, in
0.29 seconds. Serial output contained one `ZELYR P1 PANIC kind=P`,
`cpu=EL2 el_ok=true`, `ph=unknown(raw=0)`, the expected unlinked-W09 seam
message, handler-entry SP/LR, `syndrome=na`, and `ZELYR P1 REPORT END`.
No `STABLE` token appeared. This is one executed panic-report path and a
runner detection check, not W10 R1, W07 fault recursion, or stable boot.

W07-DV01..DV06 remain in progress. NC3–NC6, post-MMU diagnostics, recursive
fault containment and hardware behavior have not run; their executed
evidence belongs to W11/W08/W10 after W09 integration.
As in W02/W06, a permanently busy UART may remain in the polling writer;
the report end marker is best-effort, not guaranteed output in that case.
