# P5-W04 Architecture, State, and Concurrency

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md).

## 1. Logical modules

| Module | Layer | Responsibility | Owned state / artifacts | Inputs | Outputs | Non-responsibility | Failure boundary |
|---|---|---|---|---|---|---|---|
| H1 Handle codec | Core | Opaque handle value; total decode into {slot, generation, class}; encode on mint | The `ObjectHandle` type and layout constants | Raw u64 (Guest-supplied or internally minted) | Decoded fields or `BadEncoding` | Existence, type truth, authority (table's job) | Decode never panics; it asserts nothing about the table |
| H2 Object table | Core | Slot array; sole authority for existence, class, generation; free-list; lifecycle mutations | The table and its lock | Registration/lookup/destroy/cascade requests | Handles, identity results, invalid causes | The referenced objects themselves (P4 owns); authority (W05) | Table invariants hold after every operation; a violation is `InvariantViolation`, never a Guest-visible oddity |
| H3 Lifecycle operations | Core | register / with_object / destroy / cascade semantics; invalid-cause classification | None beyond H2 | Table state + requests | Outcomes + lifecycle events (for W09 vocabulary) | Who may call them (W05/W06 authorize at the call layer) | Every failure is a typed cause; no partial lifecycle state survives a failure |
| H4 Referent binding | Core/Core boundary | Maps identity values (class + referent ids) onto P4-established VM/vCPU state for `with_object` execution | The identity newtypes usage | ObjectRef + closure | Closure result | P4 object internals | A referent id that P4 cannot resolve is an `InvariantViolation` (table and P4 state disagree — a bug, not a Guest fault) |

Layering: H1–H3 are architecture-independent and host-testable; H4's
bindings touch P4-established state through its established contracts only.
No module branches on board/SoC/QEMU identity.

## 2. Core objects and ownership

| Object | Kind | Owner | Lifetime | Invariants |
|---|---|---|---|---|
| `ObjectHandle` | u64 newtype | H1 | Value | Opaque; zero invalid; fields in range when decode succeeds |
| `ObjectClass` | Enum {Vm=1, Vcpu=2, …} | H2 | Static | Tags unique, nonzero; extension only via approved designs |
| `ObjectRef` | Enum {Vm(VmId), Vcpu{vm, vcpu}} | H2 slot content | Until destroy | Identifies P4-established state; W04 never mutates the referent |
| `Slot` | {Free{generation, next} \| Occupied{generation, class, ref}} | H2 | Table lifetime | Exactly one state; generation advances monotonically per slot; retired slots never allocate |
| `HandleInvalidCause` | Enum {BadEncoding, FreeSlot, GenerationMismatch, ClassMismatch} | H3 | Per rejection | Maps only to W02 `INVALID_HANDLE`; never Guest-visible itself |
| Table lock | Table-wide spinlock | H2 | Table lifetime | All slot-state mutations and `with_object` executions hold it; no Guest memory access or telemetry under it |

Ownership rule: the table owns *existence and type truth*; P4 owns the
referent objects; W05 owns authority about them. Nothing else in the
Hypervisor may keep a slot's truth (no cached validity, no mirror registry).

## 3. Slot lifecycle state machine

```text
              register(class, ref)
  [Free g] ----------------------------> [Occupied g' = g+1, class, ref]
     ^                                        |            |
     |        destroy(handle) matches         |            | lookup(handle, class)
     |        slot+generation                 |            | matches slot+gen+class
     |                                        v            v
     +-------------------------------- [Free g'+1]     identity returned;
                ^       ^                    (stale handles of g now invalid)  closure runs under lock
                |       |
   cascade-destroy of owner Vm:    repeated destroy of same handle:
   contained Vcpu slots freed      handle names (slot, g): slot now
   first (each g++), then the Vm   Free at g+1  => GenerationMismatch
   slot freed (g++)                (or FreeSlot if shape check fails first)
```

Terminal rules:

- A retired slot (generation would wrap) is `Free` and permanently
  unallocatable; its only reachable state transition is none.
- Every state change bumps the generation, so every pre-change handle is
  stale afterwards — there is no "reuse with same generation" path.
- Capacity exhaustion at register is `TableFull` (a Hypervisor capacity
  condition → W02 `RESOURCE_EXHAUSTED`), never a handle-validity statement.

## 4. Concurrency model

- **Lock:** one table-wide spinlock (P3-W06 discipline). All lifecycle
  mutations and `with_object` executions hold it. Lookup-only paths may run
  under the same lock in v0 (no read-write refinement — Reserved).
- **Hold-time bound:** no Guest-data copy, no telemetry emission, no
  allocation under the lock; the `with_object` closure's cost is bounded by
  the caller's (W06) operation contract — closures that copy Guest data take
  the copy outside the lock by working on identity-derived state, not on the
  slot.
- **Cross-CPU:** two pCPUs may concurrently lookup/destroy distinct handles;
  the lock serializes slot-state transitions, so a destroy is either fully
  before or fully after any overlapping lookup — no torn states exist.
  W08's two-pCPU scenarios may rely on exactly this guarantee and nothing
  stronger.
- **Re-entrancy:** no lifecycle operation re-enters the table from within a
  closure (a re-entrant call is an `InvariantViolation`; documented because
  it is a programming error, not a Guest-triggerable condition).
- **IRQ context:** registration/destroy occur in bootstrap or VM-lifecycle
  context; the lock is IRQ-safe per the P3-W06 rules for registry-class
  locks; hypercall-path operations inherit the dispatch context rules of
  W02/W06.

## 5. Security model

- **Opaqueness:** handles carry no Host address, offset, or table location;
  decode reveals only what a Guest already supplied. Logging and telemetry
  never include referent identities (Host-side detail); W09 redaction
  consumes handle values only under its policy.
- **Uniform rejection:** invalid shapes, staleness, and type mismatch are
  indistinguishable to the Guest (decision 8 of
  [01](01-scope-and-foundations.md)); there is no oracle for probing slot
  state beyond valid/invalid.
- **No authority:** the table cannot grant, imply, or leak rights; a valid
  handle plus absent authority is a W05 denial, and no table path short-
  circuits that check (structural: rights do not exist in these types).
- **No referent mutation:** lifecycle operations change only table truth;
  P4 objects are stopped/started by their own contracts. The table is never
  the reason a VM stops — only the reason a handle stops being valid.
- **Fail-closed invariants:** table invariant violations (a destroyed handle
  validating, a class tag changing under an existing handle) are
  Hypervisor bugs → `InvariantViolation` escalation (W02 model), never
  Guest-visible oddities.
