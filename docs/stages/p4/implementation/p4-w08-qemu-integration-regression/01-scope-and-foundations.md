# P4-W08 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W08 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-K01–K05)

- The declared build/image/boot boundary: how the Hypervisor and Validation
  Guest artifacts are built (pinned toolchain baseline), how the Guest image
  reaches the Hypervisor (the W03 embedding route, unchanged), and how QEMU
  is invoked (through the single P0-W09 entry).
- The SM-series scenario matrix covering: positive EL1 entry; translation-
  fault negatives (unmapped gap, RAM boundary, Hypervisor-owned range);
  permission negatives (read-only write, XN execute); controlled illegal and
  (planned) unknown-sync; WFI/WFE defined results; and the recovery/survival
  observation after each stop.
- The repeat set: same-session repeat (W07 episodes) and cold-boot repeats
  (W07 minimums) with stability determination.
- Collection and verdict rules: versioned marker/record parsing, determinate
  verdict taxonomy, timeout budgets, evidence retention.
- The evidence layout under `docs/stages/p4/verification/`.

### Reserved (must not be precluded; not implemented in P4)

- Environment matrices beyond the single declared reference configuration
  (CPU-count/RAM-size variants are P2-W09's pattern; P4 adds them only when
  a P4 requirement names them — none does) (re-entry: a later environment
  expansion).
- CI provider wiring and required-check configuration (re-entry: the P0 CI
  package and later CI governance).
- Real-hardware (Orange Pi) runs of the same matrix (re-entry: P15).
- Linux Guest regression (re-entry: P8's automated Linux regression).
- Performance timing or benchmark output capture (re-entry: a later
  performance-baseline package).

### Out of Scope

Hypervisor or Guest mechanism implementation, QEMU emulator features,
QEMU-as-architecture claims, hardware correctness claims, completion claims,
and any change to the consumed seams (VG grammar, IS expectations, P4-RR
grammar, runner internals).

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review; divergence becomes a recorded conflict (W01 §4) and the affected W08
step stops.

| ID | Assumed contract | Source (plan/design path) | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | W05 scenario table `VG-T<n>` with `VG-<id>:BEGIN/OK/FAULT:<KIND>/FAIL:<REASON>` markers, version string in the boot banner | [P4-W05](../p4-w05-validation-guest/02-guest-architecture-and-scenarios.md) §4–§6 | expected marker sequences per scenario are parseable and versioned | grammar changes bump the version; W08 re-syncs by agreement, never by fuzzy matching |
| M2 | W06 IS-series expectations with `Match`/`Mismatch` verdicts and W07 P4-RR v1 record with declared repeat minimums and count patterns | [P4-W06](../p4-w06-fault-isolation-diagnostics/02-architecture-and-state.md) §6; [P4-W07](../p4-w07-repeatability-telemetry/04-code-contracts-telemetry.md) §4–§5 | the machine-readable expected/diagnosed surface per episode is available for comparison | field/grammar drift is rejected via version lines (`P4RR:VER`), producing `INCOMPLETE`, never a guess |
| M3 | P0-W09 single QEMU runner entry with reserved parameter surface, serial capture, timeout, exit-status, and evidence conventions | [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) plan; [P1-W10](../../../p1/plans/p1-w10-qemu-boot-regression.md) conventions; W01 R06 | P4 parameters ride the reserved surface; capture/timeout/evidence behavior is inherited, not reinvented | if the entry or surface is absent/unsuitable, W08 records a blocked prerequisite against P0-W09; a P4-local runner is out of scope |
| M4 | Pinned toolchain/target baseline builds both artifacts reproducibly | [P0-W02](../../../p0/implementation/p0-w02-rust-toolchain-baseline/README.md) design; P0-W03/W07 plans; W01 R18 | the declared build commands run under the pinned toolchain; build identity is obtainable | build variance or missing identity makes runs incomparable → declared-environment label fails; W08 reports `UNSUPPORTED` rather than comparing across builds |
| M5 | P0 logging baseline emits the run record and `diag.*` events to the console with build identity | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)/[P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) plans; W07 M5 | serial capture contains the P4-RR record lines as whole lines | degraded logging is detected as missing record lines → `INCOMPLETE` with the degraded basis named |
| M6 | W02/W03/W04 mechanisms delivered so the scenarios can run | [P4-W02](../p4-w02-stage2-address-space/README.md), [P4-W03](../p4-w03-guest-memory-image/README.md), [P4-W04](../p4-w04-vcpu-entry-exit/README.md) designs; W01 R01–R17 upstreams | the scenarios exercise real mechanisms; timeouts are environment failures, not missing-mechanism hints | missing mechanisms are blocked prerequisites (W01 §4); W08 does not mask them with relaxed verdicts |

Entry-order boundary: manifest schema, verdict rules, and evidence layout
can be built and reviewed before any run; every execution row
(P4-V13–V15) is blocked until M1–M6 deliver. The
[workflow](04-implementation-workflow.md) marks the gates.

## 3. Authority analysis for contested areas

