# P8-W16 Scenario Matrix and Marker Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W16 detailed design](README.md).

## 1. Logical artifact groups

W16 is a validation design, so its logical modules are authoritative artifact
groups, not code modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Scenario matrix (this file §3–§5) | W16 (this design) | W09–W15 outcome contracts, W19 retained-suite entry points, task book P8-V21/V22 wording | the declared row set, pass conditions, and proof boundaries; it does not own scenario *content* vocabularies or fixture recipes |
| Marker-oracle contract (this file §2) | W16 (contract shape); marker vocabularies remain owned by their source packages | W09/W10/W11/W12/W13/W14/W19 outcome wording | how markers are matched; it does not invent new Guest-visible semantics |
| Harness contract, run record, repeated-boot and failure rules | [02-harness-contract-and-workflow.md](02-harness-contract-and-workflow.md) | this matrix | execution and evidence rules; it does not judge performance or redefine isolation |
| Verification record and raw evidence | `../../verification/p8-w16-automated-linux-regression-verification.md` and `../../verification/assets/p8-w16/` (created when evidence exists) | actual runs | run/not-run evidence only; never part of this design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Marker-oracle contract

Every scenario row's observable is a sequence of Guest-console text markers
plus a terminal outcome. The contract *shape* (owned here) is:

- **Marker form:** each expected observable is declared as an ordered list of
  exact match expressions (`EXACT` or `REGEXP` class) evaluated against the
  captured serial transcript. A marker expression may be declared `ONCE` or
  `REPEAT(n)` (e.g., per-CPU events in SMP rows).
- **Ordering rule:** markers must appear in the declared order; interleaved
  unrelated console output is permitted; an out-of-order expected marker fails
  the row unless the row declares an unordered match class explicitly.
- **Terminal outcome enum:** exactly one of
  `CLEAN_SHUTDOWN` (Guest-initiated, via the approved PSCI system-off/lifecycle
  path of [P8-W06](../p8-w06-psci-virtualization/README.md)),
  `GUEST_FAULT_CONTAINED` (declared Guest-scoped failure with Hypervisor
  diagnostic, per [P8-W13](../p8-w13-guest-fault-diagnostics/README.md)
  classes), or `HARNESS_TIMEOUT` (always a failure). Any outcome outside the
  row's declaration fails the row.
- **Fatal-class exclusion:** a pass requires that no diagnostics classified
  fatal or Hypervisor-invariant by the W13 ledger appear. An unexpected fault
  class fails the row even if all declared markers matched.
- **No host leakage:** markers and diagnostics must not contain Host physical
  addresses, Host IRQ numbers, board/SoC identities, or Host firmware facts;
  their appearance fails the row (aligns with the W04 host-leak rule and the
  W14 no-QEMU-as-ABI rule).

This shape is deliberately vocabulary-free: W16 never fixes the concrete text
of a Linux boot banner or a Validation Guest completion string. Those are
owned by the source packages; until an owner package evidences its vocabulary,
the corresponding rows are **blocked prerequisites**, not authorable content.

## 3. Row families

Row IDs are stable identifiers for plans and evidence. `RAM class` consumes
the W12 small/normal/larger classes; concrete capacities are not fixed by W16.
`Fixture` consumes the W15 reproducible fixture. `Track VG` denotes the
Validation Guest retained suite; `Track LG` denotes the Linux track.

