# P4-W04 Single-vCPU Guest-EL1 Entry and Recovery — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** One reproducible Guest-EL1 vCPU: initial state, real `ERET` entry,
exit capture with EL2 recovery, re-entry, and a defined stop result, as
required by [P4-W04](../../plans/p4-w04-vcpu-entry-exit.md) (P4-C01–C04,
P4-E01, E03–E05).  
**Owner/change context:** P4-W04 implementation handoff; this design owns the
world-switch boundary, the register-ownership contract, the P4 vCPU run
states, and the P4-local exit-class vocabulary consumed by W06.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W04. It converts the bounded
work-package plan into the vCPU object and context model, the world-switch
frame and register-ownership contract, the entry/exit pseudocode, and the run/
stop workflow, with pseudocode rather than production code. It deliberately
does **not** design the Stage-2 mapper or its activation internals
([P4-W02](../p4-w02-stage2-address-space/README.md)), Guest RAM construction
([P4-W03](../p4-w03-guest-memory-image/README.md)), the Validation Guest's
scenario contents ([P4-W05](../p4-w05-validation-guest/README.md)), full exit
classification and isolation diagnostics
([P4-W06](../p4-w06-fault-isolation-diagnostics/README.md) consumes this
design's exit frame), repeatability/telemetry policy
([P4-W07](../p4-w07-repeatability-telemetry/README.md)), the final generic
`ExitReason` software API, the HVC ABI, Guest SMP, scheduler semantics,
vGIC/timer, or formal VM/vCPU crate APIs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — ledger,
  assumed upstream contracts with failure boundaries, scope classification,
  and resolved decisions (entry protocol, interrupt policy, EL1 baseline
  restore, exit budget) with authority. Load first.
- [02-architecture-and-state.md](02-architecture-and-state.md) — logical
  modules, core objects (vCPU, context, per-pCPU world-switch state), the
  vCPU state machine, the concurrency model, and the register-ownership
  contract. Load for architecture and lifecycle work.
- [03-code-contracts-world-switch.md](03-code-contracts-world-switch.md) —
  the assembly boundary contracts: frame layout, guest-entry and guest-exit
  stubs, host-state preservation, `unsafe`/SAFETY requirements. Load for the
  low-level work area.
- [04-code-contracts-vcpu-run.md](04-code-contracts-vcpu-run.md) — the
  Rust-side contracts: vCPU construction, run loop, exit-frame interpretation,
  exit actions (re-enter/stop), stop and teardown. Load for the control-flow
  work area.
- [05-implementation-workflow.md](05-implementation-workflow.md) — ordered
  implementation steps.
- [06-validation-and-handoff.md](06-validation-and-handoff.md) — validation
  matrix (P4-V04, P4-V09 inputs), error/security/observability model, handoff
  checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W04 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W04 plan](../../plans/p4-w04-vcpu-entry-exit.md) → this design → Coding
Guidelines. Binding constraints include:

- ADR §5: the architecture backend owns register save/restore, virtualization
  controls, and Guest entry/exit; Core sees a standard exit classification.
  ADR-022: the Guest runs at EL1; no virtual EL2 is exposed.
- ADR-007/ADR §19: Guest execution cannot continue as ordinary EL2 flow;
  Guest-caused faults are contained, VM-facing events.
- ADR-015/task book: no permanent single-pCPU or single-VM architecture may
  be asserted; P4 binds one vCPU to one pCPU for its test scope while keeping
  pCPU and vCPU separate objects (W01 A3).
- Task book §1 Out of scope for W04: final generic exit-reason software
  design, HVC ABI, Guest SMP, scheduler, vGIC/timer, VM lifecycle policy,
  formal VM/vCPU API boundaries. Task book §1 Reserved: WFI/WFE and unknown
  exceptions get basic handling; ERET must really enter EL1.

