# P3-W09 Code Contracts — Exception-Local State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; exact offsets and Rust paths are fixed in the implementation
record. The layout is internal representation — never serialized, never
guest-visible. The saved-register layout and vector mechanics remain the
P1-W05 contract; this file defines only the per-CPU bookkeeping W09 owns.

## 1. `ExceptionLocalSlot` (contents contract)

```text
Name and stability: ExceptionLocalSlot — contents owned by W09 from its
    init point; placement/sizing owned by
    [P3-W04](../p3-w04-per-cpu-runtime/README.md) (03 §5.3); internal;
    lifetime = whole boot.
Purpose and caller: the CPU's entire exceptional-path bookkeeping;
    touched by the owning CPU only (rank-1 paths).
Inputs / outputs: fields —
    depth: u8               (nesting depth, ≤ EXCEPTION_NESTING_MAX)
    in_exception: u8        (0/1 flag; redundant with depth > 0, kept
                             explicit for fault-path robustness)
    vector_class: u8        (P1 origin-classification note, opaque here)
    attribution: AttributionSnapshot (rank + fields, §2 of the
                             attribution file; rank-1 valid only)
    fatal_record: FatalRecord (per-CPU fatal context; written once;
                             P1-W07 field set by reference)
    init_epoch: u32         (set at init; detects stale/zeroed slots)
Preconditions / postconditions: after init — depth 0, flag 0, epoch set.
    Invariant — depth ≤ EXCEPTION_NESTING_MAX; fatal_record written at
    most once per boot per CPU; all fields written only by the owner.
State and ownership change: W04 fill → W09 init → live for boot.
Concurrency/allocation context: single-writer (owning CPU); no atomics
    required for owner-local protocol fields; cross-CPU readers are
    diagnostic-grade (plain loads, evidence only) per W06 AP-3.
Errors and failure guarantee: slot corruption (impossible depth/flag
    state, stale epoch) is fatal-class with attribution — never repaired
    silently.
Security/authorization checks: host-only; not guest-reachable.
Logic: structure only.
Validation: W09-DV01 (layout/init review), W09-DV04 (non-overlap).
```

## 2. `AttributionSnapshot` and `FatalRecord`

```text
Name and stability: AttributionSnapshot { rank: u8 {1,2,3}, logical:
    Option<LogicalCpuId>, hardware: Option<HardwareCpuId>, boot: bool,
    lifecycle: Option<CpuLifecycleState> } — internal value.
Name and stability: FatalRecord — the P1-W07 fatal field set (by
    reference to that contract) plus the snapshot and a terminal marker;
    written once, at the CPU's fatal point.
Purpose and caller: the attribution metadata every emission carries
    (README decision 7); the durable per-CPU fatal context.
Preconditions / postconditions: snapshot rank-1 fields present iff rank
    1; FatalRecord.write-once enforced by the terminal marker (a second
    write attempt is an invariant violation — fatal).
State and ownership change: none beyond the owning CPU's writes.
Concurrency/allocation context: plain fields; no allocation (fixed-
    capacity record per the P1-W07 bounded-capture contract).
Errors and failure guarantee: n/a (data).
Security/authorization checks: n/a.
Logic: plain structures.
Validation: W09-DV03/DV05.
```

## 3. `init_exception_slots`

```text
Name and stability: init_exception_slots(areas: &PerCpuSet) -> Result<(),
    InitError> — internal; called once during global initialization
    (single-threaded, pre-release), ordered with the other W07/W08 slot
    inits per the boot sequence.
Purpose and caller: the W04→W09 contents handoff; establishes the init
    epoch.
Inputs / outputs: the W04 per-CPU set; Ok or Err naming CPU/slot.
Preconditions / postconditions: precondition — boot phase Bootstrap.
    Postcondition — every candidate slot matches §1's initial
    postcondition; contents ownership is W09's.
State and ownership change: W04 → W09.
Concurrency/allocation context: single-threaded; no allocation.
Errors and failure guarantee: invalid slot = fatal boot-critical; no
    partial init.
Security/authorization checks: n/a.
Logic: validate alignment; write epoch and zeroed bookkeeping;
    re-validate.
Validation: W09-DV01.
```

## 4. `exception_begin` / `exception_end` (design-level steps)

```text
Name and stability: exception_begin / exception_end — design-level
    bookkeeping steps integrated into the P1-W05 entry path's boundaries;
    implemented as functions per the approved build design.
Purpose and caller: maintain the per-CPU nesting/flag/class state that
    makes exceptional paths diagnosable per CPU; callers: the entry
    path's prologue/epilogue only.
Inputs / outputs: begin — the P1 vector-class note; end — none.
Preconditions / postconditions: begin — rank-1 attribution resolved
    first (a CPU that cannot attribute does not touch the slot; rank 2/3
    paths skip begin/end entirely); depth+1 ≤ MAX else the terminal
    fatal path (no third level). end — depth-1, flag per depth, class
    note cleared.
State and ownership change: own slot only.
Concurrency/allocation context: no allocation; no locks; no cross-CPU
    access; interrupt posture untouched (P1 baseline discipline).
Errors and failure guarantee: violation of the ordering (end without
    begin, depth underflow) is slot corruption — fatal with attribution.
Security/authorization checks: n/a.
Logic (pseudocode):

    exception_begin(class_note):
        slot = current().exception_slot          # rank-1 only
        if slot.depth >= EXCEPTION_NESTING_MAX: terminal_fatal()
        slot.depth += 1; slot.in_exception = 1
        slot.vector_class = class_note
        slot.attribution = snapshot_rank1()      # may itself fault → rank 2/3

    exception_end():
        slot = current().exception_slot
        if slot.depth == 0: fatal_invariant()
        slot.depth -= 1; slot.in_exception = (slot.depth > 0) as u8

Validation: W09-DV01/DV02 (begin/end pairing; nesting bound; role
    coverage boot+secondary).
```

## 5. `EXCEPTION_NESTING_MAX`

```text
Name and stability: EXCEPTION_NESTING_MAX — stage-local constant = 2;
    value and rationale recorded in the implementation record (revisit
    trigger: an approved design needing deeper diagnostic nesting).
Purpose: bounds recursive faults per the P1-W05 non-recursive
    requirement, made per-CPU (README decision 4).
Semantics: depth 1 = the fault being handled; depth 2 = one bounded
    diagnostic nesting level; anything beyond is terminal.
Failure guarantee: exceeding the bound takes the terminal fatal path
    with the interrupted context's attribution from the slot snapshot
    and no further capture.
Validation: W09-DV02 (nesting-bound test via intentional faults).
```

## 6. Non-overlap invariant (normative)

- NO-1: Every location writable on an exceptional path is either (a) in
  the faulting CPU's own `PerCpuArea`, or (b) console bytes emitted
  under the [04 §4](04-code-contracts-attribution-and-logging.md) rules.
  No global exception scratch, no shared save region, no static
  work-area exists.
- NO-2: The P1 baseline's delivered surfaces must satisfy NO-1 after
  W09's relocation of state; the review/audit check is "list every
  writable location reachable from vector entry; each must be
  own-area or console" (DV04; also a W10 audit criterion).
- NO-3: A future P1-contract change that reintroduces shared
  exceptional-path state is an Architecture Change Request, not a local
  edit.
```
