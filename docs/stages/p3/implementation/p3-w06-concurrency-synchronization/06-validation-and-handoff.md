# P3-W06 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

## 1. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W06-DV01 | Normal mutual exclusion correct | L-1 hammer + ordering test (host) | N-thread acquire/mutate/release over shared counter; exact-final-value assertion; happens-before observed via payload | exact accounting at N ∈ {2,4,8,32}, K = 10_000; no lost update; no harness timeout | primitive correctness under the host memory model; not AArch64 SMP behavior |
| W06-DV02 | IRQ-sensitive flavor correct | L-3 irq-save tests (host, simulated intrinsics) | save/mask pairing, same-CPU re-entry blocked by masking, restore after release | pairing exact under contention; no re-entry; restore always executed | flavor semantics; not real DAIF behavior (P1 baseline owns the intrinsics' hardware contract) |
| W06-DV03 | Context rules hold at construction sites | MIS/CR conformance review | walk every lock construction site; check flavor choice, class tag, bounded-section rule | every site conforms; census recorded | rule adherence at review time; not future sites |
| W06-DV04 | Atomic policy consistent and applied | AP consistency review + AP-R checks | compare table vs delivered W02–W05 protocols; grep-level review for SeqCst/unjustified fences | per-protocol verdict recorded; no SeqCst; only W05's justified fence | policy consistency; not that future consumers comply (W10 audits) |
| W06-DV05 | Ladder covers required subsystems; no violation in declared census | L-4 model check + LOL review | host harness over declared construction sites; rank-ascension assertion | declared census acyclic and rule-conformant | ladder adequacy for named subsystems (allocator, registry, statistics, future consumers); not runtime deadlock-freedom of consumers |
| W06-DV06 | Misuse constraints are review-checkable | checklist walk (MIS-1..MIS-12) against the delivered code and designs | each MIS item mapped to at least one checkable artifact (code, design sentence, or review step) | all twelve items checkable; W10 can consume them | enforceability of the rules; not consumer compliance |
| W06-DV07 | Integration with consumers and handoff recorded | closure review | read the integration map against W07–W11 plans and this design's rule ids; handoff checklist below | every consumer obligation citable; no silent gaps | handoff readiness; not that consumers are done |

Task-book trace: P3-V06 passes when reviewed shared-state access follows
declared atomic/lock/IRQ rules and contention evidence shows no corruption
or unexplained deadlock within the declared limits. W06 delivers the rules,
the primitive-level evidence (DV01–DV03), and the enforcement surfaces;
consumer-level contention stress on QEMU is
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md) and the matrix is
[P3-W13](../p3-w13-qemu-smp-regression/README.md). Evidence states (passed /
failed / blocked / not run) are recorded with command, environment, and
timestamp in `../../verification/p3-w06-concurrency-synchronization-verification.md`;
implementation decisions go to
`../p3-w06-concurrency-synchronization-record.md`. Neither file is created
by this design.

## 2. Error, security, and observability model

- **Errors:** the primitives have no error returns; detected corruption of
  a lock word is a fatal-class invariant violation (P0-W14 classification)
  on the P1-W07 path with CPU attribution. Ladder violations and misuse
  findings are review/audit failures, not runtime-recoverable errors.
- **Security:** none of this surface is guest-reachable; there is no
  authorization decision in a lock. The security-relevant rule is CR-5
  (fatal path must not deadlock on a lock) because an unreportable fatal
  state is a diagnosability failure with safety consequences.
- **Observability:** lock instances expose only diagnostic-grade
  introspection (never synchronization inputs); contention *events* and
  counters are W11's catalog (Statistics class; AP-3 Relaxed counters);
  W06 provides the hooks' existence, not the event content. A held-lock
  census at boot is diagnostic output, citable by W10.

## 3. Handoff checklist

Before handing W06 to a reviewer, provide:

- the changed-file list and the `unsafe` inventory entries for the
  primitives;
- DV01–DV07 evidence paths with run status, including explicit not-run
  entries (QEMU SMP contention, real-DAIF behavior, consumer stress);
- the declared contention limits (04 §7) confirmed as recorded constants
  with rationale;
- the consistency verdict per delivered W02–W05 protocol (DV04);
- the declared ladder census rows for planned consumers (W08/W09/W11);
- open items for W07 (WFE/SEV bounds and idle-wait declaration), W08
  (Infrastructure-class lock, BW-4 reactive wait), W09 (CR-4/CR-5
  citation), W10 (MIS/LOL as audit criteria), W11 (Statistics class,
  AP-3 counters, contention hooks), W14/P4 (policy as the synchronization
  contract) — without resolving their contracts here;
- confirmation that no barrier, condvar, rwlock, blocking lock, global
  lock registry, or consumer data structure was added.
