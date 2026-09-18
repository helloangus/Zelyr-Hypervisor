# P1-W04 Read-back and Declaration Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W04 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; all
fixed-size data.

## 1. Read-back verification

```text
Name and stability: the read-back step of the specification engine
  ([02-code-contracts-control-writes.md](02-code-contracts-control-writes.md)
  §1); internal; stable within P1.
Purpose and caller: make "explicitly established" mechanically true at
  every boot (parent README decision 6). Caller: the establishment body,
  immediately after each write.
Inputs / outputs: the written control, mask, and intended value; returns
  ok or the comparison result.
Preconditions / postconditions: executes immediately after the write; pure
  observation; no state change.
State and ownership change: none.
Concurrency/allocation context: boot context; no allocation; no barrier
  needed for these context-synchronizing accesses (recorded implementation
  note).
Errors and failure guarantee: a mismatch produces the BaselineError of §2 —
  the check itself cannot fail otherwise; it never retries or rewrites.
Security/authorization checks: none.
Logic:
  observed = read_sysreg(control) & mask
  ok iff observed == intended & mask
Validation: W04-DV03; W04-DV05 (order/coverage — every write has exactly
  one read-back).
```

## 2. `BaselineError` and the fatal route

```text
Name and stability: BaselineError { control: ControlId, expected: u64,
  observed: u64 } with fn fail_baseline(e: BaselineError) -> !; internal;
  stable within P1.
Purpose and caller: the only failure class of the baseline. Callers: the
  read-back step, the precondition assertion, the guard-consistency check.
Inputs / outputs: control identity and the masked expected/observed pair;
  output is the panic-route report carrying the control's static name and
  the comparison values (through W02's bounded formatter).
Preconditions / postconditions: called only inside the establishment body;
  never returns (panic route; phase-attributed `el2-baseline` by the W09
  tracker position — W09 matrix row `el2-baseline`).
State and ownership change: none of its own; the declaration static records
  the failing point per §4's status rule.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: terminal by construction; no retry, no
  fallback value, no partial continuation (W09 T3; H2).
Security/authorization checks: none; the route reflects machine facts
  (masked register values), not secrets.
Logic: render via the bounded formatter; invoke the panic route.
Validation: W04-DV03; W11 consumes the class vocabulary.
```

## 3. Guard consistency

```text
Name and stability: the guard check inside the specification engine;
  internal; stable within P1.
Purpose and caller: enforce parent README decision 7 — optional elements
  are established only when their W03 fact is Present, and skipped with a
  recorded skip otherwise. Caller: the establishment body per guarded spec.
Inputs / outputs: the spec's guard fact; returns Present/SkippedAbsent.
Preconditions / postconditions: CAPABILITIES published (phase
  prerequisite); the queried fact is read through the W03 query API, never
  re-derived. A guard whose fact is Present but whose subsequent write
  faults is an ordinary BaselineError (§2) — the guard decides existence,
  not success.
State and ownership change: records the skip in the declaration static.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: querying an unpublished report is an
  invariant violation routed via §2 (cannot occur in a legal lifecycle).
Security/authorization checks: none.
Logic: if fact observation is Present -> proceed; else record skip; never
  guess.
Validation: W04-DV03/DV04 (ADR-044 conduct: capability-driven, no platform
  names).
```

## 4. Declaration API

```text
Name and stability: BaselineStatus (Established | SkippedAbsent |
  NotEstablished); BaselineCategory (C1..C8 per
  [01-architecture-and-state.md](01-architecture-and-state.md) §2, with
  const ALL and fn label()); static EL2_BASELINE in a once-write cell;
  fn baseline_status(cat: BaselineCategory) -> BaselineStatus; all
  internal; stable within P1.
Purpose and caller: the known-state declaration W05/W08/W09 consume
  (parent README decision 8: recorded status, not live re-reads). Callers:
  the establishment body (writer); W05, W08, W09, W11 (readers).
Inputs / outputs: category; status. Also recorded: the per-category write
  summary (control, masked value) retrievable by `baseline_value
  (ControlId) -> Option<u64>` for W08's premise checks.
Preconditions / postconditions: status reads are legal from the
  `el2-baseline` completion onward; a read before establishment returns
  NotEstablished (honest default, never a fabricated Established).
  Establish/Skip transitions happen exactly once each, in C1..C8 order.
State and ownership change: the declaration static is the only writer-
  visible state; monotone within a boot (Established/SkippedAbsent never
  revert).
Concurrency/allocation context: the audited once-write cell (same pattern
  family as W02's BootContext and W03's report; SAFETY: single boot CPU,
  DAIF masked, one boot path); readers after establishment; no allocation.
Errors and failure guarantee: cannot fail; misuse (post-failure reads)
  observes the honest NotEstablished/SkippedAbsent state, never a
  fabricated success.
Security/authorization checks: none; declaration is knowledge, not
  authority.
Logic:
  establish(cat): set status inside the audited cell
  baseline_status(cat): read from the cell
Validation: W04-DV06; consumer reviews (W04-DV07) read the API, not the
  registers.
```

## 5. Recorded-value boundary (what the declaration does not promise)

The declaration carries exactly the §2 values and no semantics beyond them:
it is not a guest-virtualization policy, not an MMU promise, not an
identity-map commitment, and not a warranty that later stages may not
supersede a value through their own designs. Consumers asserting premises
(e.g. W08 asserting `SCTLR_EL2.M=0`) use `baseline_value`; consumers
asserting readiness (e.g. W05) use `baseline_status`. The limitation set —
vectors unowned until W05, values minimal-not-final, SCR_EL3 outside P1
ownership — is part of the declaration's documentation and travels to W12
verbatim.
