# P5-W06 Containment and Error Classification

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W06 detailed design](README.md).

## 1. Failure classification model

Every failure observable on the dispatch path is classified by source before
any response is chosen. The classes follow the project error-taxonomy
convention (Plan-Agent detailed reference §55, §93) and the P0-W14
panic-classification contract
([plan](../../../p0/plans/p0-w14-panic-failure-classification.md)):

| Source class | Examples on this path | Default response |
|---|---|---|
| Guest error | malformed request, unknown call, unsupported feature, version mismatch, invalid/stale/wrong-type reference, bad state, invalid Guest address or range | structured `Denied` outcome to the calling Guest |
| Caller authority error | no capability, insufficient right class, revoked capability | structured `Denied` outcome (`NoAuthority`, `Revoked`) |
| Resource condition | no capacity to service the call (buffer too large for the declared maximum, dispatch-path allocation failure) | contained `Denied(ResourceExhausted)` outcome plus a diagnostic event |
| Hardware failure | (not exercisable in P5 scope; reserved for later stages) | declared by its owning future design |
| Hypervisor invariant violation | corrupted object table, impossible pipeline state, boundary contract returns an undefined value | fatal path (§3); never a Guest outcome |

Two prohibitions define the model:

- **No upward reclassification.** A Guest-caused condition must never become
  a Hypervisor panic or fatal. The P5-W06 plan lists reclassifying a genuine
  invariant violation as a Guest error in its out-of-scope set — and the
  inverse is equally prohibited by the task book.
- **No silent widening.** A condition the boundaries report as controlled
  must reach the Guest with its distinct class intact; P5-V06 requires the
  classes to remain distinguishable at the Guest-visible boundary.

## 2. Containment mapping

The mapping from pipeline stage ([01 §3](01-dispatch-pipeline-contract.md))
to outcome class is total. This table is the stable scenario-outcome
reference handed to W07 (Guest-observable expectations), W08 (host-side
oracle), and W09 (categories).

| Stage | Condition | Outcome class | Guest-observable result |
|---|---|---|---|
| S1 DECODE | call number not defined | `UnknownCall` | structured denial |
| S1 DECODE | defined call, feature not present/enabled | `UnsupportedFeature` | structured denial |
| S1 DECODE | version incompatible with the boundary | `VersionMismatch` | structured denial |
| S1 DECODE | structure/reserved fields/length malformed | `Malformed` | structured denial |
| S2 REFERENCE | handle value not a live reference | `InvalidReference` | structured denial |
| S2 REFERENCE | stale after destroy or slot reuse | `InvalidReference` | structured denial |
| S2 REFERENCE | wrong object class for the operation | `WrongType` | structured denial |
| S4 AUTHORITY | caller holds no capability for the object | `NoAuthority` | structured denial |
| S4 AUTHORITY | capability lacks the required right class | `NoAuthority` | structured denial |
| S4 AUTHORITY | capability revoked since grant | `Revoked` | structured denial |
| S5 ARGUMENTS | Guest range unmapped, overflowing, wrong direction, bad type | `InvalidAddress` | structured denial |
| S5 ARGUMENTS | Guest-data access faults during copy per the W03 contract | `GuestDataFault` | controlled outcome per W03 (may be a Guest-visible fault) |
| S5 ARGUMENTS | result buffer exceeds declared maximum | `ResourceExhausted` | structured denial |
| S6 STATE | object/lifecycle state does not permit the operation | `BadState` | structured denial |
| S5/S7 | dispatch-path allocation or capacity failure | `ResourceExhausted` | structured denial + diagnostic event |
| any | boundary returns a value outside its contract | invariant path (§3) | none — not Guest-facing |

Rules attached to the mapping:

- A denial at S1–S4 must leave Guest memory untouched: no stage may write a
  result buffer before S5 validation and S7 execution succeed. Partial
  writes must not leak which later stage would have failed.
- Distinguishability is required, not hidden: P5-V06 requires invalid
  object, absent authority, insufficient right, unsupported, bad-state, and
  resource conditions to be distinguishable controlled outcomes. No
  constant-time-style class merging is applied in P5; if a later stage
  requires it, that is a new security decision.
- `ResourceExhausted` is deterministic and clean (Plan-Agent detailed
  reference §57): the call fails, nothing is half-allocated, and retrying
  after capacity is freed is defined behavior.

## 3. Invariant boundary (the fatal path)

The fatal path exists only for the last table row. It is entered when an
internal invariant fails: an object-table or capability-state impossible
value, a boundary contract violation, a pipeline state that cannot occur, or
a Guest outcome that S1–S6 should have made unreachable. Its behavior is:

1. classify and record the invariant violation per the P0-W14
   panic-classification contract, with the diagnosable context the P1 crash
   path provides (exception state, exit reason, dispatch stage);
2. never encode a Guest result that would present the event as a normal
   Guest-caused denial;
3. never continue dispatch on the failing context.

Guest input alone must not reach this path. The review that proves this is
W06-DV05/DV06 in [03](03-implementation-workflow-and-validation.md): every
denial-class scenario must terminate in the Guest-facing outcome, and the
fatal path must be reachable only through injected internal faults (host-side
tests), never through any Guest-controlled value.

## 4. Result-category taxonomy (handoff to W09)

Every dispatched call yields exactly one category, derived from its outcome.
W09 registers and counts these under the P0-W13 namespace rules and the
P0-W12 visibility rules; W06 only guarantees the category exists and is
assigned uniformly.

| Category | Derived from | Notes |
|---|---|---|
| `completed` | `Completed` | per-VM success accounting |
| `denied-structural` | `UnknownCall`, `UnsupportedFeature`, `VersionMismatch`, `Malformed` | request never reached object state |
| `denied-reference` | `InvalidReference`, `WrongType` | reference boundary rejections |
| `denied-authority` | `NoAuthority`, `Revoked` | authority boundary rejections |
| `denied-address` | `InvalidAddress`, `GuestDataFault` | Guest-data boundary rejections |
| `denied-state` | `BadState` | lifecycle rejections |
| `resource` | `ResourceExhausted` | contained resource conditions |
| `invariant` | fatal path entry | expected zero in all Guest-facing evidence |

Attributes carried with each category for attribution (values only; never
Host pointers, never Guest buffer contents): calling VM/security-context
identifier, operation identity, outcome class, dispatch stage reached.
Counter loss or telemetry failure must not alter any Guest-visible outcome
(observability must not change core semantics).

## 5. Error, security, and observability summary for reviewers

- The security property this module owns: **authorization precedes every
  state interaction**, and **Guest-caused input is contained to a structured
  result**. Both are reviewable in code as the stage order plus the total
  mapping of §2.
- The diagnosability property: invariant failures remain a separately
  classified, separately recorded path (§3), so containment cannot mask a
  real Hypervisor defect.
- The observability property: the category of §4 exists for every call,
  enabling W09's per-VM accounting without new dispatch logic.
