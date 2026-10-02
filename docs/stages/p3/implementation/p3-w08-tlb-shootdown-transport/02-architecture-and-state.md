# P3-W08 Architecture and State

**Current admission (2026-10-02):** [Amendment 07](07-timeout-ownership-remediation.md) removes Pending supersession. The owner selected W08-SYNC option A on 2026-10-02: single-attempt protocol admission, TransportBusy on contention, no SpinLock across collection. W06/W08 contracts are reconciled below; final design admission, implementation and execution evidence remain pending.

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

## 1. Roles and transport shape

```text
INITIATOR (any online CPU, thread context)        TARGET (each mask member)
------------------------------------------        -------------------------
validate + reduce mask vs OnlineSet (W03)          idle / poll point
try admission once; busy => TransportBusy             |
preflight ALL slots; Pending => release admission + TargetsBusy
for each ready target:                               | wake (W07 kind 1)
  write descriptor (atomic relaxed store)                      v
  CAS exact Empty/Completed->Pending{seq} (release)       consume_request():
  notify(target, kind 1)                            acquire control; read desc
collect: bounded acquire-poll of all                execute bound operation
  target control words                               (P3: TransportNoop)
  until Completed or bound exhausted                 CAS Pending->Completed
latch result; release admission                             (release)
return Completed / TimedOut{unacked}
```

Exactly one initiation/collection call owns serialization at a time.
Older target operations may remain Pending after timeout; no selected Pending
slot is overwritten (README decision 5). Targets are stateless beyond their own
slot: they need no lock, no queue, and no knowledge of other targets.

## 2. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Target selection | `TargetMask`, validation, exclusion reporting, broadcast expansion | requested mask, W03 `OnlineSet`, W05 gate | effective target set + excluded set, or errors | none (derived) | lifecycle authority (W03) |
| B. Target consumption | slot state machine, descriptor read, bound-operation dispatch, completion | own slot, wake events | Completed transitions; consumption counts | own slot's state fields | what the bound operation does (P4) |
| C. Initiation and collection | single-attempt admission, publish, wake, bounded collect, timeout, results | requested mask + descriptor | transport result with per-target accounting | admission state; initiator-side counters | bound-operation content; Stage-2 semantics (P4) |
| D. Boundary and evidence | opaque-descriptor rule, no-op placeholder, scenario definitions | this design | citable contract statements; W12/W13 inputs | none | catalog (W11) |

## 3. Objects and ownership

| Object | Count | Owner | Writers | Readers |
|---|---|---|---|---|
| `TlbReceptionSlot` (control + descriptor) | one per CPU (in `PerCpuArea`) | W08 (contents); W04 (placement) | control: initiator CAS then target CAS; descriptor: initiator (stable while Pending) | the owning target (acquire); initiator (acquire poll) |
| Admission state: Idle / Active(cpu) | one per boot | W08 | strong CAS by an online CPU; checked release by token owner only | contender (one attempt), diagnostics |
| Transport counters (initiated/completed/timed-out per CPU) | per CPU | W08 (W11 owns catalog) | owning CPU (its own initiations); per-target completion count target-private | diagnostics read-only |
| Bound-operation binding | one per stage | W08 at P3 (`TransportNoop`); replaced by P4's design via W14 | design change only | targets (dispatch) |

Sequence tags pair publication and completion; they do not cancel receivers.
Descriptors stay stable throughout Pending, even after timeout. No receiver
access to old descriptor/resources follows Completed publication.
See [amendment 07](07-timeout-ownership-remediation.md) for wrap/lifetime reasoning.

## 4. Slot state machine (per target)

```text
Empty --initiator CAS--> Pending{seq, descriptor stable}
Pending --target CAS (after bound op)--> Completed{seq}
Completed --next request CAS--> Pending{seq+1}      (reuse after completion)
```

- Transitions are single CAS operations; no other state exists.
- The initiator never writes a target's control word except the
  Empty→Pending and Completed→Pending edges; the target writes only
  Pending→Completed. Each edge has exactly one writer role — the
  one-owner-per-transition guardrail holds at slot granularity.
- Reserved/illegal encodings in the control word are corruption — fatal
  diagnostic path (P0-W14 class), never "fixed up".

## 5. Single-flight admission and progress limits

Owner-selected A uses one strong atomic compare-exchange from Idle to Active(cpu).
Failure returns TransportBusy, with no publication or sequence change. A private
non-Copy token permits only that call to release admission; targets never acquire
it. Successful Acquire admission and Release retirement serialize initiators.
Slot release/acquire edges separately order descriptors and target completion.

The admitted call preflights all targets, publishes, then collects with no
SpinLock or data-lock guard held. It services its own reception under BW-4.
TargetsBusy releases admission before return; Completed/TimedOut copy their
result before release and perform no subsequent slot reads. Fatal partial
publication retains admission, slots and resources; neither Drop nor a retry
may reset them. These are the W06 BW-6 protocol constraints; BW-2 is unchanged.

There is no admission wait or fairness guarantee. Explicit caller retries need
bounds and reception service. A halted initiator may strand admission; a halted
target may remain Pending after timeout. Collection bounds poll count, not
scheduling or wall-clock latency. P3 TransportNoop is non-reentrant and never
waits; future P4 operations must establish their own bounded execution context.
See [amendment 07](07-timeout-ownership-remediation.md) for decision and scope.

## 6. Failure model

- **Refused initiation** (`NotReady`, `UnknownTarget`, `TargetNotOnline`,
  `EmptyMask`, `TransportBusy { excluded }`, `TargetsBusy { busy, excluded }`) makes no new-request publication/sequence writes; refusal diagnostics and
  independent old-reception progress remain possible.
- **Excluded CPUs** (requested but not online) are reported in the
  result; the request proceeds for the effective set.
- **Timeout** returns collected acked/unacked masks. Pending may mean not
  started or executing; it remains immutable to initiators. A late target
  completes normally. Only acquire-observed Completed permits reuse.
  A retry selecting any Pending target returns TargetsBusy before all writes.
  A stalled CPU stays occupied; timeout neither changes lifecycle nor releases
  descriptor-referenced resources.
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
