# P1-W11 negative/fault validation implementation record

**Status:** NC2–NC5 mechanisms and local paired runs; W11 closure not claimed.
**Scope:** Default-off NC2–NC5 selections and current-state reconciliation
for [P1-W11](../plans/p1-w11-negative-fault-validation.md).
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

## Integrated W09 trigger mechanism (2026-09-25)

The preceding foundation snapshot and table are historical. On the current
W09-enabled baseline, NC3 and NC4 attach after `FatalPath.complete`; NC5
attaches after `Stage1.complete`. The [trigger design correction](p1-w11-negative-fault-validation/04-trigger-reconciliation.md)
fixes the exact instructions and the NC5 Rust-volatile concern. The closed
validation features `p1-w11-nc3`, `p1-w11-nc4`, `p1-w11-nc5` are default-off
and mutually exclusive with each other and NC2. Their only runtime entry is
`fault_scenario(ScenarioId) -> !` in the AArch64 validation module, absent
from the default build; no runtime fault-mode state exists. NC3 emits
`.inst 0`, NC4 uses a static panic, and NC5 emits one `ldr` from typed VA
`0x5000_0000`. That VA shares L1 index 1 with W08's image but uses invalid
L2 index 128 rather than the image's L2 index 0. The selections do not
change normal boot controls. NC3/NC4 builds omit the unused W08 module from
compilation because their trigger terminates before Stage1.

The W11 `scripts/p1-w11-verify` verdict layer invokes only the W10
`scripts/qemu-runner` entry, twice per scenario. It retains the runner's
invocation, outcome and serial artifacts and adds exact scenario checks and
paired image identity in `summary.json`. It does not own QEMU command flags,
capture, timeout, or the W10 clean-boot oracle. It presently supports
NC2–NC5. NC1 needs the single runner's EL2-disabled profile, which is not
yet integrated; manual diagnostics cannot close it. NC6 remains blocked:
no genuine deterministic unexpected IRQ/FIQ/SError source is approved in
P1's masked, no-GIC runtime. A synchronous BRK/undefined instruction or
branch to a vector slot is not an NC6 proxy.

The only new unsafe segments are U-016 (NC3 Rust-to-exception instruction
glue) and U-017 (NC5 translation probe). Both have independent static
soundness approval in the [unsafe inventory](../../../security/unsafe-inventory.md)
and require executed evidence separately. No external ABI, production
public API, dependency, allocator, Guest, SMP, GIC, recovery or runtime
configuration mechanism was added. The earlier “no new unsafe” statement
applies only to the NC2 foundation; this section supersedes it for the
combined branch. W11 closure, P1-V18 and final P1-V19 remain open.

## W10-integrated NC1 profile and final local handoff (2026-09-26)

After W10 PR #55 entered `main` at `5319995`, this branch added the fixed
`p1-no-el2` profile to the **same** `scripts/qemu-runner` process owner. It
changes only `virt,virtualization=on` to `off` for the W01 environment case;
CPU, memory, serial, timeout, and image-selection grammar are unchanged.
Its oracle requires the exact EL rejection token and forbids runtime, Stable,
panic, fatal, and DTB-rejection tokens. Runner status 0 means this rejection
was observed, not that the hypervisor reached Stable. The W11 verdict layer
then checks the entire capture has exactly one P1 EL rejection line, matches
the image digest, and agrees across two runs. The [design amendment](p1-w11-negative-fault-validation/04-trigger-reconciliation.md)
owns this fixed profile. There is no in-image NC1 trigger, arbitrary QEMU
flag input, or second QEMU launcher.

NC1–NC5 now have local paired QEMU scenario evidence on the W10-integrated
source; NC4 also re-anchors W10 R2 as a real intentional panic control.
The [verification record](../verification/p1-w11-negative-fault-validation-verification.md)
contains the exact image identities and S1–S6 audit. NC6 remains blocked by
the absence of an approved genuine unexpected event source. W11 therefore
does **not** close P1-V18 or the P1 stage. No new production ABI, dependency,
runtime configuration, Guest, GIC or recovery mechanism was added.
