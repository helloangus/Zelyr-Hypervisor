# P3-W03 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

## 1. Validation matrix

Each item is recorded as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason in the
verification record. QEMU items appear only where W03's scope produces
them; the repeated matrix and stress evidence belong to
[P3-W13](../p3-w13-qemu-smp-regression/README.md) and
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md).

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W03-DV01 | Vocabulary totality and reserved-state declaration (→ P3-V03) | encoding review + round-trip tests | unit tests over all eight states; review unreachable-edge declaration | every state encodes/decodes; unknown encodings rejected; Offline/Stopping/Suspended have no P3 edges | vocabulary integrity; not runtime behavior |
| W03-DV02 | Transition table correctness (→ P3-V03) | table sweep tests | legal-edge/illegal-edge unit tests over the full matrix; refusal carries observed state | every legal edge accepted exactly once; every other edge refused without mutation; events emitted per acceptance | state-machine logic; not that firmware/CPU flows hit it correctly (W02/W05's evidence) |
| W03-DV03 | Exactly-once online admission (→ P3-V03) | admission tests + review | sequential double-admission test via the seam; review the fatal contention path | first admission succeeds; second attempt detected and fatal; non-Initializing sources refused | the "never twice online" invariant at the logic level; not true parallel contention (W12's stress boundary, recorded) |
| W03-DV04 | Gate and online-set integrity (→ P3-V03) | gate matrix tests | unit tests over all states × eligibility queries; snapshot consistency checks | Failed/never-initialized/excluded CPUs are structurally un-usable per the gate; snapshot matches scan | "no failed-CPU use / no use before init" at the gate level; not consumer compliance (consumers' reviews) |
| W03-DV05 | Lifecycle observability (→ P3-V03, P3-V11 interface) | QEMU capture + dump review | SMP-ready state dump on an all-online boot and a failed-start boot (W02 absent-CPU input) | dump shows every attempted CPU Online or Failed, failed outside the online set, causes attributed; transition events present | end-to-end lifecycle authority on the reference platform per captured boot; not repeated boots (W13), not hardware platforms |
| W03-DV06 | State contract recorded; consumers integrated (→ W03 closure) | closure review | verify implementation record, handoff checklist, consumer contract confirmations | no deviation unrecorded; W04/W05/W07/W09/W10–W14 surfaces confirmed; no completion claim | handoff readiness; not downstream correctness |

A passing gate test does not prove consumers call the gate; that is each
consumer's review obligation ([P3-W10](../p3-w10-smp-safety-audit/README.md)
audits it for P0–P2-era code, consumers' designs own their call sites).
QEMU dumps never prove hardware-platform lifecycle behavior.

## 2. Error, security, and observability model

- **Error model.** Refusals (`TransitionError`) are diagnosable and
  non-mutating; duplicate online admission is an invariant violation with
  a fatal diagnostic; build failures are fatal boot-critical before any
  CPU starts. Failed is terminal; no recovery path exists at P3 by plan.
- **Security.** The registry is not guest-reachable. The eligibility gate
  is P3's authorization boundary for per-CPU resources: consumers must
  derive permission to use a CPU from the gate, not from their own state
  guesses. No board/platform names appear; the state machine is
  platform-independent by construction.
- **Observability.** Every accepted transition emits a CPU-attributed
  event (from/to/cause/caller role); refusals emit diagnostics; the
  SMP-ready dump renders the full authority surface. Event catalog
  registration belongs to [P3-W11](../p3-w11-smp-observability/README.md);
  W03 defines content, not transport.

## 3. Handoff checklist

Before handing W03 to a reviewer:

- the exact changed-file list, with crate/module placement traced to the
  approved workspace design;
- W03-DV01–DV06 evidence paths with run status, including explicit
  not-run entries (counts not reachable; parallel-contention stress
  boundary deferred to W12; matrix deferred to W13);
- confirmation that new `unsafe` is zero for this package (pure
  Rust + atomics) or fully inventoried per P0 governance if implementation
  found a justified exception;
- confirmation that no start-call mechanics, per-CPU storage, rendezvous
  logic, lock type, or vCPU concept was added; no direct state-word
  mutation API exists;
- consumer contract confirmations: W04 (gate at install time), W05
  (admit-online ownership point; snapshot for terminal checks), W07/W08
  (online set as targeting universe), W09 (state reads for attribution),
  W10–W13 (event stream, snapshot assertions, dump);
- the recorded follow-up note on the `Failed`-vs-`Excluded` vocabulary
  conflation (state-machine file §2), with its decision owner;
- any recorded blocker (sibling-contract coordination; P2 allocation
  gap) with its decision owner.