- **Expectation authority.** The Guest marker grammar is W05-owned; IS
  verdicts are W06-owned; the run record is W07-owned. W08 owns only the
  *comparison rules* (how evidence maps to verdicts). This prevents the
  classic regression failure where the harness redefines expected behavior
  and the suite passes by construction. Any disagreement re-opens the owning
  design, never the verdict rules.
- **QEMU facts vs architecture facts.** Expected observables are
  Guest/Hypervisor-emitted contracts; QEMU's own behavior (timing, TCG vs
  KVM differences, device quirks) is environment, not expectation. A
  QEMU-specific divergence surfaces as a failed expectation plus a
  Specification Investigation item (W01 A7); the manifest never encodes a
  QEMU workaround as an expected value.
- **"Survival" observable.** P4-V14's "defined EL2-survival or Guest-stopped
  result" is made objective via W07's run record: after every fault stop,
  the record must continue (episode end, run end lines present) and the
  W06 report must have been rendered — EL2 demonstrably stayed live. No
  heuristic console reading is involved.
- **Single configuration scope.** The task book requires the QEMU `virt`
  reference environment, not a matrix; P2-W09's variant matrix is the
  precedent for later expansion. Keeping P4 to one declared configuration
  keeps P4-V15's stability claim well-defined; widening is Reserved.
- **P2-ACR-01 visibility.** Not touched by W08 (no memory-model semantics in
  automation); it remains visible via W09's unresolved-conflict list.
- **Requirement-wording provenance.** Fine-grained K wording of the
  superseded root task book is untracked (W01 A9 pattern); meaning is fixed
  from the tracked task book §5/§6 rows and the W08 plan scope, cited in the
  implementation record.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | Single entry point: one P4 regression command extending the P0-W09 runner entry with a P4 parameter set (scenario selection, repeat counts, evidence directory); no second QEMU path, no QEMU flags in Core or package docs outside the automation layer | P0-W09's purpose is "唯一、可扩展的 QEMU virt 自动化运行入口"; duplicated commands drift | P0-W09 plan; P1-W10 reuse duty; W01 R06 |
| D2 | SM-series matrix fixed in this design, keyed to VG ids + IS expectations + P4-RR fields; matrix versioned (`SM-T1`) and bumped only with the owning designs' agreement | harness must consume, never redefine, expectations; versioning aligns all four tables (VG/IS/RR/SM) | W05 §6 change rules; W07 D6; W06 O2 |
| D3 | Verdict taxonomy: `PASS`, `FAIL`, `TIMEOUT`, `INCOMPLETE`, `UNSUPPORTED`; exactly one verdict per scenario iteration and per set; every non-`PASS` is determinate and evidence-backed | plan work sequence item 5 requires timeout/incomplete/unsupported as diagnosable non-success; a tri-state pass/fail would hide the other failure modes | P4-W08 plan item 5; P1-W10 timeout/missing-marker precedent |
| D4 | Accepted evidence = versioned marker lines (VG), IS verdicts via P4-RR fields, and the complete P4-RR record; free-form console text is never a pass condition | objective, parseable, version-checked; excludes output-order-only criteria | P1-W10 work sequence item 4; W07 D6 |
| D5 | Repetition rules: fresh QEMU process per iteration; fixed scenario sequence; iteration counts = W07 minimums by default, manifest may raise; per-iteration evidence retained; no randomness or seeds anywhere in the matrix | stability claims need independence and retention; "seed" controls would imply random inputs that do not exist | W07 D7; task book P4-V15 |
| D6 | Evidence layout: per-run directory under `docs/stages/p4/verification/` with serial log, P4-RR record copy, build identity, manifest hash, verdict file; a run summary table in the W08 verification record | evidence must be findable, complete, and separate from design/plan documents | task book §3 delivery hierarchy; W01 A8 |
| D7 | Set verdict = `PASS` iff every iteration of every scenario in the declared set is `PASS`; any other outcome fails the set; truncated sets are `INCOMPLETE` | partial passes must never be reportable as regression success | task book P4-V13–V15 wording; plan acceptance |
| D8 | W08 ships automation tooling and documentation only; zero hypervisor/Guest code changes and zero new `unsafe` are authorized; discovered product defects are reported to the owning package | scope boundary in the plan ("not implementing Guest mechanisms its input packages have not established"); keeps the audited unsafe surface fixed | P4-W08 plan scope; ADR-006; W01 R20 |

## 5. Open items recorded by this design

| ID | Item | Owner / path | Handling |
|---|---|---|---|
| O1 | The P0-W09 runner's reserved parameter surface must carry P4's parameters (scenario set, repeat counts, evidence directory). Its exact shape is P0-owned and undelivered. | [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) owner; W08 records the requirement | resolved when P0-W09 delivers; until then execution rows stay blocked; no P4-local runner is created |
| O2 | Declared-environment label contents (QEMU version, machine options, accelerator) must come from the P1-W10/P0-W09 environment declarations; W08 consumes them as data. | P1-W10/P0-W09 owners | if absent at implementation, `UNSUPPORTED` is the honest verdict for environment-sensitive rows until declared |
