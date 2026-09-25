# P1-W11 negative/fault validation verification record

**Status:** Foundation validation only; P1-V18 and P1-V19 open.
**Date:** 2026-09-25 (Asia/Shanghai).
**Scope:** [W11 implementation record](../implementation/p1-w11-negative-fault-validation-record.md)
over `main` baseline `4fd5700` plus this W11 branch's diff.
**Version:** v0.1.
**Owner/change context:** P1-W11 validation-image foundation.
**Supersedes:** None.

## Executed foundation checks

- `cargo fmt --all` — passed.
- `cargo test --workspace --exclude hypervisor` — passed: 34 test executions,
  including the two new NC2 source-shared assertions. Five W03 source tests
  also execute a second time in the NC2 integration-test crate; these are not
  five additional distinct product cases (29 distinct test cases).
- `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
  — passed.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor` — passed
  for the default selection.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor --features p1-w11-nc2`
  — passed for the NC2 selection.
- `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor --features p1-w11-nc2 -- -D warnings`
  — passed.
- `python3 scripts/test_p1_runner.py` — passed: 16 W10 runner foundation
  tests. This does not execute any NC scenario or validate a W11 profile.
- `git diff --check` — passed.

These checks prove source compilation and pure substitution/policy behavior;
they do not prove that W09 invokes W03, that the selected image faults, or
that the default linked ELF excludes every future trigger. The W03 entry is
currently dormant, so link-time garbage collection can obscure selected-code
presence in the final ELF until W09 integration.

## Scenario execution status (W11-DV03/DV04)

| Scenario | Run 1 | Run 2 | Current reason |
|---|---|---|---|
| NC1 unsupported execution | Not run | Not run | W11 scenario verdict/profile not integrated; W10 boot-smoke requires Stable |
| NC2 missing required fact | Not run | Not run | W09 phase call and W11 scenario verdict/profile not integrated; source-shared policy test is not execution |
| NC3 synchronous fault | Not run | Not run | W09 insertion hook and W11 scenario verdict/profile not integrated |
| NC4 panic | Not run | Not run | W09 insertion hook and W11 scenario verdict/profile not integrated |
| NC5 post-MMU fault | Not run | Not run | W09 MMU-on continuation and W11 scenario verdict/profile not integrated |
| NC6 unexpected vector | Not run | Not run | W09 Stable hook and W11 scenario verdict/profile not integrated |

No QEMU fault capture, paired-run concordance, terminal-marker observation,
or hardware fault evidence was collected. NC2 environment-only coverage is
unavailable on reference QEMU 8.2.2; the compile-time variant must be
identified as injected policy evidence when it runs. W10 R2 has not been
anchored to NC4.

## Scope/security review status (W11-DV05)

The S1–S6 review must be performed against the final integrated tree and
record its exact commit/image identity, evidence pointers, outcomes, and
owners for findings. It is **not run** on this foundation: S1 input/range,
S2 no-RWX, S3 unsafe inventory, S4 stage boundary, S5 no fault-path
relaxation, S6 linked default-image containment. The source-level default-off
feature selection and no-new-`unsafe` diff are partial S6/S3 facts, not a
passing S1–S6 review or P1-V19 evidence.

P1-V18 remains open because none of NC1–NC6 has paired real executions.
P1-V19 remains open because S1–S6 were not completed. W11-DV01 matrix
coverage is documented in the detailed design; W11-DV02 full binary
containment and W11-DV06 downstream handoff remain pending.
