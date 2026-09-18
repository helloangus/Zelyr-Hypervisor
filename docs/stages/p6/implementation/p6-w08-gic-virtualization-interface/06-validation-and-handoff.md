# P6-W08 Validation, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md). This file closes the workflow in
[05-implementation-workflow.md](05-implementation-workflow.md).

## 1. Validation matrix

QEMU (virt, declared configuration) is the reference environment for all
runtime rows; **QEMU's GIC virtualization emulation succeeding does not
prove real-hardware correctness** — QEMU is the reference validation
environment, not an architecture definition (task book §1), and
real-hardware validation belongs to the Orange Pi stage. W08 rows prove
nothing about maintenance policy (W09), Guest-visible semantics (W10), or
the vIRQ lifecycle itself (W07).

| ID | Requirement (plan/task book) | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W08-DV01 | Virtualization-interface readiness (P6-V01 consumption) | readiness exercise + review | run readiness on the declared QEMU configuration; harness-injected mismatch fixtures for each error case; verify disabled-by-default posture | discovery values recorded and within the declared envelope; each mismatch produces its named failure with no enablement | the gating and discovery logic on the tested platform; not real-hardware discovery behavior |
| W08-DV02 | Supported presentation occurs (P6-V11 W08 share) | QEMU scenario (consumes W07 + W06 producer) | authorized pending event for a specified vCPU is loaded into an LR at entry and observable by the Guest as its pending interrupt | presentation occurs for the claimed event with correct vINTID and hw=0; descriptor mirrors hardware | the presentation bridge; not the lifecycle (W07) or Guest-side semantics (W10/W11) |
| W08-DV03 | vCPU-context preservation across transitions (P6-V11/V16 basis) | QEMU transition scenario | run vCPU with live presentation; exit through declared exit kinds; re-enter; compare context images and delivered state | no loss of pending/active outcomes across exits; images match purged state; repeated transitions stable | preservation across the declared exit set; not all exit kinds or migration |
| W08-DV04 | Safe pressure behavior (P6-V16) | QEMU over-capacity scenario + invariant review | generate more pending vIRQs than discovered LR capacity; observe fill, exit purge, and (with W09) maintenance-driven refill | no loss and no overwrite: excess remains pending in W07; loaded outcomes returned exactly once; refill progresses with freed slots | the D6 pressure policy in the declared environment; not production IRQ-pressure guarantees (W12/task book) |
| W08-DV05 | Basic priority carriage | host-side unit tests + QEMU observation | width-mapping tests (boundaries, order preservation at minimum/maximum widths); QEMU: two-pending case presents in W07 order | carriage is total, order-preserving, width-correct; presentation order matches selection order | carriage correctness; not priority semantics (W10) |
| W08-DV06 | Isolation and protocol conformance (P6-V18 share) | structural review + QEMU multi-vCPU case **only with** the evidenced prerequisite (else recorded stage block) | review running-pCPU-only access (D2); exit-assert of empty table; vCPU A unloaded state never observed by vCPU B; protocol misuse fixtures (stale claim, double clear) | no cross-vCPU state inheritance; misuse counted and force-consistent; isolation observed when exercisable | structural isolation; multi-vCPU evidence only with the prerequisite |
| W08-DV07 | Diagnostics and telemetry | review + counters observed in DV02–DV06 | discovery values, slot outcomes, pressure peaks, EOI-error counts present and correlated with scenarios | all [02] §7 counters populated by exercised paths | observability basis; not the latency baseline (W13) |
| W08-DV08 | Architecture/platform separation, unsafe boundary, P8 non-ABI | design-conformance review | inspect module graph (Core sees nothing); unsafe inventory matches the four modules; scan for machine-ABI/Guest-model leakage | register access confined and inventoried; no QEMU/platform constants in Core; no machine ABI named | scope conformance; not downstream correctness |

Planned, run, blocked, and failed are distinct evidence states recorded in
the verification record with command, input, environment, timestamp, and
reason. No W08 validation proves maintenance processing (P6-V17, W09),
Guest-visible masking/priority (W10), the vIRQ lifecycle (W07), the Linux
vGIC model (P8), or real-hardware behavior, and none may be reported as
doing so.

