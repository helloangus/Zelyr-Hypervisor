# P7-W11 Invariant Checks and Failure Signals

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W11 detailed design](README.md).

## 1. Invariant catalog

Each invariant is a checkable property stated over the P7-W09 observability
contract ([P7-W09 design](../p7-w09-accounting-diagnostics/README.md)) and the
P7-W02–W08 behavior contracts. The catalog fixes **what must be observable and
judged**; the mechanism that produces the underlying trace events, counters,
and diagnostics is owned by those contracts and is never redefined here. The
exact trace-field and counter names are bound at implementation time from the
W09 contract; if a required observable does not exist in the evidenced W09
contract, the affected invariant is a blocked prerequisite for every scenario
that lists it.

| ID | Invariant statement | Observable derivation | Check point | Failure signal |
|---|---|---|---|---|
| INV-1 | A vCPU never executes on two pCPUs at once (no double-run) | Running-state transitions paired per vCPU across the per-pCPU execution traces of W09 | Continuous during run | F2 (checker), F1 if it escapes as a hypervisor fatal |
| INV-2 | A vCPU in Running state executes on exactly one pCPU, and that pCPU records it as current | vCPU state field vs per-pCPU current-vCPU attribution in W09 counters | Continuous | F2 |
| INV-3 | vCPUs in Offline, Blocked, Paused, Stopped, or Faulted state never enter the Guest | Guest-entry events correlated with the vCPU state recorded by W02/W09 semantics | Continuous | F2 |
| INV-4 | Accounting is coherent: per-vCPU/pCPU/VM counters never regress and do not double-count across switches, preemptions, block/wakeup, and pause/resume | End-of-run and per-window audit of W09 counters against the emitted event stream | Window boundaries and end of run | F4 |
| INV-5 | Execution occurs only on pCPUs eligible under the declared pin/affinity/dedicated/shared configuration (W03) | Placement attribution of every execution interval vs the declared scenario configuration | Continuous | F2 |
| INV-6 | Every continuously Runnable, equal-class vCPU accumulates nonzero execution within every declared fairness window | Per-window execution-time aggregation from W09 accounting | Window boundaries | F3 (starvation finding) |
| INV-7 | No lost wakeup: every wakeup-capable event (timer, vIRQ/SGI, Notification, internal event) either makes its target runnable or is recorded as delivered-to-ineligible with a reason consistent with the W06/W07 contracts | Injected/observed event reconciliation: event count vs wakeup outcomes per vCPU | End of run and per-event during S2 | F2 (unexplained loss), F5 (workload-visible loss) |
| INV-8 | Containment: a Guest fault, stop, or pause affects only its own vCPU/VM scheduling context; other VMs continue | Cross-VM progress markers and scheduler state remain consistent after the contained event | Event-locked windows around injected faults | F2; escape into global failure → F1 |

Check-point semantics:

- **Continuous** checks are evaluated as the evidence stream is produced, so
  that a violation is reported with the event context in which it occurred.
- **Window/end-of-run** checks are evaluated over recorded evidence after or
  during the run at declared boundaries; they require the evidence to be
  complete (a truncated run closes as F6, never as a pass).

What the catalog does not claim: the checks observe only the W09 surface. A
violation invisible on that surface is not detected by W11; each verification
record states this limit explicitly.

## 2. Integration contract with trace and diagnostics

For every scenario, the evidence stream must contain, at minimum, the W09
contract's per-event fields (vCPU, pCPU, VM, state/reason, ordering/timestamp
source) and the diagnostic payload that P7-V21 requires (current pCPU, VM/vCPU,
state, affinity, stop reason, recent and pending events). On any F1–F5 signal,
the following must be captured together as one failure artifact set:

1. the diagnostic report as emitted by the W09/P1 failure boundary;
2. the serial capture of the run up to the signal;
3. the scenario parameter and seed record;
4. the checker evaluation (which INV entry, which evidence items).

If the W09 diagnostic contract cannot supply an item, the affected signal is
classified with the missing item named; the run is still failed, and the gap
is recorded as a finding against the observability contract. W11 never
suppresses, filters, or reinterprets diagnostics to make a run classifiable.

## 3. Failure-signal taxonomy

Objective classification is the core of P7-V24–V27: "required stress matrices
objectively reveal corruption, starvation, lost wakeup, livelock, and
placement violations when present." The taxonomy:

| Signal | Meaning | Detection source |
|---|---|---|
| F1 | Hypervisor fatal: the P1 fatal-diagnostics boundary fired (invariant panic, unhandled exception) | Fatal diagnostic output; run aborts |
| F2 | Invariant-check violation: a checker evaluation of INV-1–INV-8 failed | Checker report bound to evidence items |
| F3 | Progress failure: declared completion markers absent by the watchdog deadline, or a starvation window (INV-6) closed with zero execution for a continuously runnable vCPU | Watchdog timer over serial/evidence stream; window audit |
| F4 | Accounting incoherence: INV-4 audit found regression or double-count | Counter audit |
| F5 | Workload-visible corruption: a W10 workload's declared deterministic expectation failed (marker mismatch, shared-memory checksum error, event-count mismatch) | Workload output vs declared expectation |
| F6 | Indeterminate: the run ended without a classifiable outcome (timeout without diagnosis, truncated evidence, harness fault) | Absence of a classifiable signal at run end |

Classification rules:

- A run **passes** only when it reaches its declared completion with all
  listed invariants holding and all completion markers present.
- F6 is a **failed run with artifacts**, never a pass and never silent: the
  verification record must name why the outcome was indeterminate. Repeated
  F6 on a scenario blocks that scenario (blocked status) until the cause is
  understood.
- A scenario row that detects F1–F5 is a **failed scenario**, recorded as a
  finding; the stage-level meaning (gate failure vs informational finding) is
  per the class column of the matrices.

## 4. Determinism and reproduction rules

- Stress runs are **not required to be bit-deterministic**; they are required
  to be **determinately classifiable** (pass or a specific F-signal) under the
  recorded parameters, seeds, and environment.
- Every seeded input (event offsets, payload parameters, scheduling jitter
  sources) records its seed or parameter value. Unrecorded nondeterminism in
  the harness is a defect; the run is F6.
- A detected F1–F5 failure is retried up to two times **for classification
  and diagnosis only**. Reproduced failures are recorded as reproduced;
  non-reproduced failures remain findings with the original artifact set
  attached. Reproduction attempts never overwrite the original verdict.
- A passing run is repeated per the floors in
  [the scenario matrices](01-stress-scenario-matrices.md) §6; a single pass
  never satisfies a scenario row by itself.

What repetition proves and does not prove: repeated passes increase coverage
of the swept windows; they do not bound the probability of unexercised races
and must never be summarized as a statistical guarantee.
