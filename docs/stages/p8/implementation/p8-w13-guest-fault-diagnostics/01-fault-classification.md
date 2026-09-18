# P8-W13 Fault Classification

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W13 detailed design](README.md).

## 1. Upstream baselines consumed and their failure boundaries

"Evidenced" means present in the P0/P4 stage material referenced by their
handoff plans (P4-W09 `../../../p4/plans/p4-w09-closeout-p5-handoff.md`; P0
records); planned wording alone never satisfies a row.

| Baseline | Owning plan | Assumed, evidenced fact | Failure boundary if different |
|---|---|---|---|
| Exit categorization and Guest-boundary context | P4-W06 (`../../../p4/plans/p4-w06-fault-isolation-diagnostics.md`) | Exits distinguishable; Guest/vCPU, PC, state, IPA, access type, mapping, and reason captured; unknown synchronous conditions diagnosable; Guest faults never become unexplained EL2 panics | `P4DependencyIssue`; classification consumes whatever fields exist and every missing field is a diagnostic gap, never invented |
| Failure-class semantics | P0-W14 (`../../../p0/plans/p0-w14-panic-failure-classification.md`) | InvariantViolation / GuestCaused / ResourceExhausted / Unsupported / PlatformFailure distinctions with guest-caused faults contained to the Guest | Taxonomy top level does not apply; stop and record a conflict against the P0-W14 authority |
| Diagnostic semantics and version identity | P0-W12 (`../../../p0/plans/p0-w12-logging-diagnostic-baseline.md`) | Log levels, structured-trace semantics, minimum crash information, build/version identity rules | Context identity and trace-window rules cannot be stated; `P0DependencyIssue` |
| W05 CPU-operation classification | `../p8-w05-linux-cpu-virtualization/README.md` | Direct/Emulate/Reject/Hidden/Unsupported classification with controlled diagnostics | UnsupportedSysReg/RejectedOperation classes lack their source facts; coverage blocked for those classes |
| W06–W08 declared fault surfaces | `../p8-w06-psci-virtualization/README.md`, `../p8-w07-linux-vgicv3/README.md`, `../p8-w08-linux-timer-integration/README.md` | Declared invalid-PSCI, vGIC-contract, and timer-contract violation behaviors | Corresponding classes lack declared surfaces; coverage blocked per class |
| Containment and shutdown | P7-W07 (`../../../p7/plans/p7-w07-pause-stop-fault.md`); `../p8-w06-psci-virtualization/README.md`, `../p8-w03-linux-boot-contract/README.md` | Faulted/stopped vCPUs excluded; other scheduling unaffected; declared shutdown path | VM-facing outcomes undefined; `P7DependencyIssue` |
| Linux fault sources | W09–W12 records; fixture | Linux panics, Guest aborts, and probe-induced faults observable on the console path | F13-1/F13-3 coverage blocked until Linux evidence exists |

## 2. Classification model

A fault event is one Guest-caused or hypervisor-observed failure occurrence
arriving through the evidenced P4-W06 boundary. Classification assigns each
event exactly one class from the taxonomy (§3) using the decision rules (§4);
routing attaches the contained outcome and required observables (§5–§6). The
classification step is a pure function of recorded exit facts and context — it
reads nothing from Guest memory beyond what the boundary already captured, and
its result never depends on Guest-controlled data as a trusted input.

## 3. Taxonomy

### 3.1 Guest-facing family (VM-facing; recoverable; contained)

| ID | Class | Source surface | Originating package |
|---|---|---|---|
| F13-1 | `GuestKernelPanic` | Linux reports its own panic/crash on the console (W09 path); possibly preceded by a Guest abort exit | W09/W12 |
| F13-2 | `GuestSynchronousException` | EL1 synchronous exception routed to EL2; subclassified by the W05 classification: unsupported sysreg access, rejected operation, undefined instruction, or an exception class W05 did not anticipate | W05 |
| F13-3 | `Stage2Fault` | Stage-2 translation or permission fault on Guest access, with mapping-query result | P4-W06; observed at scale by W12 |
| F13-4 | `InvalidMmioAccess` | Guest access outside the declared Guest MMIO windows (approved map; console window per W09) | W04/W09 |
| F13-5 | `PsciViolation` | PSCI call outside the declared W06 subset: unknown FID, invalid parameters, invalid CPU/state target, or sequence violation | W06 |
| F13-6 | `VgicViolation` | Guest vGIC access or behavior inconsistent with the declared W07 contract | W07 |
| F13-7 | `TimerContractViolation` | Guest timer access/behavior that cannot be served per the declared W08 contract (not ordinary timer programming, which is Guest-free) | W08 |
| F13-8 | `VcpuStateAnomaly` | Guest-visible behavior inconsistent with the declared vCPU lifecycle (P7-W02), e.g., wakeup/event targeting a vCPU whose Linux-visible state cannot explain it | P7-W02; observed by W11 |

### 3.2 Hypervisor-invariant family (hypervisor-level; never VM-facing)

