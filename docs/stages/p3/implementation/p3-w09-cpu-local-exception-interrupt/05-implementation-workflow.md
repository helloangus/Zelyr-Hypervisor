# P3-W09 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tracked
tree. Prerequisite checks: the P1-W05/P1-W07 contracts are available as
reviewed deliverables; the W04 slot reservation and `current()` contract,
the W02 entry attribution, the W03 state read, and the W06 CR rules are
available as agreed designs; the host-side test entry (P0-W08 baseline)
is available.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the delivered P1 contract freezes a global save/scratch region or a
  global context buffer — Architecture Change Request with the P1 owner
  (README decision 3); do not fork the entry path or silently add a
  per-CPU copy alongside a shared one;
- the W04 `ExceptionLocalSlot` is absent/undersized, or `current()` is
  not valid from the first post-install instruction — cross-design
  conflict with the W04 owner;
- making exceptional paths correct appears to require enabling an
  interrupt source, designing IRQ dispatch, or touching the GIC — out of
  scope; raise it (README decision 5);
- the fatal path appears to need a lock other than Diagnostics or an
  unbounded wait — contract violation (W06 CR-5); raise, do not bend.

## 2. Ordered implementation steps

### Step 1 — exception-local slot contents and init

Target: the module the approved build design assigns.

Work: implement the `ExceptionLocalSlot` contents and
`init_exception_slots` per
[03-code-contracts-exception-local-state.md](03-code-contracts-exception-local-state.md)
§1–§3. Wire the init call into the boot sequence's global-init region
alongside the other slot inits.

Acceptance: init validates every candidate slot; the epoch mechanism
detects a stale/zeroed slot in a test.

Failure/blocker: a slot failing validation is fatal boot-critical —
diagnose the W04 integration.

Evidence: implementation record.

### Step 2 — attribution resolution

Target: same module.

Work: implement `resolve_attribution` per
[04-code-contracts-attribution-and-logging.md](04-code-contracts-attribution-and-logging.md)
§2 with the audited register-read boundaries (SAFETY comments per P0-W10
governance). The no-write-before-rank rule is enforced by construction
(function reads only).

Acceptance: rank table tests (header-valid, header-invalid + MPIDR
maps, MPIDR unusable) produce the exact rank and fields; no write occurs
on any rank before resolution completes.

Failure/blocker: a rank-1 misfire on an uninstalled CPU is the
highest-class bug here — the injected-state tests must prove the rank
boundary.

Evidence: verification record (W09-DV03 partial).

### Step 3 — begin/end bookkeeping and nesting bound

Target: same module; integration point per the P1 entry path.

Work: implement `exception_begin`/`exception_end` and the
`EXCEPTION_NESTING_MAX` terminal behavior per
[03](03-code-contracts-exception-local-state.md) §4–§5, integrated at
the boundaries the P1-W05 contract defines (mechanics unchanged).

Acceptance: begin/end pairing holds under intentional synchronous
faults; a fault-while-handling nests exactly one level; a third level
lands terminal with the snapshot's attribution; end-without-begin is
fatal.

Failure/blocker: any need to change vector mechanics to place the hooks
is a P1-contract question — stop and resolve; do not restructure the
entry path.

Evidence: verification record (W09-DV01, DV02 partial).

### Step 4 — fatal integration

Target: same module; P1-W07 integration point.

Work: implement `fatal_with_attribution` per
[04](04-code-contracts-attribution-and-logging.md) §3: write-once
per-CPU record first, then the CR-5 console strategy, then terminal.

Acceptance: every captured report carries attribution; the record is
write-once (second attempt is a detectable violation); under a
simulated poisoned console lock the fallback emits marked, attributed
lines and terminates — no hang.

Failure/blocker: a fatal path that waits unboundedly or takes any
non-Diagnostics lock is a contract violation — stop.

Evidence: verification record (W09-DV05).

### Step 5 — concurrent logging behavior

Target: same module + tests.

Work: implement the CL-1..CL-5 rules
([04 §4](04-code-contracts-attribution-and-logging.md)) on the
diagnostic emission paths; build the simultaneous-emission test
(N threads-as-CPU emitting under the Diagnostics lock; fatal-path
contention case with the bounded try-lock).

Acceptance: no interleaved partial lines in the normal case; the
contended fatal case produces marked, attributable, bounded output;
every exceptional-path line passes the CL-3 attribution check.

Failure/blocker: an unattributed line is a finding — fix the emitter,
not the check.

Evidence: verification record (W09-DV06).

### Step 6 — non-overlap review and closure

Work: perform the DV04 review (list every writable location reachable
from vector entry; each must be own-area or console) and hand the result
to the W10 audit criteria. Run the full matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md); record
implementation decisions in
`../p3-w09-cpu-local-exception-interrupt-record.md` and evidence in
`../../verification/p3-w09-cpu-local-exception-interrupt-verification.md`
only for what was actually performed; completion is claimed only in the
verification record, only for what was run.

## 3. Deferred-to-consumer obligations (recorded, not performed here)

- W10 consumes NO-1..NO-3 and the rank rules as audit criteria for
  P0–P2 diagnostic/console state.
- W11 owns the catalog/formats for emission events; W09 guarantees the
  attribution fields and the emission points only.
- W12/W13 exercise concurrent faults, simultaneous logging, and both
  CPU roles at SMP scale and over repeated boots.
- The future host-IRQ design owner consumes the declared posture, the
  Reserved triggers, and the W06 CR bindings; W09 enables nothing.
