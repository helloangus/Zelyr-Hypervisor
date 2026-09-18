# P4-W08 Scenario Matrix

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W08 detailed design](README.md).  
**Companion:** assumptions and decisions in
[01-scope-and-foundations.md](01-scope-and-foundations.md); verdict rules in
[03-automation-contract.md](03-automation-contract.md).

## 1. Matrix conventions

- Matrix version `SM-T1`; a change that alters a row's expected observable is
  a joint review with the owning designs (W05/W06/W07) before merge, and the
  version bumps.
- Every row states: input/preconditions; expected observable (only
  versioned-marker and P4-RR evidence is accepted, D4); pass condition
  (objective, checkable by the verdict rules); and the proof boundary (what
  a pass proves and does not prove).
- Every on-target row also carries the EL2-liveness observation: after the
  Guest stops for any reason, the P4-RR record must continue to the run-end
  lines and the W06 `HV-DIAG` report must be present for fault scenarios.
  This is what turns "the Guest stopped" into "EL2 survived diagnosably."
- Scenario coverage versus the task book §6 mandatory/planned split: SM rows
  cover VG-001–VG-007, VG-010, VG-012 (mandatory). VG-008, VG-009, VG-011
  rows exist and are executed only if W05 delivers those scenarios;
  otherwise the deferral record is the evidence of record and the SM row is
  marked `NOT-RUN (deferred upstream)` — a determinate non-pass, never a
  silent skip.

## 2. Scenario rows

| ID | Scenario / VG ref | Input and preconditions | Expected observable | Pass condition | Proves / does not prove |
|---|---|---|---|---|---|
| SM-01 | Positive EL1 entry (VG-001) | declared build; VG-001 scenario; default probe plan | banner incl. protocol/layout versions and scenario id; `VG-001:BEGIN`, greeting incl. `Hello from EL1`, CurrentEL = EL1; `VG-001:OK`; P4-RR episode line: `Wfi` stop, `Controlled`, verdict `Match`, clean-completion count pattern (P4-W07 [04 §5](../p4-w07-repeatability-telemetry/04-code-contracts-telemetry.md)) | marker sequence exact; record pattern satisfied; EL2 liveness present; verdict `PASS` | Guest reaches EL1 via the real entry path and completes; not multi-vCPU, not scheduler, not hardware entry behavior |
| SM-02 | Unmapped-IPA translation fault (VG-004, probe class `UnmappedGap`, IS-01) | declared build; VG-004 with gap probe | `VG-004:BEGIN`, `VG-004:FAULT:LOAD-UNMAPPED`; P4-RR fault line: `Stage2Translation`, IPA = probe, agreement `Consistent`, `Match` | as SM-01 pattern plus exactly one fault line with the named class/IPA; `HV-DIAG` present; `PASS` | unmapped access faults, is correctly diagnosed, and EL2 survives; not that every unmapped address faults identically, not hardware fault semantics |
| SM-03 | Guest-RAM-boundary fault (VG-004, probe class `RamBoundary`, IS-02) | VG-004 with boundary probe | as SM-02 with IPA = RAM end page | as SM-02 with the boundary IPA predicate | Guest RAM bounds are Stage-2-enforced and diagnosable; not complete memory-model coverage |
| SM-04 | Hypervisor-owned-range fault (VG-004, probe class `HypervisorOwned`, IS-03) | VG-004 with hypervisor probe (membership pre-checked per W06) | as SM-02 with the hypervisor probe IPA | as SM-02 with the hypervisor probe predicate | Hypervisor-owned ranges are not guest-mapped; access is contained with EL2 live; not DMA/IOMMU isolation, not all protected ranges |
| SM-05 | Read-only write permission fault (VG-005, IS-04) | VG-005 | `VG-005:BEGIN`, `VG-005:FAULT:STORE-RO`; P4-RR fault line: `Stage2Permission`, access `Write`, mapping `MappedWith(RO)`, `Match` | as SM-02 with access `Write` | write enforcement is immediate and distinguishable; not all permission corner cases |
| SM-06 | XN execute permission fault (VG-006, IS-05) | VG-006 | `VG-006:BEGIN`, `VG-006:FAULT:EXEC-XN`; P4-RR fault line: `Stage2Permission`, access `Execute`, `Match` | as SM-02 with access `Execute`, distinguishable from SM-05 by the access field | execute and write permission faults are distinguishable; not instruction semantics |
| SM-07 | Controlled illegal execution (VG-010, IS-06) | VG-010 | `VG-010:BEGIN`, `VG-010:FAULT:ILLEGAL`; P4-RR fault line: `IllegalExecution`, `Match` | as SM-02 with the illegal class | illegal Guest behavior stays VM-facing with EL2 alive; not a full illegal-opcode taxonomy |
| SM-08 | Unknown synchronous exception (VG-011, IS-07) — planned | VG-011 if delivered | `VG-011:BEGIN`, `VG-011:FAULT:SYNC-UNKNOWN`; P4-RR fault line: `UnknownSync` with raw ESR retained, `Match` per delivered routing | as SM-02; if W04 routing yields a different class, the row fails and routes to joint W04/W05/W06 review (never re-labeled here) | unknown syncs are retained raw and contained; not that all unknown classes are enumerable |
| SM-09 | WFI defined result (VG-007, IS-08) | VG-007 | `VG-007:BEGIN`; P4-RR: `Wfi` exit, `Controlled` stop, clean pattern | marker + record agreement; `PASS` | WFI has a defined diagnosable result; not timer/wakeup behavior (P6) |
| SM-10 | WFE defined result (VG-008, IS-08) — planned | VG-008 if delivered | `VG-008:BEGIN`; P4-RR: `Wfe` exit, `Controlled` stop | as SM-09 | as SM-09 for WFE |
| SM-11 | Fault-and-reenter sequence (VG-009) — planned | VG-009 if delivered | two episode records with distinct triggers; ≥ 1 `vcpu.reenter`; both stops determinate; context preserved between episodes (W04 DV04 basis) | both episodes `Match`; reenter count ≥ 1; `PASS` | re-entry preserves Guest context across a classified exit; not general rescheduling |
| SM-12 | Completion and controlled stop (VG-012) | VG-012 as the repeat plan's standard episode | banner; `VG-012:OK`; `Wfi` stop; teardown clean; repeat-ready | as SM-01 plus teardown-clean record fields | stop/teardown protocol works repeatedly; not lifecycle policy (P7+) |