| ID | Class | Source surface | Handling |
|---|---|---|---|
| F13-9 | `HypervisorInvariantViolation` | Any ADR §19-class break observed in EL2: corrupted scheduler/memory/IRQ state, ownership break, Guest input causing an EL2-side fault | Per P0-W14 invariant handling (EL2-level failure path owned by P0-W14/P1-W07). W13 records the classification and context; it does not define the handler |
| F13-10 | `PlatformFailure` | QEMU/host/environment failure (per P0-W14's platform class) | Recorded as environmental; rerun policy belongs to W16 |

### 3.3 Escape

| ID | Class | Handling |
|---|---|---|
| F13-0 | `Unclassified` | No rule matched. Always recorded as a block with full captured context; carried to closure review (workflow step 6); never forced into a class |

## 4. Decision rules

Ordered rules; the first match wins. Inputs are only the P4-W06-evidenced exit
facts, mapping-query results, and the W05 classification of the underlying
operation.

1. EL2-side fault or detected EL2-state corruption → `F13-9`.
2. Environment-level failure (host/QEMU crash not attributable to Guest
   action) → `F13-10`.
3. Console evidence of a Linux-initiated panic with no EL2 fault → `F13-1`.
4. Exit is a Stage-2 fault → `F13-3` (permission vs translation per the
   mapping-query result).
5. Exit is a trapped synchronous exception from EL1 → `F13-2`, subclassed per
   the W05 classification; an operation W05 classifies Reject/Unsupported whose
   diagnostic outcome was produced stays `F13-2`.
6. Access target outside declared MMIO windows → `F13-4`.
7. PSCI call outside the declared W06 subset or with invalid parameters/state →
   `F13-5`.
8. vGIC access/behavior outside the declared W07 contract → `F13-6`.
9. Timer access/behavior unservable per the declared W08 contract → `F13-7`.
10. Lifecycle-inconsistent Guest-visible event with otherwise valid exits →
    `F13-8`.
11. Otherwise → `F13-0` (block).

Notes: a Linux panic preceded by a Stage-2 fault classifies as `F13-3` with the
panic recorded in its console-capture context — the EL2-observable cause wins,
the Guest narrative is evidence, not the class. Classes F13-5/6/7 fire only
when the corresponding contract declares the violated surface; an unservable
request on an undeclared surface is `F13-0`.

## 5. Containment and routing per class

| Class(es) | Contained outcome (authority) | Required observables |
|---|---|---|
| F13-1 | VM/vCPU proceeds to the declared Guest-faulted outcome per P7-W07 and the W03/W06 shutdown contract; EL2 unaffected | Console capture (Linux panic text), vCPU state at exit, class, context |
| F13-2 | Per the W05 declared outcome for the operation (Reject → guest-visible error; Unsupported → declared diagnostic/controlled outcome); never silent corruption | Exit facts, W05 classification, VM-facing response |
| F13-3 | VM-scoped diagnostic; Linux observes its own abort; EL2 unaffected (P4-W06 basis; W12 verdicts) | Full context incl. IPA, access type, syndrome, mapping query |
| F13-4 | VM-scoped diagnostic; in-window console unaffected | Address, window facts, console-health evidence |
| F13-5 | Guest-visible error per the PSCI contract; state unchanged | FID/params (as captured), violation rule |
| F13-6 / F13-7 | Declared W07/W08 diagnostic/controlled outcome; VM-scoped | Violating access facts, contract rule |
| F13-8 | Scheduler/lifecycle investigation item (feeds W11 routing); contained unless it degrades to F13-9 | Event facts, lifecycle state, recent trace |
| F13-9 | Hypervisor-level failure per P0-W14; never absorbed as a Guest fault; other-VM impact recorded as part of the failure, not normalized | Full EL2 context per the evidenced P4-W06/P1-W07 boundary |
| F13-10 | Environmental record; no VM attribution | Host/QEMU facts |
| F13-0 | Block: recorded with full context; reviewed at closure | Everything captured |

Absolute rule (ADR §19, P0-W14): no Guest-facing class may produce an EL2
panic, hang, or cross-VM effect. Observing one is a `HypervisorInvariantViolation`
finding against the containment contracts, not a reclassification.

## 6. Console and regression observable mapping

For automated consumption ([W16](../p8-w16-automated-linux-regression/README.md),
[W18](../p8-w18-security-isolation-regression/README.md)), each Guest-facing
class maps to:

- a **console observable**: the deterministic marker or transcript shape on the
  W09 console path (e.g., Linux panic banner for F13-1, declared diagnostic
  line for F13-2 Reject outcomes); exact diagnostic line formats are owned by
  the producing designs (W05–W08), not invented here;
- a **regression assertion**: expected class + contained outcome +
  `diagnostic_context_sufficient` = true ([02 §3](02-diagnostic-context-contracts.md)),
  checked per induced event;
- a **forbidden outcome** list: EL2 panic/hang, cross-VM effect, missing or
  insufficient context — any of which fails the run regardless of the console
  transcript.

## 7. Coverage map (scenario → class)

Declared inducing scenarios (realized via W15 fixture loads and W05–W12
scenarios; executed by W16):

| Scenario | Induced class(es) |
|---|---|
| Linux kernel panic under fixture control (e.g., requested by a declared fixture test hook) | F13-1 |
| Declared W05 unsupported-operation probe | F13-2 |
| W12 `map-probe` N1–N5 rows | F13-3, F13-4 |
| Declared invalid PSCI call probe from the fixture (within the W06-declared surface) | F13-5 |
| Declared W07/W08 violation probes | F13-6, F13-7 |
| W11 lifecycle-anomaly observations | F13-8 |
| Not induced by design | F13-9, F13-10, F13-0 |

F13-9 is never deliberately induced; its "coverage" is the standing forbidden-
outcome assertion on every other scenario plus the escalation path. F13-0's
coverage is the block-review process. A scenario that cannot induce its class
without a new EL2 mechanism is blocked — injection facilities are not designed
here.
