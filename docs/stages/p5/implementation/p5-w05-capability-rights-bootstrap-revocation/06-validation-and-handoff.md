# P5-W05 Validation, Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md).

## 1. Preconditions for judging acceptance

Evidence lives only in the verification record. Host-seam evidence is
package-local; guest-path and two-context QEMU evidence requires AC-05.3,
W04 integration, and
[P5-W07](../p5-w07-validation-guest-isolation-suite/README.md) assets, and
is recorded **blocked** until then.

## 2. Validation matrix

| ID | Requirement → source | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W05-DV01 | Caller-associated record model; check correctness → P5-T07, P5-V06 | host unit matrix | valid use; no record; wrong subject with identical handle value; destroyed-then-recreated target | Ok only for an Active record matching subject + exact binding + rights; purity property holds | the check semantics; not guest-path integration |
| W05-DV02 | Rights classes and distinguishable denials → P5-T08/T10, P5-V06 | host unit tests | per-class required sets; single and multi-bit shortfalls; NO_AUTHORITY vs INSUFFICIENT_RIGHTS class outcomes via the single conversion | each shortfall yields INSUFFICIENT_RIGHTS; absence/revocation/wrong-subject uniformly NO_AUTHORITY; causes never Guest-visible | denial vocabulary; not the minimal operation's behavior (W06) |
| W05-DV03 | Bootstrap-only explicit grants; no identity shortcut → P5-T09/T11, P5-V07 | structural review + negative tests | confirm grant is not Guest-reachable; probe matrix: first-VM subject, VM-ID-0-equivalent synthetic identity, role-free identities — none authorize without a record | no path authorizes without an explicit record; grant is internal-only | the no-shortcut rule at this boundary; full V07 evidence completes with W07 guest scenarios |
| W05-DV04 | Grant / use / revoke / reject-old loop → P5-T12, P5-V08 | host loop test | grant → authorize Ok → revoke → authorize denied (Revoked cause internally) → revoke again (AlreadyRevoked) → re-grant → fresh authority works | the full loop behaves as specified; tombstone retained until reclaim | basic revocation semantics; not delegation trees or attenuation (Reserved) |
| W05-DV05 | Two-context isolation basis → P5-V12 (via W07) | host two-subject exercise + review | two synthetic subjects; subject B presents subject A's handle value; destroy/recreate generations | B is denied; A unaffected; generation change neuters A's records too | isolation-by-construction at the model level; QEMU two-context evidence is W07's |
| W05-DV06 | Multi-pCPU/concurrency soundness → P5-V14 basis | concurrency review/test | revoke-vs-authorize linearization; reclaim under destroyed events; never-nested lock rule audit | linearized outcomes only; no deadlock by construction (rule audit passes); stress invariants hold | the designed concurrency boundary; not scalability (Reserved) |
| W05-DV07 | Consumer consumability → W05 closure | consumer walkthrough | read the record as W06 (ordering + required-rights declaration clear?), W07 (hooks + observable outcomes sufficient?), W08 (seam + invariants stated?), W09/W10 (event vocabulary + redaction inputs?) | each consumer can act without redefining this boundary | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, environment, timestamp, and reason. Guest-path and
two-context QEMU rows are blocked until prerequisites exist. No row proves
P5-V09–V10 (integrated dispatch, W06), and none may be reported as doing
so.

## 3. Error, security, and observability model

- **Error model:** two Guest-visible denial classes (`NO_AUTHORITY`,
  `INSUFFICIENT_RIGHTS`) with three internal causes; capacity is
  `RESOURCE_EXHAUSTED`; one-way revocation with tombstones; invariants
  escalate through the W02 invariant model. No operation leaves partial
  state.
- **Security model:** authority exists only as explicit bootstrap-created
  records bound to an exact target generation; identity, roles, first-VM
  status, and pCPU placement have no representable influence; two-context
  isolation falls out of subject keying; revocation is one-way and survives
  target recreation; Guest-visible denial is uniform (no probing oracle
  beyond the two classes); secrets and authentication are absent by scope.
- **Observability model:** granted / revoked / denied-cause events with
  class granularity; no subject values, handle values, or record counts in
  Guest-visible outputs; W09 owns transport and redaction. The P5-V08 loop
  (revoked-use attempts observable) is preserved through internal causes.
- **Known limitations carried forward:** Delegate right defined but
  unimplemented (no delegation in v0); single VM-level caller granularity
  (no per-vCPU trust separation); fixed record capacity; exact-binding scan
  (matching shortcuts Reserved with W08-evidence trigger).

## 4. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list, including any new `unsafe` with SAFETY
  comments and P0-W10 inventory entry (expected: none — safe Rust; any
  exception needs explicit justification);
- the fixed record-capacity value with rationale, and confirmation that
  DELEGATE is never granted in v0;
- DV01–DV07 evidence paths with run status, including explicit
  blocked/not-run entries (guest-path caller identity, W04 hook wiring,
  two-context QEMU rows);
- confirmation that no delegation, attenuation, guest-initiated
  grant/revoke, authentication, RBAC, or identity-valued logic was
  implemented, that W04 and W05 locks are never nested, and that the check
  has no default-allow path;
- handoff artifacts per consumer: W06 (authorize contract, mandatory
  required-rights parameter, sequencing and lock-order rule), W07 (bootstrap
  hooks, observable outcomes, internal-cause events), W08 (stress seam and
  invariants), W09/W10 (authority-event vocabulary); and
- open items: sibling `Contract Conflict` status with W02/W04 if any,
  AC-05.3 binding status — none resolved inside W05.
