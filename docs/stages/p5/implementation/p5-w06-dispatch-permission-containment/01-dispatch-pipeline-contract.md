# P5-W06 Dispatch Pipeline Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W06 detailed design](README.md).

## 1. Logical modules and ownership

W06 adds one logical module — the **hypercall dispatch integration** — and no
new persistent subsystem. Physical placement (crate/module path) follows the
crate and module contracts established by the P0–P4 designs; this design fixes
only logical boundaries and must not invent a file tree.

| Module | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Hypercall dispatch integration | W06 (this design) | classified trap/exit with raw HVC request; calling vCPU and its security context | one structured outcome per call plus its result category; it does not own object state, capability state, Guest-memory mappings, or ABI encoding |
| Request decode and outcome vocabulary | W02 design (assumed contract) | raw register words | typed request or structural rejection class; W06 does not define call numbers, encodings, or error values |
| Guest-data boundary | W03 design (assumed contract) | Guest IPA + length + direction | validated/copyable access or a controlled failure; W06 does not re-implement range or mapping checks |
| Object-reference boundary | W04 design (assumed contract) | opaque handle value | live object reference of a stated class or a controlled invalid/stale/wrong-type outcome; W06 does not touch object-table state directly |
| Authority boundary | W05 design (assumed contract) | caller context + object reference + required right class | grant or a controlled denial; W06 does not read or mutate capability state directly |
| Telemetry integration | W09 design (future consumer) | result category per call | counters and safe logs; W06 guarantees only that a category exists |

## 2. Assumed prerequisite contracts and failure boundaries

The W02–W05 detailed designs are parallel work on this branch
(`../p5-w02-hypercall-abi-error-boundary/README.md`,
`../p5-w03-guest-data-safety/README.md`,
`../p5-w04-handle-lifecycle-type-safety/README.md`,
`../p5-w05-capability-rights-bootstrap-revocation/README.md`). Their exact
Rust names are theirs; W06 references them by the logical operations below
and adapts to the delivered names at implementation. Each row is an
assumption with a stated boundary if it delivers differently.

| Assumed contract | Source | What W06 assumes | Failure boundary if delivered differently |
|---|---|---|---|
| Request decode → typed request or structural rejection class | W02 | every call decodes to either a typed request or exactly one structural class (unknown call, unsupported feature, version mismatch, malformed) | a second structural vocabulary would fork the ABI — stop, record `Architecture Change Request`; do not map locally |
| Structured result encoding back to Guest registers | W02 | a single encoding path from semantic outcome to Guest-visible result | if absent, W06 cannot return results — blocked prerequisite, record it |
| `validate Guest range / copy Guest buffer` | W03 | checked, overflow-safe access with controlled failure per case (unmapped, partial, read-only write, bad type); a Guest-data fault is a contained outcome | W06 must not add its own range checks; if the boundary lacks a required case, the gap belongs to W03 — record, do not patch here |
| `resolve reference (handle) → live reference or controlled outcome` | W04 | invalid, stale-after-reuse, destroyed, repeated-destroy, and wrong-type values are distinguishable controlled outcomes; a handle grants no authority | if validity and authority are merged, ADR-013's model is violated — `Architecture Change Request` |
| `check authority (caller, reference, right class) → grant or denial` | W05 | caller-associated capability lookup with operation-level right classes; no VM-ID/first-VM/role shortcut; revocation observable | any identity shortcut is a security-model conflict — stop, `ADR Required` |
| Caller context (vCPU → owning security context) | P4 foundation + W05 bootstrap | the dispatch entry can name the calling security context without Guest-provided identity | without caller association the closed loop cannot close — blocked prerequisite, record |
| HVC/exception classification to a standard exit reason | P1/P4 | the Arch layer delivers the trap with saved request registers | dispatch must not parse exception state itself; gap belongs to P1/P4 — record |

In every boundary case the action is: stop the affected step, record the
conflict or block in the implementation record and the
`Architecture Change Request` / decision list, and continue only with
governing resolution. Silently adapting W06 to a deviating contract is
prohibited.

## 3. The ordered validation pipeline

Every dispatched call passes the stages in order. A failure at any stage
short-circuits to the containment mapping of
[02 §2](02-containment-and-error-classification.md). No stage before EXECUTE
mutates persistent state; stage order is therefore also the side-effect
boundary.

```text
S0  CONTEXT      bind calling vCPU -> security context (Hypervisor-side,
                 never Guest-supplied); snapshot raw request registers
S1  DECODE       W02 decode: typed request or structural class
                 (unknown | unsupported | version-mismatch | malformed)
S2  REFERENCE    W04 resolve of the request's object reference
                 (invalid | stale | destroyed | wrong-type are controlled)
S3  TYPE         request's operation applies to the resolved object class
S4  AUTHORITY    W05 check: caller-associated capability covering the
                 required right class for this operation (no-right |
                 revoked are controlled denials)
S5  ARGUMENTS    W03 validation of Guest-supplied buffers/ranges;
                 operation-specific argument legality
S6  STATE        operation's required object/lifecycle state holds
                 (bad-state is a controlled denial)
S7  EXECUTE      perform the operation (read-only for the P5 minimal call)
S8  RESULT       encode structured result to Guest registers (W02 path);
                 assign the call its result category (02 §4)
```

