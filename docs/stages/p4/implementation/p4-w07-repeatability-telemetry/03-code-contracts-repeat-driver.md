# P4-W07 Code Contracts — Repeat Driver and Determinism

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).  
**Companion:** module map, episode machine, and determinism surfaces in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names are P4-internal and unstable-by-declaration; P4-W09 records them as
implemented facts only. Pseudocode is an outline; the Coding Guidelines govern
the final Rust shape. W02/W03/W04/W06 contracts are consumed as fixed seams
(assumed contracts M2–M4) and are not redefined here.

## 1. `EpisodePlan` and `RunPlan` (module `run-driver`)

- **Name and stability:** `EpisodePlan { scenario: ScenarioId, mode:
  TeardownDepth, expectations: ExpectationId }` with `TeardownDepth —
  FullRebuild | RamReuse`; `RunPlan { episodes: BoundedList<EpisodePlan>,
  cold_boot_label: DeclaredEnvironmentLabel }`. Internal, P4-unstable.
- **Purpose and caller:** the declared repeat configuration; constructed by
  the run setup from constants; consumed by `RunDriver::new`.
- **Preconditions:** scenario ids valid per the W05 table; expectation ids
  valid per the W06 `IS-T1` table; episode list length ≥ the same-session
  minimum (D7) when the plan is the P4-V10 evidence plan.
- **Postconditions:** the plan is immutable; it contains no Guest-derived
  data.
- **Errors:** construction errors for invalid ids or below-minimum counts
  (`PlanInvalid`) — VM-facing setup failures.
- **Security:** plans are host-authored; nothing here is Guest-reachable.
- **Validation:** plan-validation unit tests; cross-table id agreement review
  (W05/W06 versions cited).

## 2. `RunDriver::new` and run-result types

- **Name and stability:** `fn new(plan: RunPlan, ledger: &mut
  TelemetryLedger) -> Result<RunDriver, DriverError>`. Internal.
- **Purpose and caller:** construct the driver with its accumulator and
  ledger reference; called once per cold boot by the run setup.
- **Preconditions:** no other live driver (structural in P4: setup creates
  one); ledger empty or explicitly reset.
- **Postconditions:** driver `Idle`; episode index 0; reference surfaces
  unset.
- **Errors:** `DriverAlreadyActive`-class misuse is a host invariant
  violation (fatal escalation per M1), never a recoverable value.
- **Validation:** unit construction tests; misuse test documents the fatal
  path.

- **Name and stability:** `RunResult { verdicts: BoundedList<EpisodeResult>,
  determinism: DeterminismSummary, counts: RunCountSummary, record_emitted:
  bool }` with `EpisodeResult { index, scenario, mode, stop_cause,
  match_verdict, surfaces: SurfaceSnapshot, accounting_ok: bool }` and
  `DeterminismSummary — Stable | Diverged(surface, index) | Incomplete`.
  Internal.

## 3. `run_episode`

- **Name and stability:** `fn run_episode(&mut self, plan: &EpisodePlan) ->
  EpisodeResult`. Internal; the driver's core.
- **Purpose and caller:** execute one full Guest episode (construct → run →
  stop → teardown) per the W03 §4 order and W04 §4 sequencing, capturing the
  determinism surfaces; called by the driver's run loop only.
- **Inputs/outputs:** the episode plan → the episode result with surfaces,
  stop cause, expectation verdict, and accounting outcome.
- **Preconditions:** driver holds the retained `GuestRam` when `mode ==
  RamReuse` (else invariant); no live episode; setup context (not IRQ, not
  exit path).
- **Postconditions:** on success, no Guest object remains except the
  driver-retained `GuestRam` under `RamReuse` (recorded in the result's
  depth field); the ledger holds this episode's count vector and, for fault
  scenarios, one correlation record; on failure, the run aborts per §4.
- **State/ownership change:** creates and consumes the episode's space/vCPU;
  transfers `GuestRam` ownership driver↔episode per depth; all transitions
  via the sibling contracts.
