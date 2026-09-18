# P7-W09 Validation Matrix, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W09 detailed design](README.md).  
All rows are planned evidence with objective conditions; none claims that a
test ran. Results go only to
`../../verification/p7-w09-accounting-diagnostics-verification.md`.

## 1. Validation matrix

| ID | Maps to | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W09-DV01 | P7-V19 | Counter coherence under event replay | Replay randomized hook histories (all producers) against the records | Every accumulator matches the history; monotonic, no regress, no double-count; invariant breaches surface per P0-W14 | Record/update-rule correctness; not that producers fire hooks correctly everywhere (their matrices cover that) |
| W09-DV02 | P7-V19 | Single-writer review | Search all update sites; compare against A-2..A-5 | Zero out-of-contract update sites; one writer class per field | Structural coherence; not runtime freedom from future regressions |
| W09-DV03 | P7-V19 | Aggregation consistency | Compare computed VmSummary against member records under concurrent activity | Aggregation equals the member census within declared staleness; never stored | Aggregation semantics; not VM-level performance claims |
| W09-DV04 | P7-V20 | Reason distinguishability | Drive one switch of each `SwitchReason`; read records and trace | Each reason is separately countable and visible in the trace with identity fields (VM/vCPU/pCPU/time) | Vocabulary coverage of F-4 dispositions; not scheduling policy correctness |
| W09-DV05 | P7-V20 | Trace field completeness | Inspect every C-2 emission for required context | VM/vCPU/pCPU/reason/time (and source where defined) present on every event | Correlation capability; not transport delivery guarantees (P0-W12 owns those) |
| W09-DV06 | P7-V21 | Diagnostic completeness | Fault-inject each C-4 invocation site; capture the report | Report shows current pCPU identity, VM/vCPU, state, placement summary, reason, recent transitions, pending events, markers, counters — in fixed order | Diagnostic content per P7-V21; explicitly NOT a crash-dump capability |
| W09-DV07 | P7-V21 | Determinism and safety of reports | Run the same failure twice; diff outputs; enumerate field sources | Byte-identical reports for identical state (modulo declared time values); zero Guest-controlled bytes; no Host pointers | Determinism and safety; not full logging-system security (T-1/T-6 boundary) |
| W09-DV08 | P0 compatibility | Governance registration review | Check C-5 registration outcome against P0-W13 rules and C-4 against P0-W12 minimums | All events registered with domain/names/fields/trimming; report shape compliant | Governance compatibility; not the P0 contracts themselves |
| W09-DV09 | P7-V04 clause | No accounting regression under stress | Run W11-style stress with counters enabled (cooperative preview) | Counters stay coherent under storm density; trimming does not affect record truth | Record robustness under load; full stress evidence is P7-W11's (P7-V24–V27) |
| W09-DV10 | W09 closure | Scope-exclusion review | Inspect the delta for transports, backends, encodings, management surfaces, KPIs | None present | Boundary discipline; not downstream correctness |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. W09-DV01–DV05 and
DV09 require the producer packages' running scheduler; without them they are
recorded not run with the owning package named. No row proves performance
conclusions (P7-W13, P7-V29 owns the method and its limits), stress
properties (P7-W11), or any P8 behavior; none may be reported as doing so.

## 2. Error, security, and observability model

- **Errors.** W09 adds no new failure classes to the hypervisor: record or
  emission anomalies (accumulator regression, unregistered event, report
  channel loss) are observability defects surfaced per the P0-W14 boundary
  or recorded as degraded (silent renderer completion), never secondary
  panics. Trace delivery loss is declared by the P0-W12 channel; records
  remain authoritative, so diagnostics never depend on trace delivery.
- **Security.** All record, event, and report content is identifiers,
  enumerations, and monotonic values; no Guest-controlled bytes, strings, or
  raw Host pointers (decision 7, DV07). A Guest can influence *when*
  diagnostics fire (by causing contained faults) but not *what* they contain;
  flooding is bounded by the fixed-capacity snapshot and channel trimming.
  This design adds no authority surface: no capability checks are performed
  or needed.
- **Observability.** Self-application: the observability module is itself
  observable — its own defects surface through the same P0-W14 boundary —
  and its outputs are the P7-V19–V21 evidence instruments for W11/W13/W14.
  Every output carries the P0-W12 version-identity association point;
  release builds retain the minimum set (decision 8) so production failures
  remain diagnosable without a crash-dump framework.

## 3. Handoff checklist

Before handing W09 to a reviewer and to consumers:

- Changed-file list respecting the module boundary of
  [01-accounting-model.md](01-accounting-model.md) §1 (no producer
  transition logic modified; hook bodies only).
- Assumed-contract and hook-inventory status from workflow step 1, including
  any producer-hook gaps and blocked prerequisites.
- W09-DV01–DV10 status with evidence paths and explicit not-run entries
  (naming which rows await producer baselines).
- Confirmation: no new `unsafe`, no ABI/public API, no new dependency, no
  transport/backend/encoding, no management surface, no crash-dump claim
  anywhere in documentation or code comments.
- Seam inventory handed on: W11 (counters/reasons as stress instruments and
  coherence rules its checks must preserve), W13 (records + trace as the
  sole baseline data source, with declared limits), W14 (observability
  contract, governance registration status, diagnostic boundaries for
  closeout and P8 handoff), W12 (determinate counter/trace assertions).
- Open items for consumers (not resolved here): W11 owns stress evidence;
  W13 owns the performance method and KPI-free baseline; the steal-time ABI
  remains P8+/machine-ABI territory.
- Record locations: implementation notes to
  `../p7-w09-accounting-diagnostics-record.md`; evidence to
  `../../verification/p7-w09-accounting-diagnostics-verification.md` — both
  created only when work or evidence exists.
