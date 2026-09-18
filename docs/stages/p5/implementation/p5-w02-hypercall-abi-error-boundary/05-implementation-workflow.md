# P5-W02 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before editing, the implementer completes the Coding-Guidelines preflight and
verifies the W01 entry review exists and names the inputs this design assumes
([01](01-scope-and-foundations.md) §2). Re-check the assumed-contract status
at implementation entry: AC-02.1 (trap path), AC-02.2 (context/preservation),
AC-02.3 (fatal classification), AC-02.4 (host tests), AC-02.5 (event
namespace), AC-02.6 (ABI location).

Stop and obtain direction instead of guessing when any of the following
occurs:

- an assumed contract is unevidenced at the point the dependent step needs it
  (host tests for step 5, trap integration for step 6) — mark the dependent
  evidence **blocked**, implement nothing in its place, and record the block;
- the implemented P1/P4 exception path lacks an HVC-class extension seam —
  `Contract Conflict` against P4-W06; do not fork the vector path locally;
- satisfying a check seems to require authority, identity, handle, or
  Guest-data semantics — those are W03–W06 scope; stop the step and record;
- a reviewer requests a status-code, register, or version change after Guest-
  visible code exists — route per
  [06](06-validation-and-handoff.md) §3 (compatibility event), never patch
  silently;
- the fatal path (AC-02.3) is absent and an invariant escalation must be
  testable — implement the type-level separation (it is independent), leave
  the escalation *target* wired to the P0-W14 boundary as a marked
  integration point, and record the dependency.

## 2. Ordered implementation steps

### Step 1 — fix the ABI surface constants and types

Target: Core ABI-surface module (M2–M5 types) and the Arch frame type (M1).

Work: encode [03](03-code-contracts-abi-surface.md) §1 constants, the status
taxonomy ([04](04-code-contracts-error-boundary.md) §1–§2), `HypercallFrame`,
`DecodedCall`, `AbiVersionWord`, `CallDescriptor` as types with their
invariants. No handlers yet. Where constants are Guest-visible, keep one
source of truth host and Guest code can both cite.

Suggested observation: the host-side build compiles with no `unsafe`
introduced.

**Acceptance:** every constant matches this design value-for-value; types
enforce their invariants at construction; no Guest-visible vocabulary beyond
§1/§2 exists.  
**Failure/blocker:** a required constant not fixed by this design is a gap —
stop and amend the design through review, do not improvise a value.

### Step 2 — implement decode, version, and registry

Target: M2 `decode_envelope`, M4 version functions, M3 registry.

Work: implement per [03](03-code-contracts-abi-surface.md) §3–§5, keeping
each total, pure, and allocation-free. The registry contains the discovery
descriptor and a `CALL_MINIMAL` descriptor whose handler entry is an explicit
unimplemented slot reserved for
[P5-W06](../p5-w06-dispatch-permission-containment/README.md) — reaching it
in dispatch is an invariant, never a `todo!()` reachable from Guest input.

**Acceptance:** functions are total on all `u64` inputs; the minimal-call
slot is not callable from any Guest-reachable path in this package.  
**Failure/blocker:** any temptation to interpret arguments, consult
identity, or touch memory inside decode is a design violation — stop.

### Step 3 — implement the error domain and composition

Target: M5 error types and M6 `compose_result`.

Work: implement `GuestBoundaryError`, the single conversion, the
`InvariantViolation` type with no status conversion, and `compose_result`
with reserved-result zeroing ([03 §7](03-code-contracts-abi-surface.md),
[04](04-code-contracts-error-boundary.md) §3, §5). The escalation target is
the P0-W14 fatal path; if that is not yet integrated, escalate to a marked
Arch-level integration point and record AC-02.3 as a live dependency.

**Acceptance:** exactly one status-construction site exists; an invariant
value cannot be named where a status is expected (type error).  
**Failure/blocker:** a second mapping site or a "convenient" conversion from
invariant to status is a review stop (Decision 7).

### Step 4 — wire the Arch trap adapter to the exception path

Target: M1, consuming the P1/P4 synchronous-exception classification.

Work: integrate HVC-class recognition for Guest-EL1 origin, frame capture,
and the result view used by composition. Other exception classes and origins
remain on the P1/P4 path untouched. This is the package's only Arch
integration; any `unsafe` here follows the P0-W10 governance (SAFETY
comments, inventory entry).

**Acceptance:** a real Guest HVC trap produces a frame with the Guest's
register values and the ESR immediate; non-HVC traps behave exactly as before
the change.  
**Failure/blocker:** AC-02.1/AC-02.2 not evidenced — mark QEMU-dependent
acceptance blocked, complete host-side work, record the block. Behavior
differences in P1/P4 paths are a `Contract Conflict`, never absorbed.

### Step 5 — build the host-side validation seam and tests

Target: host test module (P0-W07/W08 assumed gates).

Work: unit tests for every contract in [03](03-code-contracts-abi-surface.md)
and [04](04-code-contracts-error-boundary.md): envelope cases (bad immediate,
junk halves, nonzero reserved, zero/`u32::MAX` boundaries), version matrix,
registry known/unknown/gated, status table bijection, composition zeroing,
conversion totality. Add the property harness seam (decode + classification
as pure functions over synthetic frames) for
[P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md) consumption.

**Acceptance:** every W02 contract has at least one focused test; the fuzz
seam is callable without trap state or a Guest.  
**Failure/blocker:** host gates unavailable (AC-02.4) — tests are still
written; their execution is recorded blocked.

### Step 6 — prepare the integration and documentation handoffs

Target: implementation record; compatibility-analysis skeleton.

Work: (a) write the dispatch-flow integration contract for W06 (the S1–S8
pipeline of [02 §3](02-architecture-and-state.md) with the S6 slot and its
required check ordering left to W06) into the implementation record; (b)
draft the factual-ABI compatibility-analysis skeleton (what changed vs. any
prior internal state, the compatibility commitment class at major 0, the
review route) as input to the `docs/abi/` artifact — the artifact itself is
not written here. Emit the telemetry vocabulary (04 §4) as defined constants
for W09 without transport wiring.

**Acceptance:** W06 can read the record and state its integration without
asking for values this design should have fixed; the skeleton names every
Guest-visible value and its change policy.  
**Failure/blocker:** any need to fix W06's internal ordering here is scope
leak — record it as a handoff note instead.

### Step 7 — closure review

Work: run the validation matrix in
[06](06-validation-and-handoff.md) §2, confirm the handoff checklist, and
record evidence in
`../../verification/p5-w02-hypercall-abi-error-boundary-verification.md` and
decisions/deviations in
`../p5-w02-hypercall-abi-error-boundary-record.md`. Completion is claimed
only there, only for what actually ran.

## 3. Evidence destinations

| Artifact | Path |
|---|---|
| Implementation record (decisions taken, deviations, changed files, `unsafe` delta) | `../p5-w02-hypercall-abi-error-boundary-record.md` |
| Verification record (commands, environments, results, run/not-run) | `../../verification/p5-w02-hypercall-abi-error-boundary-verification.md` |
| Factual ABI document (later, evidence-gated) | `docs/abi/` per [W01 routing](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md) §4 |