| Family | Rows | Purpose | Content owner |
|---|---|---|---|
| LG-BOOT (Linux single-CPU boot) | LG-B1 | one-vCPU Linux to interactive initramfs userspace with retained markers; explicitly not `start_kernel`-only ([P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md)) | W09 markers; W15 fixture; W03 boot contract |
| LG-SMP-BOOT (multi-CPU boot) | LG-B2, LG-B4 | 2- and 4-vCPU enumeration, PSCI secondary start, per-CPU timer/IRQ/SGI/scheduler/idle observables ([P8-W10](../p8-w10-linux-smp-bringup/README.md)) | W10; W06 PSCI boundary; W04 DTB |
| LG-SMP-STAB (SMP stability) | LG-S2, LG-S4 | declared busy/thread/sleep/affinity/interrupt/scheduler workloads; loss-of-event/race/corruption oracle ([P8-W10](../p8-w10-linux-smp-bringup/README.md)) | W10 scenario set |
| LG-SCHED (scheduler integration) | LG-SC1 (1:1), LG-SC2 (declared M:N) | Linux progress with preserved timer/interrupt/WFI/accounting semantics; no fairness claim ([P8-W11](../p8-w11-scheduler-linux-integration/README.md)) | W11; P7 evidenced scheduler facts |
| LG-MEM (memory model) | LG-M1, LG-M2, LG-M4 (same Linux row per vCPU count across RAM classes) | RAM-class execution honoring reserved/MMIO boundaries; boundary-fault rows retain actionable Stage-2 diagnostics ([P8-W12](../p8-w12-linux-memory-model/README.md)) | W12 classes; approved machine values |
| LG-FAULT (intentional fault) | LG-F1 | one representative declared Guest fault producing a contained VM-scoped diagnostic; harness must continue afterwards | W13 fault classes; W18 expected results |
| LG-COMPAT (ABI drift) | LG-C1 | the W14 drift-detection assertions against the approved machine contract facts, executed on the same approved configuration | [P8-W14](../p8-w14-machine-abi-compatibility/README.md) |
| VG (Validation Guest retained suite) | VG-RET (entry-point set owned by W19) | mechanism-level suite (HVC, Stage-2, timer, SGI/vIRQ, MMIO, SMP, scheduler interaction) executed under the W16 envelope | [P8-W19](../p8-w19-validation-guest-dual-track/README.md) and the P4–P7 suite designs |
| REP (repeated boot) | REP-B1, REP-B4 | repeated consecutive boots with cross-run comparison ([02](02-harness-contract-and-workflow.md) §5) | W16; diagnostic classes from W13 |

The task book's P8-V21 wording requires that every scheduled matrix run
include at least: one Validation Guest row, Linux at 1, 2, and 4 vCPU, more
than one RAM class, and at least one intentional-fault row. The
**full matrix** is the complete row set of §4; a scheduled run may declare a
**smoke subset** (a named, versioned subset recorded in the run record), but a
subset that omits any P8-V21-mandated element is not a valid matrix run for
P8-V21 closure.

## 4. Full scenario matrix

Repetition/duration rules per row: `rep` is the declared repetition count
(recorded per run; the REP family's minimum is defined in
[02](02-harness-contract-and-workflow.md) §5), `seed` is the seed discipline
(`none` by default), `timeout` is the per-repetition failure timeout (declared
run parameter, always recorded; timeout ⇒ row failure with retained log).
Durations are never pass conditions.

