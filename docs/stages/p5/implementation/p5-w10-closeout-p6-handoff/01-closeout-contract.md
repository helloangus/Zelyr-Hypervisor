# P5-W10 Closeout Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W10 detailed design](README.md).

## 1. Artifact groups and ownership

| Artifact group | Authoritative owner | Content source | Non-responsibility |
|---|---|---|---|
| Factual HVC ABI artifact | implementing agent of the W02 boundary, published through the §3 gate | W02 design + implementation + compatibility analysis | not a public compatibility promise; not authored by W10 |
| Security / input-safety factual document | implementing agents of W03/W06 (Guest-data and containment facts), W09 (redaction rules) | their records + verification evidence | not a threat model or security claim beyond compiled evidence |
| Handle / capability factual document | implementing agents of W04/W05 | their records + verification evidence | not an API reference; encodings stay in the ABI artifact |
| P5 closeout record (`p5-w10-closeout-p6-handoff-record.md`) | W10 | all W01–W09 records | no new technical decisions; no command logs (verification holds those) |
| P5 verification record (closure review) | W10 | actual review runs | no closure claim beyond evidenced items |
| Link index into `docs/abi/` and `docs/security/` | W10, per W01's routing | published artifacts | does not restate artifact content |

Each artifact named in the third column of the plan's scope (P5-D01–D09
family) maps to one of these groups; nothing outside them is created by
W10. The publication locations follow W01's routing deliverable; where W01
recorded no routing for an artifact, publishing it is blocked pending that
routing — not decided locally.

## 2. Factual-content minimums

Every factual document and record section must state, where applicable:

- **What was implemented** — mechanisms, their owning packages, and the
  established module boundaries they live in;
- **What was verified** — the validation rows, environments, and run
  statuses, linked to verification records;
- **What was deliberately not implemented** — Reserved and Out-of-Scope
  items from the task book plus package-recorded limitations;
- **Commitment level** — for every published boundary: experimental
  internal convention, internal compatibility commitment, or (only if an
  authorized decision exists) public contract. P5 expects the first two
  and no third.

A statement without both a linked implementation record and linked
verification evidence is prohibited in every group above.

## 3. Publication gate (ABI and security artifacts)

An artifact may be published (placed in its governed location and linked)
only when all of the following hold; each is checked per artifact and
recorded:

1. the behavior it describes is implemented and recorded by its owning
   package;
2. the applicable compatibility analysis exists (for the ABI artifact:
   version/compatibility semantics per the W02 design, including
   unknown-call, version-mismatch, and reserved-field behavior as
   implemented);
3. verification evidence exists with run status for the described
   behavior;
4. the artifact marks its commitment level per §2 and, for the ABI
   artifact, distinguishes the experimental internal commitment from any
   later public contract.

Failure of any check parks the artifact as "implemented but not
publishable (reason)" in the closeout record. Parking is a normal,
recorded outcome — never a silent omission and never a reason to weaken
the artifact's content.

## 4. Invariant inventory reconciliation

For each of INV-P5-01 through INV-P5-10 (task book §6), the closeout
record contains one reconciliation row:

```text
INV-P5-nn  <task-book invariant>
  enforced by:   <packages / mechanisms>
  evidence:      <verification rows, e.g., W07-DVxx, W06-DVxx, with status>
  status:        evidenced | partially evidenced (gap stated) | not implemented
  deferrals:     <recorded deferrals or architecture-change references, or none>
```