Rules:

- **Authorize before touch.** No stage reads or writes object, capability, or
  Guest-buffer state before S4 grants authority, except the reference
  resolution itself, which W04 defines as non-mutating lookup. A capability
  check must never degrade into a role, VM-ID, or first-VM test.
- **Validate before use.** Guest-controlled values (call number, version,
  flags, lengths, addresses, references) are consumed only after their stage
  accepts them; `debug_assert` is never the validation mechanism.
- **Short and auditable.** The security-relevant stages (S1–S6) stay small,
  branch-light, and free of business logic, per the Coding Guidelines
  security-boundary rule.
- **One exit per call.** Each call produces exactly one outcome and one
  result category; no path may leave the Guest registers unencoded except the
  invariant path of [02 §3](02-containment-and-error-classification.md).

## 4. Core type contracts

Names below are W06-owned, internal to the P5 stage, and experimental:
they are not ABI, not public API, and may be renamed by a later approved
design. Exact Rust signatures follow the delivered module tree; the contracts
fix semantics.

### 4.1 `HypercallDispatchContext` (type, internal)

```text
Name and stability: HypercallDispatchContext; internal to the P5 dispatch
path; not exported across crate boundaries as API.
Purpose and caller: carries everything one dispatched call needs; created by
the VM-exit handler that received the classified HVC; consumed by
dispatch_hypercall.
Inputs / outputs: constructed from (calling vCPU identity, its
Hypervisor-side security context, raw request register snapshot, handles to
the W03/W04/W05 boundary interfaces). Not Guest-visible.
Preconditions / postconditions: valid only for the duration of one dispatch;
must not be stored, cached across exits, or moved to another pCPU.
State and ownership change: owns nothing persistent; borrows boundary
interfaces.
Concurrency/allocation context: VM-exit context on the calling pCPU; no
blocking, no unbounded allocation; construction must not fault on Guest data.
Errors and failure guarantee: construction failure (e.g., context unavailable)
is an internal error entering the invariant path, never a Guest outcome.
Security/authorization checks: the security context is established from
Hypervisor state only; no field may be initialized from Guest-writable memory.
Logic: a plain data holder; see dispatch_hypercall for use.
Validation: unit review that no Guest-controlled value flows in unchecked.
```

### 4.2 `GuestCallOutcome` (type, internal)

```text
Name and stability: GuestCallOutcome; internal; P5-experimental.
Purpose and caller: the single semantic result of one dispatched call;
consumed by the result encoder (W02 path) and by the category mapper of
02 §4.
Inputs / outputs: exactly one of:
  - Completed { payload }            — allowed operation produced its result
  - Denied { class }                 — Guest-caused controlled rejection,
                                       class one of:
      UnknownCall, UnsupportedFeature, VersionMismatch, Malformed,
      InvalidReference, WrongType, NoAuthority, Revoked, BadState,
      InvalidAddress, GuestDataFault, ResourceExhausted
There is no InvariantViolation variant: invariant failures never become
Guest outcomes (02 §3).
Preconditions / postconditions: produced exactly once per dispatch.
State and ownership change: none.
Concurrency/allocation context: plain value; payload is bounded and fixed
size for the P5 minimal call.
Errors and failure guarantee: not applicable; this type is the error model.
Security/authorization checks: construction is allowed only at S8 or at the
short-circuit points of S1–S6; no other code path may synthesize Completed.
Logic: enumeration only.
Validation: exhaustive-match review; unknown classes are a review failure.
```

The class names are W06's semantic vocabulary. Their wire values, if the W02
ABI assigns distinct numbers, are W02's; the mapping is a table, not logic
scattered through the pipeline.

## 5. Dispatch function contract

### 5.1 `dispatch_hypercall` (function, internal)

