# P3-W07 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tracked tree.
Prerequisite checks: the W04 area/slot contract and the W03 registry/eligibility
and W05 phase-gate contracts are available as agreed designs; the W06 policy
ids referenced here exist; the host-side test entry (P0-W08 baseline) is
available.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W04 slot reservation is absent, undersized, or differently aligned —
  cross-design conflict with the W04 owner; do not invent a second storage
  location;
- W03's online set or W05's phase gate are unavailable or shaped
  differently than assumed ([01 §1.2](01-scope-and-foundations.md)) —
  resolve with their owners; do not build a W07-private online-set copy or
  a second phase word;
- implementing the primitive appears to need allocation, a lock, a queue,
  or a timer — design error; raise it (README decisions 2, 8);
- a consumer claims a reserved kind without a design — stop at review;
  kind claims are design-level.

## 2. Ordered implementation steps

### Step 1 — slot contents and init

Target: the module the approved build design assigns; the W04 slot.

Work: implement the `NotificationSlot` contents, `NotificationKind`
encoding, and `init_slots` per
[03-code-contracts-notification-slot.md](03-code-contracts-notification-slot.md)
§1–§3. Wire the init call into the boot sequence's global-init region
(single-threaded, pre-release) at the point after W04's `allocate_all`.

Acceptance: init validates every candidate slot; the initial postconditions
hold; the W04→W07 ownership handoff is recorded in the implementation
record.

Failure/blocker: a slot that fails validation is fatal boot-critical per
the contract — diagnose the W04 integration; do not skip the CPU silently.

Evidence: implementation record.

### Step 2 — send and observe primitives

Target: same module.

Work: implement `send_event` and `observe` per
[03](03-code-contracts-notification-slot.md) §4, §6 with the W06 AP-1/AP-2
orderings and the `sev`/`wfe` audited boundaries (SAFETY comments per
P0-W10 governance).

Acceptance: unit tests show: single send → single delivery; ordering
(receiver never observes payload before sequence); coalescing gaps counted
exactly.

Failure/blocker: an ordering failure is a design bug — fix the protocol,
not the test's tolerance.

Evidence: verification record (W07-DV02 partial).

### Step 3 — gates and `notify`

Target: same module.

Work: implement `notify` per
[04-code-contracts-notification-api.md](04-code-contracts-notification-api.md)
§2 with the W05 phase gate and W03 online-set targeting; the error rows
are exhaustive and side-effect-free.

Acceptance: every error row demonstrated by a test with the corresponding
gate state; no slot write occurs on any refusal (asserted via a
slot-fake).

Failure/blocker: a missing gate (e.g., sending before `SmpReady` in a
test) is a contract bug — the gates are the security boundary.

Evidence: verification record (W07-DV01, DV05).

### Step 4 — poll/wait and self-notification

Target: same module.

Work: implement `poll`, `wait`, and confirm self-notification behavior
per [04](04-code-contracts-notification-api.md) §3, §6. `wait` uses the
audited `wfe` boundary; spurious wakeups re-checked.

Acceptance: self-notification appears in own accounting; wait returns
exactly one event per delivery under mixed senders; spurious wakes do not
fabricate events.

Failure/blocker: an event fabricated from a spurious wake is a protocol
bug (observe's sequence check must prevent it).

Evidence: verification record (W07-DV03, DV04).

### Step 5 — accounting invariants

Target: tests.

Work: implement the storm and mixed-traffic tests asserting the
[04 §4](04-code-contracts-notification-api.md) invariants
(arrivals + coalesced = sequences closed; per-kind sums; no silent loss)
within the declared limits (threads-as-CPU host harness per P0-W08).

Acceptance: invariants hold exactly at the declared limits; the
coalescing limitation is exercised (gap counting proven).

Failure/blocker: a counter drift is a finding — record failed, diagnose,
fix protocol, re-run.

Evidence: verification record (W07-DV06).

### Step 6 — closure review

Work: run the matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md); confirm the
handoff checklist; verify the non-RPC limit statement and the kind-claim
table are recorded. Record implementation decisions in
`../p3-w07-cross-cpu-notification-record.md` and evidence in
`../../verification/p3-w07-cross-cpu-notification-verification.md` only
for what was actually performed; completion is claimed only in the
verification record, only for what was run.

## 3. Deferred-to-consumer obligations (recorded, not performed here)

- W08 claims kind 1, defines its payload meaning, and builds its
  request/acknowledgement layer on notify/poll — W07 carries events only.
- W11 owns the catalog: event ids for send/receive/coalesce activity and
  any aggregation of the slot counters.
- W12/W13 exercise the primitive at SMP scale; W07's host-side evidence is
  the base layer, not a substitute.
- W02/W05 may integrate `wait` into their idle/parking paths through
  their own recorded extension points; W07 performs no edit to their
  designs.