Coverage note: SM-01–SM-12 realize P4-V13 (SM-01 positive), P4-V14 (SM-02
through SM-08 fault/survival), and P4-V09 support jointly with W04/W05/W06;
the task book's mandatory set is covered by SM-01–SM-07, SM-09, SM-12 plus
the repeat set.

## 3. Repeat sets (P4-V10/V11/V15 realization)

| Set | Composition | Repetition rule | Stability condition |
|---|---|---|---|
| RS-A same-session | one declared build; ≥ 3 episodes per W07 minimums (episode 0 `FullRebuild` reference, 1 `FullRebuild`, 1 `RamReuse`), scenarios fixed by the manifest (default: SM-12, SM-12, SM-12) | single QEMU boot; episodes sequential inside it | all episode verdicts `PASS`; P4-RR `P4RR:DET Stable`; accounting fields clean |
| RS-B fault mix | one boot cycling one fault scenario set (default: SM-02, SM-05, SM-06, SM-07 in fixed order, one episode each) | single QEMU boot per iteration | every episode `Match` with its named class; EL2 liveness after every stop |
| RS-C cold boot | declared build; one boot per iteration; default scenario SM-12; manifest may add SM-01 | ≥ 5 iterations (W07 minimum); fresh QEMU process each; no state carried between boots | per-boot P4-RR records equal on S1–S5 surfaces (W07 D3) and all verdicts `PASS`; comparison is per declared build identity (W07 open item O2) |
| RS-D regression set | SM-01–SM-07, SM-09, SM-12 executed once each (planned rows appended when delivered) | the declared P4-V13/V14 set in one pass | set verdict `PASS` per D7 |

Rules common to all sets:

- Iterations are independent: no artifact, file, or emulator state is reused
  between iterations beyond the declared build and manifest inputs (D5).
- Any `TIMEOUT`/`INCOMPLETE`/`UNSUPPORTED` in a set makes the set verdict
  non-`PASS` with the causing iteration named.
- A failed expectation is a defect report against the owning package's
  design (W05/W06/W07/W02/W03/W04), never a manifest relaxation (D2).

## 4. Explicit non-goals of the matrix

A passing matrix does not prove: real-hardware behavior (ADR-003; task book
acceptance wording), absence of all isolation gaps (only the probed ranges),
timer/IRQ/scheduler behavior (P6/P7), multi-VM or multi-vCPU behavior,
performance of any path, or QEMU-version independence (the declared
environment is part of the evidence identity, open item O2).