## 2. Error, security, and observability model

**Error model.** Named cases and guarantees:

- Readiness failures (`CapabilityMismatch`, `SysregInterfaceUnavailable`,
  `PhysicalNotReady`): presentation unavailable on the affected pCPU with
  an identifiable reason; no partial enablement; a platform W01 rejected
  never reaches enablement.
- Load shortfall (`NoFreeSlot`): normal pressure outcome — pending work
  stays in W07; never an error surfaced to producers.
- Sequence/invariant anomalies (descriptor/hardware mismatch, occupied
  slot at exit-assert, stale or duplicate claims): counted by kind, forced
  to the nearest consistent state (slot cleared, claim preserved as
  StillPending so no event is lost), surfaced as Host-attributed
  diagnostics — these cannot be caused by Guest input (Guests have no W08
  surface), so the ADR panic-policy distinction applies: containment plus
  visibility, never a Guest-triggered Host panic and never silent
  corruption.
- `Contradiction` outcomes at maintenance clear: counted, slot forced
  empty, the completion question left to W09's duplicate protection —
  never double-reported by W08.

**Security model.** Guests have no W08 access path; every LR field derives
from validated W07 claims and recorded configuration. Discovery data is
hardware fact validated against the W01 envelope before use. The `unsafe`
surface is the inventoried register accessors of the four logical modules;
the barrier/sequence tables ([03] §5, [04] §6) are part of the audited
boundary, and QEMU success never licenses omitting them.

**Observability model.** The counters and trace events of
[02](02-architecture-and-state.md) §7 are the designed surface, including
once-recorded discovery values; P6-W13 owns collection, correlation
(P6-V24), maintenance-frequency accounting, and the latency baseline.
Pressure peaks make the P6-V16 story quantitative without being a
performance claim.

## 3. Handoff checklist

Before handing W08 to a reviewer, provide:

- the exact changed-file list and the crate/module placement chosen for the
  logical modules;
- the W08 `unsafe` inventory delta (all register accessors) with `SAFETY`
  justifications, and the locked specification revision with the resolved
  register surface;
- W08-DV01–DV08 evidence paths and run status, including explicit not-run
  or blocked entries (multi-vCPU row without the prerequisite; W09-dependent
  refill rows without a W09 delivery);
- the discovered capacity/width values per exercised pCPU and the declared
  P6 VMCR defaults;
- confirmation that no hardware-mapped LR path, maintenance policy, Guest
  GIC MMIO model, machine ABI/DTB reference, or scheduler semantics was
  introduced, and that no Core-layer module contains register access;
- open items for consumers, without resolving their contracts here:
  - **P6-W09** (`../p6-w09-maintenance-interrupt/README.md`): the enabled
    maintenance sources, status read, clear, and refill primitives are the
    completed-presentation/reusable-capacity boundary; processing policy
    and duplicate protection on the W09 side remain W09's;
  - **P6-W10** (`../p6-w10-interrupt-semantics/README.md`): presentation
    order and priority carriage are the hardware-facing facts its
    semantics build on;
  - **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`):
    the Guest-observable presentation behavior (what a pending LR looks
    like across entry) is defined by DV02/DV03 conditions;
  - **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`): the
    pressure containment and forced-consistency rules are the surface to
    stress;
  - **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`):
    counters, discovery values, and trace identities are available;
    collection and baselines are W13's;
  - **P8**: evidenced lower-level capability and presentation facts only;
    the vGIC Distributor/Redistributor model, Guest DTB, and machine ABI
    remain P8-owned;
  - cross-design: the D8 maintenance boundary and the W07 protocol must
    stay reconciled with those packages' delivered designs; divergence is
    an Architecture Change Request, not a W08-local change.

No completion claim may be made anywhere in this design; completion
evidence belongs only in
`../../verification/p6-w08-gic-virtualization-interface-verification.md`.
