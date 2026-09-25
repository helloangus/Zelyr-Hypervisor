# P1-W11 negative/fault validation implementation record

**Status:** NC2 validation-image foundation only; W11 execution and closure
not claimed.
**Scope:** Default-off, build-selected NC2 capability-sample injection and
current-state reconciliation for [P1-W11](../plans/p1-w11-negative-fault-validation.md).
**Version:** v0.1.
**Owner/change context:** P1-W11 implementation, 2026-09-25.
**Supersedes:** None.

## Implemented boundary

The `hypervisor` feature `p1-w11-nc2` is a class-1 binary capability under
the [build switch governance](../../../development/build-profile-governance.md):
it includes one intentional validation-image behavior, is additive and
default-off, and is not a runtime policy, profile, or externally selectable
mode. The default image has neither the `validation_nc2` module nor its
injection call compiled in. The selection is for a test image only, not for
normal deployment.

At the W03 `ID_AA64MMFR0_EL1` acquisition boundary,
`inject_nc2_sample(raw: u64) -> u64` replaces only bits 31:28 (TGran4)
with `0xf`, an unsupported encoding. The same W03 decoder,
`verify_required` policy, `cap-reject fact=granule-4k` reason path, and
once-publication logic remain unchanged. The function is pure, stateless,
allocation-free, non-blocking, and has no `unsafe`. It does not impersonate a
CPU with missing hardware capability; its purpose is to exercise the real
required-fact policy after a controlled sample substitution. Source-shared
host tests cover all sixteen input field encodings, preservation of all other
bits and facts, idempotence, and the required rejection.

This is the NC2 validation-image variant authorized by the [scenario
matrix](p1-w11-negative-fault-validation/01-fault-scenario-matrix.md#nc2--missing-required-capability).
The environment-only NC2 variant remains unavailable on the reference QEMU
model; no DTB or EL2-entry rejection is relabeled as missing granule support.
The feature changes the compiled binary's behavior only when explicitly
selected. It is not a new production public API or external ABI.

## Scenario reconciliation against current main

The branch is rebased on `main` at `4fd5700` on 2026-09-25. The W01
rejection boundary, W03 policy, W05 vectors, W07 report format and W08
Stage-1 activation mechanism exist. The normal boot path still ends before
W09's eight-phase sequencer. W10's runner foundation and regression driver
are integrated, but their boot-smoke profile requires W09's Stable marker,
and W11-specific scenario selection/verdicts are not implemented. Therefore
the exact in-image insertion points for NC3–NC6 and an accepted fault-run
profile are not available on this baseline.

| Scenario | Fixed technique / diagnostic contract | Implementation state |
|---|---|---|
| NC1 | EL2-less environment; W01 rejection before runtime | Environment-only; W11 verdict/profile integration pending; no execution |
| NC2 | `p1-w11-nc2` sample substitution; W03 `cap-reject fact=granule-4k`, early panic route in `capabilities` | Injection and pure policy test implemented; phase-routed execution pending W09 and W11 runner integration |
| NC3 | Selected undefined instruction after vectors; W05/W07 `kind=E`, syndrome and phase | Trigger insertion pending W09's console-or-later hook |
| NC4 | Selected panic after console; W07 `kind=P` and terminal end marker | Trigger insertion pending W09; W10 R2 must use the real NC4 case |
| NC5 | Selected unmapped volatile access after W08 MMU-on continuation; W07 `kind=E` with FAR | Trigger insertion pending W09's post-MMU hook |
| NC6 | Selected unexpected category at Stable; W05/W07 `kind=E`, `ph=stable` | Trigger insertion pending W09's Stable hook |

The W07 prefixes are `ZELYR P1 PANIC kind=P` and `ZELYR P1 FATAL kind=E`;
its terminal line is `ZELYR P1 REPORT END`. These are source contracts,
not observed fault output. The unowned pre-vector window remains excluded.
No NC3–NC6 trigger, runtime mode switch, guest mechanism, recovery policy,
IRQ/GIC injection, or custom runner was introduced in this foundation.

## Validation and open work

[The verification record](../verification/p1-w11-negative-fault-validation-verification.md)
separates host/target compilation evidence from the six required paired QEMU
runs and S1–S6 scope/security review. Before W11 closure, integrate W09's
real hooks, add bounded scenario handling to W10's one runner, execute
NC1–NC6 twice, inspect the
default image for absence of reachable triggers, perform S1–S6 on the final
integrated tree, and hand the resulting limitations/evidence map to W12.

Changed runtime/build files: `hypervisor/Cargo.toml`, W03
`capabilities/mod.rs`, `capabilities/validation_nc2.rs`; host test:
`crates/host-test-baseline/tests/p1_w11_nc2.rs`. No new dependency, `unsafe`,
external ABI, or production public API was added. No unresolved architecture
or ADR conflict was found. W09/W10 availability is an integration
prerequisite, not permission to invent substitute interfaces.
