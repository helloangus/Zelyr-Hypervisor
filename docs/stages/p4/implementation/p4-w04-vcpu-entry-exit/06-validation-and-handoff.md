# P4-W04 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).

## 1. Validation matrix

World-switch behavior is hardware-executed; the meaningful evidence is
on-target, with host-side tests covering pure constructions and
classification tables. Guest-side scenarios come from
[P4-W05](../p4-w05-validation-guest/README.md); automation from
[P4-W08](../p4-w08-qemu-integration-regression/README.md); classification
detail from [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md). The
matrix defines what W04's evidence must show.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W04-DV01 | P4-C01/C03 initial state | construction unit tests + review | determinism (same inputs → identical context), rejection suite | state is a pure function; invalid inputs rejected; EL1h/DAIF-masked enforced | construction correctness; not on-target entry |
| W04-DV02 | P4-C02/C04 Guest EL1 via real ERET | on-target entry evidence (W05 VG-001 class) | boot → entry → Guest marker + CurrentEL report | Guest code executes at EL1 and reports EL1; reached by ERET (path review) | first entry works; not multi-vCPU or scheduler behavior |
| W04-DV03 | P4-E03 EL2 recovery | on-target controlled-exit evidence | fault-scenario exit (VG-004 class) → Rust handler runs | after every exit, EL2 state is functional (console/allocator/logging); exit frame complete | recovery boundary works; not fault-diagnosis completeness (W06) |
| W04-DV04 | P4-E04 re-entry | on-target re-entry scenario (W05 planned re-entry class) | exit → context save → re-enter → Guest continues | at least one re-entry resumes correctly with validated captured state | re-entry path works; not general rescheduling |
| W04-DV05 | P4-E05 defined stop | on-target stop evidence (VG-007 class WFI or controlled stop) | stop action → Stopped(cause) → teardown | stop is terminal, cause recorded, diagnostics retained, teardown restores ownership | defined stop; not full VM lifecycle |
| W04-DV06 → P4-V09 | WFI/WFE defined result; Guest fault contained | on-target scenarios via W05/W08 | WFI trap → classified exit + stop; illegal-execution scenario → GuestFault stop, Host alive | both receive defined, distinguishable, VM-facing results; no hypervisor panic | containment and defined results for these classes; not vGIC/timer delivery (P6) |
| W04-DV07 | Containment: no EL2-flow continuation | security review + injection | review ERET target construction; inject Guest attempts (EL1 sysreg writes, HVC attempt) | Guest modifications cannot produce EL2 execution; EL1 baseline restore effective across exits | mode containment by construction; not complete side-channel analysis |
| W04-DV08 | Exit-budget guardrail | host + on-target test | force repeated re-entries (planned re-entry scenario loop) | budget exhaustion → `Stop(BudgetExhausted)`; run remains diagnosable | runaway guard works; not scheduling fairness |
| W04-DV09 | Frame fidelity | exit-handler assertions + W06 cross-check | captured PC/syndrome vs scenario-expected values | frame fields match the Guest scenario's expected fault site and syndrome class | capture correctness; not full syndrome semantics (W06 owns detail) |
| W04-DV10 | unsafe and layering review | static review | audit stubs/helpers vs [01 D9](01-scope-and-foundations.md) and [02 §5](02-architecture-and-state.md) ownership table | unsafe surface = inventory; every SAFETY note cites ownership rows; no Core-visible register names | controlled-unsafe compliance; not functional correctness |
| W04-DV11 | Repeat readiness | same-session restart (with W07) | stop → reinitialize → run again | clean second run without residue dependence | teardown/reinit soundness; not performance |

Record each as **passed / failed / blocked / not run** with command or review
input, environment, date, and reason. QEMU rows prove the stated reference
environment only, not real-hardware semantics (W01 A7). Rows are plans until
the verification record exists.

## 2. Error model

Two disjoint failure classes, enforced by type separation
([04 §5](04-code-contracts-vcpu-run.md)):

- **Guest-caused outcomes** (faults, traps, budget exhaustion): values
  (`ExitInfo`, `StopCause`), always contained, always diagnosable, never
  host-fatal (W01 A2; ADR §19). Unknown conditions fail closed to
  `UnknownSync` + stop.
- **Host invariant violations** (double entry, wrong-state teardown, capture
  corruption, interrupt-mask mismatch): fatal escalation through the P0
  failure-classification path with non-recursive diagnostics (P1-W07
  contract, M3).

Setup-class errors (`InvalidScenario`, range errors) are recoverable VM-facing
construction failures that stop Guest creation without Host disturbance.

## 3. Security model

- The world-switch is the package's security boundary: Guest-owned registers
  are captured or reset ([02 §5](02-architecture-and-state.md)); the Guest
  has no architectural route to EL2 state; ERET targets are host-constructed.
- EL1 baseline restore (D5) bounds what an untrusted Guest can persist; the
  limitation (no EL1-state persistence across exits) is a recorded P4 fact,
  not a hidden simplification.
- Interrupt masking (D4) removes the re-entrancy class from P4; it is a
  recorded limitation with the P6 Reserved re-entry, not a silent shortcut.
- `unsafe` surface: two naked stubs, their host save/restore sequences, and
  sysreg helpers — each with ownership-citing SAFETY notes; growth beyond
  the inventory is a review failure.
- Standing scope boundaries: no HVC convention (P5), no virtual interrupts
  (P6), no scheduling states (P7) — any pressure to add them here is a stage
  boundary violation to record.

## 4. Observability model

Events (`vcpu.construct`, `vcpu.enter`, `vcpu.exit` with class + Guest PC,
`vcpu.reenter`, `vcpu.stop` with cause, `vcpu.destroy`) route through the P0
logging/trace baseline (W01 A8). The preserved exit frame is the
post-stop diagnostic source (EL2 stays "live and diagnosable" per the task
book). W06 consumes frames for diagnosis; W07 counts transitions; W08
matches markers. No event carries Guest data beyond PC/IPA-class addresses
required for diagnosis.

## 5. Handoff checklist

Before handing W04 work to a reviewer:

- exact changed-file list and implementation-record path
  (`../p4-w04-vcpu-entry-exit-record.md`);
- DV01–DV11 statuses with explicit not-run/blocked entries and the upstream
  rows (W01 R01–R05, R13–R16) each blocked item waits on;
- new `unsafe` list with SAFETY note locations, inventory delta, and the
  generated-code review evidence for the handoff windows;
- factual notes for [P4-W09](../p4-w09-closeout-p5-handoff/README.md): entry
  protocol, EL1-baseline limitation, interrupt-masking limitation, P4 exit
  classes (temporary), budget constant;
- handoff to consumers: execution path and entry convention to
  [P4-W05](../p4-w05-validation-guest/README.md); exit frame and class
  vocabulary to [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md);
  transitions and stop path to
  [P4-W07](../p4-w07-repeatability-telemetry/README.md); markers to
  [P4-W08](../p4-w08-qemu-integration-regression/README.md);
- open items: any Specification Investigation records (trap encodings),
  upstream mismatches (M3/M5/M7/M8) and their resolution notes, P2-ACR-01
  unchanged.
