# P4-W07 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).

## 1. Validation matrix

Driver logic and equality checks are host-testable; the repeat evidence
itself is on-target through the delivered W02–W06 paths, executed at scale
by [P4-W08](../p4-w08-qemu-integration-regression/README.md). This matrix
defines what W07's evidence must show; it is a plan until the verification
record exists.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W07-DV01 → P4-V10 | Same-session restart works | on-target episodes (step 5 plan) | 3+ episodes incl. 1 `RamReuse` through the real W02–W04 paths | every episode constructs, runs, stops, tears down; next episode starts clean | the P4-H01 cycle; not multi-VM or lifecycle policy |
| W07-DV02 → P4-V10 | No residue: accounting restoration | per-episode accounting checks + teardown-failure injection (host-side) | allocated == freed after each teardown; imbalance aborts run | zero imbalance across all episodes; injection aborts as designed | allocator-level cleanliness (M6 basis); not absence of all conceivable residue |
| W07-DV03 → P4-V10 | No residue: global/CPU state | structural review + retained-state audit | review driver/episode objects; enumerate cross-episode survivors | only the documented W04 frame/diagnostic retention survives; no statics | structural no-residue property; not a proof over future code changes |
| W07-DV04 → P4-V11 | Cold-boot consistency | W08-executed repeats compared via P4-RR records | ≥ 5 boots of one declared build; record-vs-record comparison | identical S1–S5 surfaces and determinate verdicts per boot, within the declared environment | declared-environment consistency; not hardware behavior or cross-build stability |
| W07-DV05 | Cleanup/initialization determinism | host-side digest/equality tests + on-target spot check | re-init equivalence; boot-info determinism (W03 contract) | `RamReuse` digest == `FullRebuild` digest; fields stable | deterministic initialization as measured; not cryptographically strong integrity |
| W07-DV06 → P4-V12 | Event categories observable | ledger unit tests + on-target run | exercise every §1 category; read vectors/summary | all P4-V12 categories counted and queryable | observability of the required categories; not a production metrics system |
| W07-DV07 → P4-V12 | Counts, patterns, and fault correlation | on-target episode records vs [04 §5](04-code-contracts-telemetry.md) patterns | per-episode vectors + correlation lines | every episode satisfies its pattern; correlation fields complete for every fault | counts/correlation correctness; not performance measurement |
| W07-DV08 | Determinism comparisons sound | host-side tests (digest known answers, prefix rule, modulo rule) | mutate one surface; verify detection with the right surface named | divergences detected and attributed; equal runs report `Stable` | comparison logic; not that future surfaces cannot be added |
| W07-DV09 | Run-record grammar coherence | joint review with W08 + golden tests | P4-RR v1 fields vs W08 manifest expectations | W08 parses without invention; version rejection works for unknown versions | automation seam stability; not a frozen API (W09 records facts) |
| W07-DV10 | No-unsafe and layering review | static review | audit W07 surface vs inventory; module layering | zero new `unsafe`; no register/board/QEMU names; no second event emission | controlled surface; not functional correctness |

Record each row as **passed / failed / blocked / not run** with command or
review input, environment, date, and reason. On-target rows prove the QEMU
reference environment only, never real-hardware semantics (W01 A7). Rows
DV01–DV04 and DV06–DV07 are blocked until the M2–M6 upstream evidence
exists; blocked rows name the waiting W01 row.

## 2. Error model

- Guest-caused conditions reach W07 only as outcomes (`stop_cause`,
  `match_verdict`, marker echoes) — never as errors; the W01 A2 type split
  is preserved end to end.
- Host-authored contract violations (teardown-depth violation, missing
  retained RAM, second driver) escalate through the P0 failure
  classification as fatal invariants — they indicate broken coordination,
  not Guest behavior.
- Capacity/infrastructure failures (`LedgerOverflow`, `RecordError::
  BufferTooSmall`, emission failure) surface and degrade the run verdict
  honestly (missing evidence is a non-success), never silently truncate.
- Upstream signature/accounting gaps are blocked prerequisites recorded per
  W01 §4, with the degraded evidence basis named.

## 3. Security model

- The driver reads Guest RAM only through the W03 host view for digesting;
  Guest content influences nothing except recorded equality verdicts.
- The run record and correlation records carry no Guest data content beyond
  diagnosed addresses/classifications; they are host-authored console output,
  never a Guest-writable surface.
- W07 adds no `unsafe`, no Guest-reachable interface, and no privileged
  operation; it coordinates already-audited seams.
- Standing scope boundaries: no telemetry transport or backend, no scheduler
  statistics (P7), no cross-pCPU shootdown proof, no fault-tolerant
  orchestration — pressure to add any here is a stage-boundary violation to
  record.

## 4. Observability model

- W07's own events (`run.*`) and the P4-RR v1 record route through the P0
  baseline with build identity (W01 A8); the record is the single
  machine-readable summary W08 parses.
- Counters derive only from the sibling event inventory
  ([04 §1](04-code-contracts-telemetry.md)); W07 creates no second emission
  point, so observability cannot drift from behavior.
- Verification claims live only in
  `../../verification/p4-w07-repeatability-telemetry-verification.md`;
  design documents and implementation records carry no run evidence.

## 5. Handoff checklist

Before handing W07 work to a reviewer:

- exact changed-file list and implementation-record path
  (`../p4-w07-repeatability-telemetry-record.md`);
- DV01–DV10 statuses with explicit not-run/blocked entries and the waiting
  upstream rows (M1–M6) each blocked item waits on;
- confirmation: zero new `unsafe`; no changes to W03 sequencing, W04 stop
  path, W06 diagnosis semantics, or W05 scenario bodies/markers;
- recorded decisions and versions: teardown depths delivered, event
  inventory as counted, grammar `P4-RR-1`, repeat minimums, open item O2
  (per-build-identity stability) status;
- handoff to consumers: grammar + minimums + count patterns to
  [P4-W08](../p4-w08-qemu-integration-regression/README.md); factual
  observability baseline and limitations to
  [P4-W09](../p4-w09-closeout-p5-handoff/README.md);
- open items carried forward: O1 grammar acknowledgment, O2 cross-build
  stability boundary, P2-ACR-01 unchanged and unresolved.
