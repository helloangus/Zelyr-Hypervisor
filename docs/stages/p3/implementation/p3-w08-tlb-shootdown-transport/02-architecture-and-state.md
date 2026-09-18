# P3-W08 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

## 1. Roles and transport shape

```text
INITIATOR (any online CPU, thread context)        TARGET (each mask member)
------------------------------------------        -------------------------
acquire initiation SpinLock (single-flight)       idle / poll point
validate + reduce mask vs OnlineSet (W03)             |
for each target:                                      | wake (W07 kind 1)
  write descriptor (plain store)                      v
  CAS control Empty->Pending{seq} (release)       consume_request():
  notify(target, kind 1)                            acquire control; read desc
collect: bounded acquire-poll of all                execute bound operation
  target control words                               (P3: TransportNoop)
  until Completed or bound exhausted                 CAS Pending->Completed
release initiation lock                             (release)
return Completed / TimedOut{unacked}
```

Exactly one transport operation is in flight system-wide at any time
(single-flight, README decision 2). Targets are stateless beyond their own
slot: they need no lock, no queue, and no knowledge of other targets.

## 2. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Target selection | `TargetMask`, validation, exclusion reporting, broadcast expansion | requested mask, W03 `OnlineSet`, W05 gate | effective target set + excluded set, or errors | none (derived) | lifecycle authority (W03) |
| B. Target consumption | slot state machine, descriptor read, bound-operation dispatch, completion | own slot, wake events | Completed transitions; consumption counts | own slot's state fields | what the bound operation does (P4) |
| C. Initiation and collection | single-flight lock, publish, wake, bounded collect, timeout, results | requested mask + descriptor | transport result with per-target accounting | initiation lock; initiator-side counters | bound-operation content; Stage-2 semantics (P4) |
| D. Boundary and evidence | opaque-descriptor rule, no-op placeholder, scenario definitions | this design | citable contract statements; W12/W13 inputs | none | catalog (W11) |

## 3. Objects and ownership

| Object | Count | Owner | Writers | Readers |
|---|---|---|---|---|
| `TlbReceptionSlot` (control + descriptor) | one per CPU (in `PerCpuArea`) | W08 (contents); W04 (placement) | control: initiator CAS then target CAS; descriptor: initiator (stable while Pending) | the owning target (acquire); initiator (acquire poll) |
| Initiation lock | one per boot | W08 | any online CPU (short critical sections) | nobody (lock, not data) |
| Transport counters (initiated/completed/timed-out per CPU) | per CPU | W08 (W11 owns catalog) | owning CPU (its own initiations); per-target completion count target-private | diagnostics read-only |
| Bound-operation binding | one per stage | W08 at P3 (`TransportNoop`); replaced by P4's design via W14 | design change only | targets (dispatch) |

Sequence tags make stale state detectable: every Pending carries the
request's global sequence; a target completing a superseded request is
impossible under single-flight, and the tag still guards the
post-timeout supersession path (README decision 5).

## 4. Slot state machine (per target)

```text
Empty --initiator CAS--> Pending{seq, descriptor stable}
Pending --target CAS (after bound op)--> Completed{seq}
Completed --next request CAS--> Pending{seq+1}      (supersession)
```

- Transitions are single CAS operations; no other state exists.
- The initiator never writes a target's control word except the
  Empty→Pending and Completed→Pending edges; the target writes only
  Pending→Completed. Each edge has exactly one writer role — the
  one-owner-per-transition guardrail holds at slot granularity.
- Reserved/illegal encodings in the control word are corruption — fatal
  diagnostic path (P0-W14 class), never "fixed up".

## 5. The single-flight argument (why the only wait cannot deadlock)

The protocol's only blocking wait is an initiator's completion collection
(bounded) and the initiation lock (unbounded in principle). The hazard:
CPU A holds the initiation lock and waits for CPU B's completion, while B
spins on the initiation lock and never consumes its own request.

The hazard is removed by W06's BW-4 reactive-wait rule, made binding for
W08: **a CPU waiting for the initiation lock must interleave
`consume_pending_requests()` (its own target duty) into its wait loop.**
Then B remains a functioning target while waiting, A's collection
completes, the lock is released, and B proceeds. Formally: the wait graph
has no cycle because every waiter either (a) holds the lock and waits on
targets that never hold it, or (b) holds nothing and remains
target-responsive. Targets never wait on initiators. This argument is the
reason the reactive-wait rule is a Required contract item, not advice.

Bounds: collection is bounded (BW-1 constant, recorded); the initiation
lock's hold time is bounded by the collection bound; therefore lock
waiters are also bounded in practice, though the bound is the collection
bound, not a lock-specific constant (recorded).

## 6. Failure model

- **Refused initiation** (`NotReady`, `UnknownTarget`, `TargetNotOnline`,
  `EmptyMask`) is side-effect-free and diagnosable; no slot is touched.
- **Excluded CPUs** (requested but not online) are reported in the
  result; the request proceeds for the effective set.
- **Timeout**: `TimedOut { unacked: TargetMask }` after the bound; slot
  states stay readable for CPU-by-CPU diagnosis (Pending = no progress,
  Completed = accounting lag). The transport remains usable: a later
  request supersedes by sequence. A target that repeatedly never
  completes is a suspect-CPU diagnostic for W11/W12 surfaces — at P3 it
  is not a lifecycle event (W03 has no post-admission failure edge).
- **Slot/control corruption** (illegal encodings, sequence regressions
  under single-flight): fatal invariant path with CPU attribution.
- **Bound-operation misbehavior** (P4's future content) is outside this
  contract: P3's `TransportNoop` cannot fail; the contract *requires*
  P4's binding to define its own failure semantics.

## 7. Consumer integration map

| Consumer | Surface consumed | Contract point |
|---|---|---|
| W11 (observability) | per-target states, initiation/completion/timeout outcomes, counters | [03](03-code-contracts-transport-request.md) §3; [04](04-code-contracts-transport-initiator.md) §3–§4 |
| W12 (stress) | concurrent initiations, invalid/offline masks, induced non-consuming target, accounting invariants | [04](04-code-contracts-transport-initiator.md) §2–§5 |
| W13 (regression) | scenario definitions as matrix inputs | [06-validation-and-handoff.md](06-validation-and-handoff.md) §3 |
| W14/P4 (handoff) | the full transport contract + the two P4 obligations (descriptor meaning; bound operation with barriers) | [06-validation-and-handoff.md](06-validation-and-handoff.md) §3 |
