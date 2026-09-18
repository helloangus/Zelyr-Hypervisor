# P4-W07 Code Contracts — Telemetry Ledger and Run Record

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).  
**Companion:** telemetry points and surfaces in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names are P4-internal and unstable-by-declaration; P4-W09 records them as
implemented facts only. The counter set and grammar are P4-local and
temporary: they are not the final telemetry API (task book §1 Out of Scope)
and changes follow the joint-review rules stated here. Pseudocode is an
outline; the Coding Guidelines govern the final Rust shape.

## 1. Event-source inventory (read-only seam into sibling events)

The ledger counts events that W02–W06 already emit through the P0 baseline
(assumed contracts M2–M4; one emission source per event, D5):

| Domain | Events counted (stage-local names) | Source |
|---|---|---|
| memory/Guest RAM | `gm.ram.allocate`, `gm.ram.release`, `gm.load.plan`, `gm.load.copy`, `gm.bootinfo.write` | [P4-W03](../p4-w03-guest-memory-image/02-architecture-and-state.md) §7 |
| Stage-2 | `s2.space.create`, `s2.map`, `s2.unmap`, `s2.protect`, `s2.activate`, `s2.invalidate`, `s2.space.destroy` | [P4-W02](../p4-w02-stage2-address-space/02-architecture-and-state.md) §8 |
| vCPU | `vcpu.construct`, `vcpu.enter`, `vcpu.exit`, `vcpu.reenter`, `vcpu.stop`, `vcpu.destroy` | [P4-W04](../p4-w04-vcpu-entry-exit/02-architecture-and-state.md) §7 |
| diagnostics | `diag.exit`, `diag.fault`, `diag.isolation.verdict`, `diag.report`, `diag.dump` | [P4-W06](../p4-w06-fault-isolation-diagnostics/02-architecture-and-state.md) §7 |
| run (W07's own) | `run.episode.begin`, `run.episode.end`, `run.determinism`, `run.summary` | this design |

Counting mechanism: the P4 baseline (M5) is assumed to provide structured
events with names and fields; the ledger increments at the episode boundaries
by reading the episode's collected outcomes (the driver holds them) rather
than by hooking the emission path — one read-side aggregation point, no
second emitter, no hidden global counter. If the delivered baseline instead
supports subscription callbacks, the ledger adapts behind its own interface;
the counted set and semantics do not change.

## 2. `TelemetryLedger` contracts

- **Name and stability:** `struct TelemetryLedger` with `fn new() -> Self`,
  `fn close_episode(&mut self, index: EpisodeIndex, result: &EpisodeResult)`,
  `fn episode_vector(&self, index: EpisodeIndex) -> &CountVector`,
  `fn record_fault(&mut self, c: FaultCorrelation)`,
  `fn summary(&self) -> RunCountSummary`. Internal.
- **Purpose and caller:** per-run aggregation for P4-I01/I02; called only by
  the driver (`run-episode`/`run-driver` modules).
- **Inputs/outputs:** episode results and driver-captured outcomes → count
  vectors, correlation records, summary.
- **Preconditions:** `close_episode` once per episode, in order; `record_fault`
  once per diagnosed fault (driver holds the W06 diagnostic reference).
- **Postconditions:** counters are monotonic within a run; vectors are
  episode-local deltas; the summary aggregates all closed episodes.
- **State/ownership change:** grows the correlation list by bounded appends
  (one per fault; P4 scenarios produce at most one fault per episode by W05
  design).
- **Concurrency/allocation:** setup-context only; bounded fixed-capacity
  storage sized by the plan (no unbounded growth); no allocation after
  driver construction.
- **Errors:** `LedgerOverflow` if the plan's fault budget is exceeded —
  surfaces as an episode failure (an uncountable run is a broken assumption,
  not a truncated report).
- **Security:** correlation fields are host-captured addresses/classifications
  only; no Guest data content is recorded (same rule as the sibling events).
- **Logic (close_episode):**

```text
close_episode(i, result):
    v = CountVector::zero()
    v.bump(gm.*, from result.construct_facts)
    v.bump(s2.*, from result.mapping_facts)
    v.bump(vcpu.*, from result.outcome)
    v.bump(diag.*, from result.diagnosis)
    vectors[i] = v
    if let Some(d) = result.diagnosis_of_fault:
        record_fault(FaultCorrelation {
            episode: i, vcpu: d.vcpu, scenario: result.scenario,
            guest_pc: d.guest_pc, faulting_ipa: d.ipa,
            class: d.class, access: d.access,
            mapping_agreement: d.mapping_agreement,
            expectation_verdict: result.match_verdict,
            stop_cause: result.stop_cause })
```

- **Validation:** unit tests with synthetic episode results (vector
  correctness, monotonicity, correlation completeness); review that every
  P4-V12 event category has a counting path.

## 3. `FaultCorrelation` (P4-I02)

- **Name and stability:** `FaultCorrelation { episode: EpisodeIndex, vcpu:
  VcpuId, scenario: ScenarioId, guest_pc: u64, faulting_ipa:
  IpaReconstruction, class: ExitClass, access: Option<FaultAccess>,
  mapping_agreement: MappingAgreement, expectation_verdict: MatchVerdict,
  stop_cause: StopCause }`. Internal value type.
- **Purpose and caller:** the per-fault correlation record answering
  "which VM/vCPU, which scenario, which PC, which IPA, which class, which
  outcome"; assembled by the ledger from driver-held W04/W06 values.
- **Preconditions:** produced only from a completed W06 diagnosis (the
  driver never synthesizes fields).
- **Postconditions:** immutable; one record per diagnosed fault.
- **Contract notes:** fields cite W06's `ExitDiagnostic` verbatim (M4); W07
  adds only the episode/scenario keys. If W06 extends the diagnostic, the
  correlation record extension is a joint W06/W07 review item.
- **Validation:** completeness review — every P4-V12 correlation dimension
  present; unit test that a synthesized fault episode yields the expected
  record.

## 4. Run record — grammar `P4-RR v1` (module `run-record`)

- **Name and stability:** `fn emit_run_record(values: &RunRecordValues, out:
  &mut RecordBuf) -> Result<(), RecordError>`. Internal. Grammar version
  `P4-RR-1`, declared in every record's first line.
- **Purpose and caller:** the machine-readable run summary consumed by
  [P4-W08](../p4-w08-qemu-integration-regression/README.md) for verdicts;
  emitted once at run end through the P0 logging baseline (M5), line-oriented
  so serial capture preserves line boundaries.
- **Inputs/outputs:** driver/ledger values + build identity → record lines.
- **Field list (normative for v1):**

```text
P4RR:VER v1                           grammar version; first line
P4RR:BUILD <build-identity>           P0 version baseline (M5)
P4RR:ENV <declared-environment-label> from the run plan
P4RR:EP <i> <scenario> <mode> <stop-cause> <verdict> <digest-s1> <ctx-ok>
                                      <seq-ok> <counts-ok> <acct-ok>
                                      one line per episode
P4RR:REF <digest-s1>                  episode-0 reference digest
P4RR:COUNTS <domain>=<n> ...          run totals per domain
P4RR:FAULT <episode> <class> <ipa|-> <access|-> <agreement> <verdict>
                                      one line per correlation record
P4RR:DET <Stable|Diverged(surface,i)|Incomplete>
P4RR:RESULT <PASS|FAIL|ABORTED>       run-level self-assessment (W08 applies
                                      its own verdict rules on top)
P4RR:END                              last line
```

- **Preconditions:** called after `run_all` completes (including the abort
  path — an aborted run still emits a record with `RESULT ABORTED`); buffer
  sized per the plan's episode count.
- **Postconditions:** the record is a deterministic function of the values
  plus build identity; no timestamps or environment randomness appear in
  field values (the environment label is declared input); truncation is
  annotated per the W06 report rule.
- **Errors:** `RecordError::BufferTooSmall` surfaces (retry policy per the
  workflow; a missing record is a W08-visible non-success, never silent).
- **Security:** the record carries no Guest data content; addresses appear
  only in correlation lines as already-diagnosed values. It is host-authored
  output on the host console — never a Guest-writable surface.
- **Change rules:** any field addition/removal/rename bumps the grammar
  version (`P4-RR-2`, …) and requires [P4-W08](../p4-w08-qemu-integration-regression/README.md)
  agreement before merge (open item O1 of [01 §5](01-scope-and-foundations.md));
  the version line lets W08 reject records it does not understand instead of
  mis-parsing them.
- **Logic:**

```text
emit_run_record(values, out):
    line("P4RR:VER v1"); line("P4RR:BUILD", values.build)
    line("P4RR:ENV", values.env_label)
    for ep in values.episodes: emit_ep_line(ep)
    line("P4RR:REF", values.reference_digest)
    line("P4RR:COUNTS", values.totals)
    for f in values.faults: emit_fault_line(f)
    line("P4RR:DET", values.determinism)
    line("P4RR:RESULT", values.result)
    line("P4RR:END")
```

- **Validation:** golden-record unit tests (happy run, diverged run, aborted
  run); truncation tests; grammar conformance review with W08 (DV09).

## 5. Per-scenario expected count patterns (handoff to W08)

Minimum event-expectation patterns per scenario class (the automation
manifest may add stricter checks; it may not weaken these):

| Scenario class (VG ref) | Expected pattern (episode-local deltas) |
|---|---|
| Clean completion (VG-001/002/003/012) | 1 `s2.space.create`; ≥ 2 `s2.map`; ≥ 1 `vcpu.enter`; 1 `vcpu.exit` (class `Wfi`); 1 `vcpu.stop` (`Controlled`); 0 `diag.fault` |
| Translation-fault class (VG-004 / IS-01–03) | as clean, plus exactly 1 `diag.fault` with class `Stage2Translation`; stop `GuestFault` |
| Permission-fault class (VG-005/006 / IS-04/05) | as clean, plus exactly 1 `diag.fault` with class `Stage2Permission` and access `Write`/`Execute` respectively |
| Illegal/unknown (VG-010/011 / IS-06/07) | as clean, plus exactly 1 `diag.fault` with the respective class |
| WFI/WFE (VG-007/008) | as clean, exit class `Wfi`/`Wfe`, stop `Controlled` |
| Re-entry (planned VG-009) | ≥ 2 `vcpu.enter`/`vcpu.exit` pairs; ≥ 1 `vcpu.reenter`; final stop determinate |

Patterns are expressed over the counted event set of §1 and travel to W08 as
data (episode results + count vectors), not as re-derived expectations.

- **Validation:** pattern-satisfaction unit tests over synthetic vectors;
  on-target: every W08-run scenario episode satisfies its pattern (DV07).

## 6. Error model recap

`DriverError`/`RecordError` from
[03](03-code-contracts-repeat-driver.md) plus `LedgerOverflow`. All are
host-authored-contract or capacity conditions, surfaced and recorded; Guest
behavior never appears as an error. A telemetry failure must never corrupt a
run result: record-emission failure degrades the run verdict to a
non-success in W08's eyes (missing evidence), which is the honest outcome.