Classification: the world-switch boundary, register-ownership contract,
vCPU/context objects, run loop, and stop path ([02](02-architecture-and-state.md),
[03](03-code-contracts-world-switch.md), [04](04-code-contracts-vcpu-run.md))
are **Required** for P4-C01–C04 and E01/E03–E05. The vCPU run-state machine is
a documented P4 subset of the ADR vCPU lifecycle (full machine is P7).
Interruptible Guest execution, virtual-interrupt injection, blocked/paused
states, and M:N scheduling are **Reserved**. Multi-vCPU, scheduler policy,
vGIC/timer virtualization, HVC ABI, management lifecycle, and performance work
are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-C01 initial Guest PC/SP/processor state | [contracts](04-code-contracts-vcpu-run.md) §2.1; [foundations](01-scope-and-foundations.md) D1 | P4-V04 (correct Guest state reaches EL1) |
| P4-C02 Guest EL1, not virtual EL2 | [architecture](02-architecture-and-state.md) §4; [foundations](01-scope-and-foundations.md) D3 | P4-V04 (CurrentEL evidence via W05 scenario) |
| P4-C03 repeatable construction | [contracts](04-code-contracts-vcpu-run.md) §2.1 (pure function) | P4-V04; repeatability consumption by P4-W07 |
| P4-C04 real `ERET` into Guest | [contracts](03-code-contracts-world-switch.md) §3 | P4-V04 |
| P4-E01 first entry | [workflow](05-implementation-workflow.md) step 7 | P4-V04 |
| P4-E03 EL2 recovery on exit | [contracts](03-code-contracts-world-switch.md) §4; [architecture](02-architecture-and-state.md) §5 | P4-V04 (classified exit retains EL2 control) |
| P4-E04 re-entry | [contracts](04-code-contracts-vcpu-run.md) §3.3 | P4-V04 (at least one re-entry) |
| P4-E05 defined stop | [contracts](04-code-contracts-vcpu-run.md) §4 | P4-V04; P4-V09 stop evidence |
| WFI/WFE + controlled Guest fault diagnosable | [contracts](04-code-contracts-vcpu-run.md) §3.2 (exit classes); W06 consumes | P4-V09 (joint with W06/W05) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs` at
`4e631ee`): documentation-only repository — no EL2 runtime, no vectors, no
per-CPU state, no Stage-2, no vCPU code. P1–P3 are planned only (W01 rows
R01–R05, R13–R16). Everything below is an assumed contract with a recorded
failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Correct Guest PC/SP/state reaches EL1 by `ERET` | No EL2 runtime exists (P1 planned) | Stable EL2 with working vectors and a known EL2 baseline (assumed M4/M5) plus W04's entry state | `ERET` needs a live EL2 to return from and a vector path when the Guest exits | P1-W02/W04/W05 plans (assumed); W04 owns entry state | P4-V04 on-target |
| A classified exit retains EL2 control | No exception path exists | Guest-exit vector stub + host-context restore (assumed M4) and W04's exit frame | "Recovery" means EL2 state (stack, clobbered registers) is restored before Rust runs | P1-W05 (assumed); W04 owns the stub | P4-V04; fault rows via W06 |
| At least one re-entry and a defined stop path | Nothing exists | Saved guest context + resume entry + stop state ([04 §3–§4](04-code-contracts-vcpu-run.md)) | Re-entry is a resume from saved context; stop is a defined terminal transition | W04 | P4-V04, P4-V09 |
| Guest execution cannot continue as ordinary EL2 flow | Nothing exists | PSTATE/mode discipline in entry construction (EL1h) + exit always via exception to EL2 ([01 D3](01-scope-and-foundations.md)) | A Guest that "returned" into EL2 flow would be a privilege escape, not an exit | W04; ADR-022 | P4-V04 review + runtime evidence |
| No permanent single-pCPU/single-VM assumption | Nothing exists | Per-pCPU world-switch state discovered via the P3 CPU-local mechanism; object-shaped vCPU (assumed M6) | binding one vCPU to one pCPU is P4 policy, not a structural fact | P3-W04/W14 (assumed); W04 shapes its objects | design review |
| WFI/WFE yields a defined diagnosable result | Nothing exists | WFI/WFE trap routing intent + exit classes `Wfi`/`Wfe` with re-enter/stop actions ([04 §3.2](04-code-contracts-vcpu-run.md)) | the task book requires a defined result, not silent continuation | W04; W06 classifies details | P4-V09 via W05/W08 |

No row requires a decision outside this design's authority; entry protocol,
interrupt policy, EL1-baseline restore strategy, and the exit budget are
Implementation Choices assigned to detailed design (task book §8: "assembly/
Rust boundary", "Stage-2 page-table representation ... and assembly boundary
remain detailed-design choices").

## Resolved design decisions and their authority

Summarized; full rationale in [01 §4](01-scope-and-foundations.md):

1. **Initial state is a pure construction** from W03's `GuestInput` plus a
   validated scenario id; no firmware residue is consulted. P4-C03; W01 A2.
2. **Entry protocol:** validate → (activate Stage-2 via W02) → restore EL1
   baseline → load guest context → mask-and-save host PSTATE → `ERET`.
   ADR §5; ADR-022.
3. **Mode discipline:** the Guest is constructed at EL1h with DAIF masked;
   it can never reach EL2 execution state because EL2 return happens only
   through the exception entry, which overwrites ELR/SPSR with EL2-reserved
   copies. ADR-022; ADR §19.
4. **Interrupt policy for P4:** host interrupts are masked for the whole
   Guest run segment (entry to exit); WFI/WFE trap to EL2 via HCR routing
   intent. Recorded limitation: no interruptible Guest until P6. Task book
   (vGIC/timer out of scope); ADR §7 defers delivery.
5. **EL1 state strategy:** restore a fixed EL1 architectural baseline on
   every entry (from the P1 EL1/EL0-preparation baseline) instead of full
   EL1 sysreg save/restore; Guest writes to EL1 system state do not persist
   across exits. Safe-by-construction containment; documented P4 test
   limitation. ADR §5; P1-W04 contract.
6. **Exit capture:** one Guest-exit stub saves guest GP registers plus
   ELR/SPSR/ESR-class syndrome into the per-pCPU exit frame, restores host
   state, and calls the Rust exit handler; classification detail belongs to
   W06 but the frame carries what W06 needs (W01 row alignment).
7. **Exit budget:** a bounded consecutive-exit counter guards against
   re-entry storms; exhaustion stops the vCPU with a defined cause. P4 test
   guardrail; task book E05.
8. **`unsafe` shape:** two naked-function boundaries (guest entry, guest
   exit) plus a host-context save/restore block, each with structured SAFETY
   justifications and inventory entries. ADR-006; Coding Guidelines.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): ledger,
   assumed contracts (M-series), decisions D1–D8.
2. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module map, objects, state machine, concurrency model, and the
   register-ownership contract that both code-contract files rely on.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md),
   loading [03](03-code-contracts-world-switch.md) for the assembly boundary
   and [04](04-code-contracts-vcpu-run.md) for the run control.
4. Record validation in
   `../../verification/p4-w04-vcpu-entry-exit-verification.md` and facts in
   `../p4-w04-vcpu-entry-exit-record.md` only when work starts; no completion
   claims.

## Explicitly excluded interfaces

Not designed or authorized by W04: the final generic `ExitReason` API and its
full taxonomy (P4-W06/W07 evolve the P4-local classes; the ADR-level API
arrives with the Arch exit contract), any HVC/hypercall convention or
management ABI (P5), vGIC/timer state handling beyond "host-owned baseline"
(P6), scheduler-visible runnable/blocked states (P7), Guest SMP and multi-vCPU
execution (P6+/P7+), PSCI virtualization (P8), and any QEMU-conditional Core
behavior. The P4 exit classes are temporary vocabulary, recorded as
implemented facts by W09, never a frozen ABI.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W05** receives the real Guest-EL1 execution path: entry convention
  (entry IPA, stack top, scenario id in the initial register), the
  WFI/WFE trap behavior its scenarios rely on, and the exit classes its
  fault scenarios produce.
- **P4-W06** receives the exit frame (syndrome, Guest PC, faulting IPA
  fields as captured) and the P4-local exit-class vocabulary; it owns
  classification detail, diagnostics, and the Guest-vs-hypervisor fault
  boundary on top.
- **P4-W07** receives the run-state transitions and stop path as the repeat
  surface (same-session restart and stop evidence) and the enter/exit event
  points.
- **P4-W08** receives stable entry/exit/stop markers for automation.
- **P4-W09** records only implemented facts (entry protocol, EL1-baseline
  limitation, interrupt masking limitation, exit-class list).
- **P5** inherits the evidenced EL2→EL1→EL2 boundary only; it owns all
  hypercall/capability semantics on top of it.
