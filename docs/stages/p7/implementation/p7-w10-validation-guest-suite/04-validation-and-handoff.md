# P7-W10 Validation Matrix, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W10 detailed design](README.md).  
All rows are planned evidence with objective conditions; none claims that a
test ran. Results go only to
`../../verification/p7-w10-validation-guest-suite-verification.md`.

## 1. Validation matrix

| ID | Maps to | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W10-DV01 | P7-V22 | Single-vCPU regression run | Execute the inherited P4/P5/P6 scenario sets on 1VM/1vCPU/1pCPU under the scheduled entry path; diff markers/expectations against upstream records | Every inherited scenario matches its original expectations; entry/exit visible as scheduler-controlled in telemetry | Stability of inherited behavior under scheduler control; not any multi-vCPU property |
| W10-DV02 | P7-V23 | VG-SCHED-01/-05/-08 progress workloads | Run the counter/shared-memory/multi-VM workloads per catalog topologies | Completion markers from every vCPU/VM; telemetry shows interleaved switches, nonzero per-entity runtime, no invariant trips | M:N progress and switch isolation at Guest scale; not fairness ratios or performance |
| W10-DV03 | P7-V23 | VG-SCHED-02/-03/-04 wake workloads | Run WFI-block, SGI-wake, and timer-wake scenarios | No busy re-entry; SGI receipts gap-free and equal to sends; timer wakes exactly as programmed; telemetry sources match | Block/wake behavior with Guest-side witnesses; not SGI/timer latency or hardware behavior |
| W10-DV04 | P7-V23 | VG-SCHED-06 exit-storm workload | Run the HVC-heavy mix | All results in expected P5 classes; accounting coherent through the storm; no contained fault misclassification | Scheduler robustness under Guest-controlled exit rates; not HVC ABI semantics (P5's evidence) |
| W10-DV05 | P7-V23 | VG-SCHED-07 pause/stop observation | Run the two-context pause/resume/stop scenario | No markers during pause; gap-free resume; stopped/faulted never re-executes; dispositions visible in telemetry | Guest-observable pause/stop containment; not VM fault policy |
| W10-DV06 | P7-V23 | Two-sided correlation check | For each executed scenario, compare marker streams with telemetry correlates per the authority split | Every authority assignment holds; disagreements (if any) are recorded findings, never silently reconciled | Correlation method integrity; not the underlying scheduler proofs themselves |
| W10-DV07 | suite integrity | Platform-contract review | Apply §5 rules of the asset contract to the delta | All five rules hold; inherited scenarios unchanged; no ABI/machine-types content touched | Suite discipline; not hypervisor correctness |
| W10-DV08 | W10 closure | Suite completeness review | Check every catalog scenario has workload module, expectation entry, and evidence destination | Catalog, asset, and tables are complete and mutually consistent | Handoff readiness for W11/W12; not execution |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment (including QEMU version and emulated CPU
count), timestamp, and reason. All execution rows require the P1/P4 QEMU
baselines and the W04–W09 behavior they exercise; without them they are
recorded not run with the owning stage named. Global boundary: QEMU success
does not prove hardware behavior (RK3566 evidence is later-stage work); a
passing scenario proves its declared row only — none proves fairness
verdicts (W11), performance conclusions (W13), or P8 behavior.

## 2. Error, security, and observability model

- **Errors.** Scenario failure classes: marker mismatch (Guest-visible
  behavior deviated), telemetry mismatch (scheduler facts deviated),
  correlation discrepancy (both), blocked prerequisite. Each is recorded
  with both observation sides; a scenario failure is evidence about the
  scheduler contract under test or the suite itself — the record states
  which, and never adjusts the suite to force agreement.
- **Security.** The suite treats the Guest as untrusted in both directions:
  Guest workloads must not gain authority (no new interfaces, §5 rule 1),
  and Guest-reported facts never decide scheduler claims (decision 6).
  Markers leak no Host pointers or capability values; scenario loading
  follows the P4 memory/image conventions with their validation intact.
- **Observability.** The suite is itself an observability consumer: its
  expectation tables exercise the W09 trace/counters end-to-end, so W10
  evidence doubles as verification that scheduler observability is usable
  from outside the hypervisor. Every evidence row carries environment
  identity so W12's automation and W13's baselines can reproduce conditions.

## 3. Handoff checklist

Before handing W10 to a reviewer and to consumers:

- Changed-file list (Guest asset workload modules, expectation tables,
  record files only; no hypervisor source changes — or, if a defect fix was
  required, a pointer to the owning design's record for it).
- Assumed-contract status from workflow step 1 with per-scenario blocked
  prerequisites named.
- W10-DV01–DV08 status with evidence paths, per-scenario, including
  explicit not-run entries naming the missing baselines.
- Confirmation: no new `unsafe`, no ABI/public API, no machine-ABI or
  platform-contract documentation changes, no automation scripts, no
  timing-dependent pass conditions.
- Scenario inventory handed to **[P7-W11](../p7-w11-stress-invariants/README.md)**:
  IDs, topologies, markers, scaling notes (which parameters may be amplified
  for P7-V24–V27), and each scenario's proof boundary.
- Determinate expectations handed to **[P7-W12](../p7-w12-qemu-regression/README.md)**
  (via W11) for the automated matrix (P7-V28), and workload shapes noted for
  **[P7-W13](../p7-w13-performance-baseline/README.md)** — with the reminder
  that P8 receives no machine-ABI commitment (parent README handoff).
- Open items for consumers (not resolved here): W11 owns stress verdicts;
  W12 owns automation and its matrix; W13 owns method and limits.
- Record locations: implementation notes to
  `../p7-w10-validation-guest-suite-record.md`; evidence to
  `../../verification/p7-w10-validation-guest-suite-verification.md` — both
  created only when work or evidence exists.