```text
Name and stability: dispatch_hypercall; internal to the P5 dispatch path;
not ABI, not public API.
Purpose and caller: the single entry from the classified HVC exit to one
structured outcome; called by the VM-exit handler after the Arch layer has
saved context and classified the trap.
Inputs / outputs: &HypercallDispatchContext (or delivered equivalent) ->
GuestCallOutcome. The caller encodes the outcome to Guest registers via the
W02 result path and records the category.
Preconditions: running at EL2 in the calling vCPU's exit context; context
valid; the trap was classified as the P5 hypercall boundary; no dispatch-
owned lock is held.
Postconditions: exactly one outcome returned; no persistent state mutated
(P5 minimal call); Guest registers unmodified by this function (encoding is
the caller's step); the outcome's category is recordable.
State and ownership change: none persistent; per-call context consumed.
Concurrency/allocation context: VM-exit context on one pCPU; must not block,
sleep, or allocate unboundedly; boundary calls (W03 copy, W04 lookup, W05
check) are used exactly as their contracts allow in this context; the
caller's pCPU identity is never an authorization input.
Errors and failure guarantee: every Guest-caused failure becomes a Denied
outcome (02 §2); allocation or internal failure becomes ResourceExhausted or
the invariant path (02 §3) — never a panic caused by Guest input.
Security/authorization checks: enforces the S0–S8 order; short-circuits at
first failing stage; guarantees no stage after S4 failure executes.
Logic (pseudocode, not production code):

    outcome = match decode(request) {                       # S1 (W02)
        Typed(r)        => r,
        Structural(cls) => return Denied(cls),
    }
    reference = match resolve_reference(request.handle) {   # S2 (W04)
        Live(ref)         => ref,
        Controlled(cls)   => return Denied(cls),            # invalid/stale/
    }                                                       #  destroyed/wrong-type
    if !operation_applies_to(outcome.op, reference.class) { # S3
        return Denied(WrongType);
    }
    grant = match check_authority(ctx.caller, reference,    # S4 (W05)
                                   required_right(outcome.op)) {
        Granted    => grant,
        NoAuthority => return Denied(NoAuthority),
        Revoked    => return Denied(Revoked),
    }
    args = match validate_arguments(outcome, reference) {   # S5 (W03)
        Ok(a)   => a,
        Fault   => return Denied(InvalidAddress or GuestDataFault per W03),
    }
    if !operation_state_holds(reference, outcome.op) {      # S6
        return Denied(BadState);
    }
    payload = execute_query(reference, args)                # S7 (read-only)
    Completed(payload)                                      # S8

Validation: focused unit tests per stage boundary; the acceptance classes of
02 §2; the QEMU integrated evidence of 03's matrix.
```

### 5.2 Minimal permitted operation: object-attributes query (semantic contract)

```text
Name and stability: semantic operation "object attributes query"; the call
identity, number, and register encoding are owned by the W02 design and the
factual ABI artifact; W06 owns only this semantic contract. Internal and
P5-experimental.
Purpose and caller: the one integrated operation of plan step 3; issued by
the Validation Guest to prove the full chain; the only operation W06
authorizes in P5.
Inputs / outputs: one opaque object reference (Guest-supplied handle value);
one Guest-writable result buffer (Guest IPA + length, W03-checked). On
success the buffer receives the object's class, its generation-stable
attribute set as defined by W04, and a fixed-format status word; payload is
bounded by a W02-declared maximum.
Preconditions / postconditions: full S1–S6 chain holds; on success the
buffer is fully written and no other state changed; on any denial the buffer
is not written (no partial writes leak stage information beyond the
designated outcome).
State and ownership change: none (read-only by decision 5 of the README).
Concurrency/allocation context: as dispatch_hypercall; buffer copy uses the
W03 contract only.
Errors and failure guarantee: every denial class of 02 §2 is reachable and
distinguishable; failure leaves object, capability, and Guest state
unchanged.
Security/authorization checks: the buffer is Guest memory, never Host
memory; the reference confers no authority; the authority check precedes any
buffer access.
Logic: S7 reads the W04-resolved object's class/attributes under the access
rules of W04's contract and copies them out via W03; nothing else.
Validation: W06-DV03 (valid chain), W06-DV04 (denial classes), plus the W07
Guest-side scenarios that exercise this contract.
```

## 6. State and lifecycle

The dispatch module owns no persistent state. Its lifecycle is per-call:

```text
trap classified (Arch layer) -> context bound (S0) -> pipeline S1..S7
-> outcome encoded (S8) -> context dropped
```

All mutable state consulted by the pipeline (object table, capability state,
Guest address space) has exactly one authoritative owner in W03/W04/W05; the
pipeline reaches it only through their contract interfaces. If an
implementation is tempted to cache a reference, capability, or mapped buffer
beyond the per-call lifecycle, that is a design violation to stop at review:
cached authority or Guest mappings outliving the call would bypass
revocation and invalidation semantics.

## 7. Concurrency and context model

- The pipeline runs in VM-exit context on the calling pCPU: no sleeping, no
  waiting locks, no unbounded loops, no heavy formatting or logging; logging
  is a category assignment, deferred to W09's integration.
- Cross-pCPU shared state (object table, capability state) is protected by
  the W04/W05 concurrency contracts as delivered. W06 adds no lock and no
  lock-order rule; if a delivered contract cannot be honored inside exit
  context, that contract has a defect — stop and record, do not add a W06
  local lock layer.
- The same Guest may be scheduled on a different pCPU later; nothing in the
  pipeline may key behavior or authority to the current pCPU.
- Re-entrancy: a Guest cannot re-enter dispatch while its own call is being
  serviced (it is not running); inter-VM concurrency is exercised by the W08
  two-pCPU scenarios, not assumed here.