| Row | Track / config | Input and precondition | Expected ordered observables (content owner) | Terminal outcome | Pass condition | Proves | Does not prove |
|---|---|---|---|---|---|---|---|
| LG-B1 | LG, 1 vCPU, normal RAM | approved fixture v<fd>, approved machine config, clean start | W09 boot/console milestone sequence through an interactive-shell marker and an executed shell command | CLEAN_SHUTDOWN | markers in order; no fatal-class output; command echo observed | the automated single-CPU path to interactive userspace is determinately observable | not that boot time is acceptable, not hardware behavior, not that all kernel paths work |
| LG-B2 / LG-B4 | LG, 2 / 4 vCPU, normal RAM | as LG-B1, SMP-enabled DTB per approved topology | W10 enumeration and secondary-start sequence with per-CPU timer/IRQ/SGI/scheduler/idle observables (`REPEAT` per vCPU) | CLEAN_SHUTDOWN | all per-CPU observables present for every declared vCPU; counts exact | PSCI-based SMP start is automatable and determinate in QEMU | not Host-SMP behavior, not real-board topology, not scheduler fairness |
| LG-S2 / LG-S4 | LG, 2 / 4 vCPU, normal RAM | W10 declared workload profile active | workload-start and workload-summary markers; completion marker; no loss-of-event/race/corruption class in W13 ledger | CLEAN_SHUTDOWN | completion observed; fault-class ledger empty of declared anomaly classes | declared stability workloads complete without the declared anomaly classes | not absence of all races, not real-time behavior, not performance |
| LG-SC1 / LG-SC2 | LG, vCPU count > available scheduling context per W11 declaration | W11 declared 1:1 and M:N scenario | progress markers per scheduled entity; scheduler accounting observable present (W11/P7 telemetry) | CLEAN_SHUTDOWN | all declared entities make declared progress; accounting present | Linux progresses under the declared scheduling configuration with preserved semantics | no final fairness claim; no scheduler-policy judgment (ADR-057 untouched) |
| LG-M1/M2/M4 | LG, 1/2/4 vCPU, each W12 RAM class (small and larger at least once) | approved machine values for the class | LG-B-family markers under the class config; boundary rows add W12 boundary observables | CLEAN_SHUTDOWN (normal rows); GUEST_FAULT_CONTAINED with actionable Stage-2 diagnostic (boundary-fault rows) | per-row declaration honored | approved RAM classes execute with reserved/MMIO boundaries honored | not Stage-2 redesign soundness, not overcommit/advanced memory behavior |
| LG-F1 | LG, 1 vCPU, normal RAM | W13-declared fault injection enabled in fixture | fault trigger marker; contained VM-scoped diagnostic per W13 minimum context; harness-continuation marker | GUEST_FAULT_CONTAINED | diagnostic context complete per W13; no EL2 fatal output; subsequent harness steps run | an intentional Guest fault is contained and diagnosable end-to-end | not complete crash analysis; not that every fault class is covered (W18 owns the full set) |
| LG-C1 | LG, approved v1 configuration | W14 drift-detection fixture/assertions for approved machine facts | W14 assertion transcript: Guest-visible map, device location, IRQ assignment, DT compatible, topology, PSCI, timer, console, machine identity | CLEAN_SHUTDOWN | all W14 assertions pass on the approved configuration | the automated route detects Guest-visible drift | not that values are correct (only that they match the approved contract); not a machine-ABI freeze |
| VG-RET | Track VG, per W19 entry set | VG fixture per P4–P7 suite designs | retained-suite markers owned by W19/P4–P7 designs | per W19 row declarations | per W19 rows under this envelope | the mechanism suite remains executable alongside Linux | not OS integration; VG pass never substitutes for Linux coverage or vice versa |
| REP-B1 / REP-B4 | LG, 1 / 4 vCPU, normal RAM | consecutive repetitions of the boot row in one harness session, no host restart between | each repetition matches the boot row's oracle; cross-run comparison per [02](02-harness-contract-and-workflow.md) §5 shows no new fault class and no first-boot-only symptom | CLEAN_SHUTDOWN on every repetition | all repetitions pass oracle; comparison set empty | repeated lifecycle does not surface stale VMID/TLB/vCPU/IRQ state or uninitialized-state symptoms in the declared checks | not proof that no race exists anywhere; not hardware repeat behavior |

## 5. Determinism, repetition, and seed rules

1. **Determinism rules.** Oracles depend only on marker order, marker
   multiplicity, terminal outcome, and fault-class ledger contents. They must
   not depend on wall-clock values, marker byte offsets, or console framing.
   A row whose expected observable cannot be expressed under these classes
   must be redesigned at review, not weakened into a substring guess.
2. **Repetition rules.** Boot-family rows declare a repetition count per run;
   the count is recorded in every run record. The REP family requires at
   least the repetition count declared for it in the run record, and a
   single-repetition REP run is invalid for P8-V22 closure. Repetition must
   never be used to retry a failing row into passing (see
   [02](02-harness-contract-and-workflow.md) §6).
3. **Seed rules.** Rows default to `seed: none`. A row needing randomness
   (none is currently declared) must name its seed source, fix the seed per
   repetition, and record all seeds in the run record; an unfixed seed makes
   the row non-determinate and ineligible for P8-V21/V22 closure.
4. **Duration rules.** Each row declares a timeout class (`generous` boot,
   `bounded` workload, per W10/W11 workload profiles) with the concrete value
   recorded per run. Timeouts are failure oracles with retained transcripts;
   they are never tuned to make a borderline run pass and never reported as
   performance measurements (that is
   [W17](../p8-w17-linux-performance-baseline/README.md) scope, with its own
   method).
5. **Environment rule.** Every run records the environment identity required
   by [02](02-harness-contract-and-workflow.md) §3 so that a later reader can
   distinguish "the matrix passed" from "the matrix passed in this declared
   environment". A QEMU/TCG pass proves the declared scenario in the declared
   environment only; it never proves AArch64 hardware semantics
   (ADR-003 boundary).