The reconciliation must also list the regression rows (W09's set) that
continuously re-check each invariant, so the inventory stays verifiable
after closure. A `partially evidenced` or `not implemented` status is an
expected possible outcome and does not block recording — it blocks closure
claims that depend on it, per
[02](02-closeout-workflow-and-validation.md) step 6.

## 5. Factual records: environment, deltas, limitations, regression inventory

- **Environment and execution facts:** the declared test environment per
  package (QEMU version/config, pCPU counts), negative/stress durations
  and bounds (W08's recorded configurations), and the performance record
  reference (W08's method, raw data, and the QEMU-not-hardware statement —
  summarized, never re-quoted as a target).
- **Unsafe delta:** the aggregate `unsafe` inventory delta for P5, compiled
  from the packages' reports against the P0 unsafe-governance inventory
  ([P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md)); expected
  location per the designs: concentrated in delivered low-level boundaries,
  none in W06 pipeline logic, W07 scenarios, W08 harness, or W09
  telemetry. Any deviation is recorded as reported by the owning package.
- **Dependencies:** the aggregate dependency delta with each entry's
  governance reference ([P0-W18](../../../p0/plans/p0-w18-dependency-governance.md)).
- **Known limitations:** at minimum — the dispatch integrates one minimal
  permitted operation (no general management surface); markers, scenario
  constants, and test-bootstrap values are test conventions, not ABI; all
  behavior and timing evidence is QEMU-scoped; the authority model proves
  grant/check/revoke only (no delegation); multi-pCPU evidence is the
  declared two-pCPU set; plus every limitation recorded by W01–W09.
- **Regression inventory:** W09's frozen row-list version, its verdict
  summary, and the extension/weakening rules later stages must follow.
- **Open decisions:** every `Architecture Change Request` / `ADR Required`
  / blocked-prerequisite record left open by W01–W09, with owner and
  status — closure does not resolve them.

## 6. P6 consumer map

The [P6 task book](../../../p6/task-book-v0.1.md) is a named consumer only;
its semantics are not designed here. Each row maps a P6 need to the P5
deliverable it may consume, by owning package and record — P6 reads the
owning record, never a closeout paraphrase.

| P6 consumer (by package ID) | P6 need (task-book wording) | P5 deliverable it may consume | Owner record |
|---|---|---|---|
| P6-W01 (entry reconciliation) | P6-ENTRY-07: versioned HVC/error behavior | the published factual HVC ABI artifact (if gated through [01 §3](#3-publication-gate-abi-and-security-artifacts)) or its parked status | W02 design + implementation record |
| P6-W01 | P6-ENTRY-07: handle/capability checks | handle/capability factual document + W06-DV02 evidence | W04/W05 records |
| P6-W01 | P6-ENTRY-07: guest-safe access | Guest-data boundary facts + W06-DV04 address-class evidence | W03/W06 records |
| P6-W07/W08 (vIRQ core, virtualization interface) | introduce interrupt/virtual-IRQ objects "through this foundation" | object-reference extensibility (W04) and caller-associated authority (W05) as evidenced; the containment category model (W06) | W04/W05/W06 records |
| P6-W12 (robustness) | invalid requests contained and diagnosable | W06 containment table + W08 oracle/limit records as method precedent | W06/W08 records |
| P6-W13 (telemetry/regression) | P0–P5 regression (P6-V23); telemetry constraints | W09's regression boundary (row list, non-success classes, environment constraints) and the redaction-rule reference | W09 records |
| P6-W11 (Validation Guest suite) | maintained Guest asset pattern | the maintained-asset and marker-discipline rules as method precedent (not the markers themselves) | W07 records |
| P6-W13 (latency baseline) | performance-baseline precedent | W08's baseline method and non-KPI rules (no numbers inherited as targets) | W08 record |

Explicit non-deliverables to P6 (restating the task book §7): no frozen
management ABI, no machine ABI, no Control Domain, no IPC, no delegation
tree, no scheduler policy, no final concurrent algorithm, and no
real-hardware correctness claim.

## 7. What this contract does not authorize

No authoring of technical content owned by W02–W09; no publication into
locations W01 has not routed; no closure language anywhere except the
verification record's evidence-bound review; no edits to upstream records
(corrections are requested from the owning package); no P6 design work.
Any temptation beyond these boundaries is a scope violation to stop at
review.
