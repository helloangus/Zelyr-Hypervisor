# P8-W19 Dual-Track Scenario Matrix

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W19 detailed design](README.md).

## 1. Logical artifact groups

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Retained-scenario inventory (this file §3) | Source suites own scenario content and observables (assumed contracts); W19 owns the consolidated inventory and applicability columns | P4-W05, P5-W07, P6-W11, P7-W10 designs and records; W01 reconciliation | the retained set; it does not restate or modify observables |
| Track-separation mapping (this file §4) | W19 (mapping); W09–W12 own Linux check content | ADR-008/009; P8 task book P8-V25 | which track proves what; it does not author Linux checks |
| Coexistence, maintenance, block rules, workflow | [02-coexistence-and-workflow.md](02-coexistence-and-workflow.md) | W16 envelope; this file | execution and retention rules |

## 2. Mechanism set

The plan names the mechanism areas P8 must retain mechanism-level coverage
for. Each maps to source suites:

| Mechanism area | Source suite(s) |
|---|---|
| HVC / hypercall ABI and controlled exit | P4-W05 (`p4-w05-validation-guest`); P5-W07 (`p5-w07-validation-guest-isolation-suite`) |
| Stage-2 fault (unmapped/permission) | P4-W05 |
| Timer | P6-W11 (`p6-w11-validation-guest-interrupt-suite`) |
| SGI / vIRQ / masking / maintenance | P6-W11; P5-W07 (two-context authority) |
| MMIO | P4-W05; P6-W11 (Guest-side observation) |
| SMP interaction | P6-W11 conditional multi-vCPU rows; P7-W10 (`p7-w10-validation-guest-suite`) |
| Scheduler interaction | P7-W10 |
| Authority isolation (capability/handle, cross-VM misuse) | P5-W07 |

## 3. Retained-scenario inventory

The inventory consolidates the source suites' scenarios. Column discipline:
`Observable` is cited from the source design/record, never restated; `P8
applicability` is the W19-owned judgment (unchanged / adapted-input /
blocked) recorded per [02](02-coexistence-and-workflow.md) §4; `Evidence` is
the source suite's verification record (inherited) plus the W16 `VG-RET` run
record when executed in P8. Statuses below are planned states, not results.

| Inventory group | Source scenarios | Mechanism | Observable authority | P8 applicability judgment |
|---|---|---|---|---|
| VG-001–VG-012 (P4 core set) | EL1 marker, CurrentEL, RAM/stack/code, unmapped fault, permission fault, WFI/WFE, controlled illegal behavior, and the remaining P4 rows per the P4-W05 design | EL1 entry, Stage-2, controlled exit | P4-W05 design and its implementation/verification records | unchanged; inputs re-expressed only if the approved P8 machine contract changes load parameters (adapted-input, see [02] §4) |
| P5 ABI-security set | discovery, valid HVC/references/data; unknown/invalid version, flags, length, address, overflow, type; generation misuse (zero/max/random/stale/destroyed); insufficient/no/revoked/cross-VM authority; repeated destruction; wrong lifecycle state | HVC ABI, handle/capability, authority | P5-W07 design and records; P5-W10 closeout links | unchanged |
| P5 two-context set | two independent security contexts; one context cannot use another's authority | authority isolation | P5-W07 design; consumed by [W18 §4.2](../p8-w18-security-isolation-regression/01-isolation-scenario-matrix.md) | unchanged; carries the W18 cross-VM boundary citation |
| VG-TIMER-01–04 | periodic timer, deadline behavior, masked/unmasked, repeated re-arm | virtual timer | P6-W11 design and records | unchanged |
| VG-IRQ-01–06 | vIRQ inject/deliver, mask/unmask, pending/active, repeated, concurrent | vIRQ/SGI | P6-W11 design and records | unchanged |
| P6 multi-vCPU conditional rows | multi-vCPU timer/IRQ isolation | SMP interrupt/timer | P6-W11 design; blocked-unless-evidenced rule inherited from P6 | unchanged; applicability re-checked against the evidenced P6 capability |
| P7 workload set | single-vCPU regression; multi-vCPU CPU-bound, periodic WFI, virtual timer, SGI/IRQ, shared-memory, HVC-heavy workloads | scheduler interaction | P7-W10 design and records; P7-W14 handoff links | unchanged; scenario inputs consume the P8 scheduler configuration where W11's scenarios declare it |

Inventory maintenance rules:

- The inventory is the single consolidated view; source designs remain
  authoritative. A source-suite change (new scenario, changed observable) is
  reflected here by reviewed edit, citing the source change.
- A row whose source suite has no evidenced record is an **assumed-contract
  placeholder**: it appears in the inventory with status
  `blocked:<owner>` and is ineligible for P8-V25 closure until the source
  evidence exists.
- W19 never merges, splits, or renames source scenarios.

## 4. Track-separation mapping

Per mechanism: the VG check is the precise, mechanism-level probe; the Linux
check is the OS-integration probe. Both exist precisely so neither is
inferred from the other. Linux check content is owned by the cited W8-stage
plans; this table only fixes the separation.

| Mechanism | Track VG proves (precise mechanism check) | Track LG proves (OS integration check) | Why both are needed |
|---|---|---|---|
| HVC / controlled exit | ABI accepts/rejects declared inputs; two-context authority holds (P5) | Linux does not depend on private HVC paths and uses the declared PSCI boundary ([P8-W06](../p8-w06-psci-virtualization/README.md)) | a compliant Linux hides ABI regressions that the P5 suite would catch; a green suite says nothing about Linux behavior |
| Stage-2 fault | fault fires at the exact declared IPA with declared syndrome class (P4) | Linux page-fault/COW/OOM flows operate with actionable Stage-2 diagnostics ([P8-W12](../p8-w12-linux-memory-model/README.md)) | mechanism precision vs. workload realism |
| Timer | deadline and masking semantics at declared granularity (P6) | Linux timekeeping, sleep, high-resolution behavior ([P8-W08](../p8-w08-linux-timer-integration/README.md)) | suite proves the mechanism; Linux proves the OS's use of it |
| SGI / vIRQ | injection, masking, pending/active at mechanism level (P6) | per-CPU IRQ/SGI under real kernel concurrency ([P8-W10](../p8-w10-linux-smp-bringup/README.md)) | kernel masking layers can mask (or unmask) behaviors the suite sees directly |
| MMIO | trap/abort at declared addresses with controlled exit (P4/P6) | console and device-window access from real drivers ([P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md)) | driver access patterns differ from probe patterns |
| SMP | multi-vCPU timer/IRQ isolation rows (P6/P7) | 2/4-vCPU boot, secondary start, stability workloads (P8-W10) | suite isolates the mechanism; Linux couples it with kernel SMP state |
| Scheduler interaction | P7 workload suite against scheduler semantics (P7) | 1:1 and declared M:N Linux progress ([P8-W11](../p8-w11-scheduler-linux-integration/README.md)) | scheduler correctness for a bare-metal loop is not Linux correctness and vice versa |

Standing rule (the anti-substitution clause): a green Linux track never
closes a red or blocked VG row, and a green VG track never closes a red or
blocked Linux row. Evidence and closure always report the two tracks as
separate columns.