- **Concurrency/allocation:** allocation allowed (setup context); bounded by
  the P4 one-Guest sizes; no locks held across the Guest run segment (the
  run segment is W04's masked path).
- **Errors:** sibling errors surface verbatim (`Stage2Error`,
  `GuestMemoryError`, `VcpuError`) with the episode aborting; W07 adds
  `AccountingImbalance` (M6 residue check) and `TeardownDepthViolation`.
  Guest-caused conditions are outcomes (`stop_cause`), never errors (W01 A2
  type split preserved).
- **Security:** all plan data is host-authored; the driver reads Guest RAM
  only through the W03 host view for the digest; nothing Guest-influenced
  affects sequencing.
- **Logic:**

```text
run_episode(plan):
    case plan.mode:
      FullRebuild:
        space = GuestAddressSpace::create(alloc, layout.span())?
        ram   = GuestRam::allocate(alloc, layout)?
      RamReuse:
        space = GuestAddressSpace::create(alloc, layout.span())?
        ram   = self.retained_ram else InvariantViolation      // fatal class
    grants = ram.mapping_grants()?
    for g in grants: space.map(range_for(g), g)?
    input  = load_guest(&mut ram, image, plan.scenario)?       // re-zeroes on
                                                               // RamReuse path
    digest = fnv1a_64(ram.host_view())                          // S1
    vcpu   = Vcpu::construct(input, plan.scenario)?             // S2 captured
    outcome = run_entry(&mut vcpu, &mut space)                  // W04; W06
                                                               // diagnosis runs
                                                               // inside stop
    accounting_ok = check_accounting_restored(alloc, baseline)  // M6
    case (plan.mode, outcome.stopped()):
      (FullRebuild, _) -> vcpu.destroy(); space.destroy(); ram.release(unmap_proof)
      (RamReuse,    _) -> vcpu.destroy(); space.destroy()
                          self.retained_ram = ram                 // keep; reload
                                                               // next episode
    surfaces = snapshot(outcome, digest, markers, counts)
    return EpisodeResult { .. }
```

- **Validation:** host-side state-machine tests with stubbed W02/W03/W04
  surfaces (both depths, abort paths, truncation-point recording); on-target
  evidence via DV01–DV03 and the W08 repeat runs.

## 4. `run_all` and failure boundary

- **Name and stability:** `fn run_all(&mut self) -> RunResult`. Internal.
- **Purpose and caller:** iterate the plan's episodes, compare surfaces, and
  produce the run result; called by the run setup once per cold boot.
- **Preconditions:** driver `Idle` or mid-run (resumable after a handled
  abort only for diagnostic reads); ledger attached.
- **Postconditions:** driver consumed (`run_all` is terminal); the run record
  has been emitted (or its emission failure recorded); every episode has a
  determinate result — success, failure, or aborted-with-cause; nothing is
  left ambiguous.
- **Failure guarantee:** any episode step failure or accounting imbalance
  aborts the run immediately with `DeterminismSummary::Incomplete` plus the
  failing episode's cause; the driver never starts a new episode after an
  abnormal teardown (W04 §4 duty). A determinism divergence does *not* abort
  remaining episodes by default — divergence is evidence, and the full
  episode list still runs so the divergence pattern is observable; the run
  verdict is `Diverged` regardless.
- **Errors:** first error wins; subsequent errors are recorded, not raised.
- **Logic:**

```text
run_all():
    for (i, ep) in plan.episodes():
        result = run_episode(ep)
        ledger.close_episode(i, result)
        emit run.episode.end
        if !result.accounting_ok or result.abnormal_teardown:
            record.append(result); return Incomplete(result)
        if i == 0: reference = result.surfaces
        else if result.surfaces != reference:
            record.diverged(surface_of_first_difference, i)   // keep running
    determinism = compare(reference, all)                     // S1–S5
    counts = ledger.summary()
    emit run.summary; emit P4-RR record
    return RunResult { .. }
```

- **Validation:** driver-loop unit tests (abort-on-teardown-failure,
  continue-on-divergence, record-emitted flag); on-target via W08 repeat
  runs (DV04).

## 5. Determinism checks (module `determinism`)

- **Name and stability:** `fn digest_ram(view: &GuestRamView) -> SurfaceDigest`
  (FNV-1a-64, D4); `fn contexts_equal(a: &VcpuContext, b: &VcpuContext) ->
  bool` (typed field comparison); `fn sequences_equal(a: &[(ExitClass,
  StopCause)], b: &[(ExitClass, StopCause)]) -> bool`;
  `fn counts_equal(a: &CountVector, b: &CountVector) -> bool` (episode-local
  deltas). Internal, pure.
- **Purpose and caller:** the five surfaces (S1–S5) of
  [02 §5](02-architecture-and-state.md); called by the driver at episode end
  and by the final comparison.
- **Inputs/outputs:** sibling-owned values → equality results/digests.
- **Preconditions:** the digest reads only the W03-provided host view over
  the region (bounded, setup context); sequences are the declared expected
  prefix for fault scenarios (S3 note).
- **Postconditions:** pure functions; identical inputs → identical outputs.
- **Concurrency/allocation:** no allocation; digest is linear in region size
  (bounded by the P4 RAM size, setup context — no time constraint).
- **Errors:** none.
- **Security:** the digest is a comparison value only; it never gates control
  flow except as recorded evidence; Guest RAM content cannot influence the
  driver beyond producing a divergence verdict.
- **Logic (digest):**

```text
digest_ram(view):
    h = FNV_OFFSET basis
    for page in view.pages():
        for b in page.bytes(): h = (h xor b) * FNV_PRIME
    return SurfaceDigest(h)
```

- **Validation:** known-answer tests for the digest; equality/inequality
  tests per surface; re-init equivalence test (allocate+init vs retain+re-init
  produce equal digests — D8); host-side only (RAM content equality is
  environment-independent).

## 6. Error model recap

- `DriverError`: `PlanInvalid` (recoverable setup), plus surfaced sibling
  errors verbatim. Misuse-class conditions (second driver, missing retained
  RAM, teardown-depth violation) are host invariant violations escalating
  through the P0 failure classification (M1) — fatal, never swallowed, never
  adapted around.
- Guest-caused conditions are outcomes in `EpisodeResult` (`stop_cause`,
  `match_verdict`), preserving the W01 A2 boundary end to end.
- `AccountingImbalance` fails the run and is reported with the leaked-page
  facts per W03 §3.4's leak-reporting precedent.
