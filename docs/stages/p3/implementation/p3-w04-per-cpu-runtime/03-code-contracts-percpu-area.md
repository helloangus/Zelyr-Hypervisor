# P3-W04 Code Contracts — Per-CPU Area, Slots, and Stacks

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths and numeric offsets are fixed in the
implementation record against the approved build design. The layout is an
internal representation, not an ABI: it must never be serialized or exposed
to a guest (Coding Guidelines rule on raw struct memory as a contract).

## 1. `PerCpuArea`

```text
Name and stability: PerCpuArea — repr(align=page) region; internal;
    exactly one per topology candidate; lifetime = whole boot (no Drop).
Purpose and caller: the single home of a CPU's local state; accessed by
    its own CPU via `current()` and by diagnostics/tests via the
    read-only table.
Inputs / outputs: constructed by `allocate_all` (§6); fields per §3.
Preconditions / postconditions: page-aligned; header valid before any
    consumer access; reserved region untouched after fill.
State and ownership change: allocator flow → installing CPU (§4 of the
    architecture file); never freed.
Concurrency/allocation context: allocated single-threaded pre-release;
    afterwards private to its CPU except documented read-only surfaces.
Errors and failure guarantee: allocation failure is fatal boot-critical;
    validation failure (§6) quarantines the allocation and fails the
    boot, publishing nothing.
Security/authorization checks: not guest-reachable; no serialization.
Logic: structure only (offsets recorded at implementation):

    PerCpuArea {
        header: PerCpuHeader                  // §3
        slot_notification: NotificationSlot   // owner W07; §5.1
        slot_tlb: TlbReceptionSlot            // owner W08; §5.2
        slot_exception: ExceptionLocalSlot    // owner W09; §5.3
        counters: TelemetryCounters           // owner W11; §5.4
        reserved: ReservedRegion              // P4+; §5.5
    }

Validation: W04-DV01 layout review; W04-DV02 validation tests.
```

## 2. Runtime stack

```text
Name and stability: per-CPU runtime stack — separate page-aligned
    allocation; `RUNTIME_STACK_SIZE` stage-local constant (value and
    rationale recorded in the implementation record; revisit trigger:
    first real workload or the P7 scheduler design).
Purpose and caller: execution stack for a CPU's post-install life;
    installed by the installing CPU itself.
Inputs / outputs: bounds recorded into the owning area's header at
    allocation; the secondary replaces its W02 provisional stack at the
    install point (transfer contract with W02); the boot CPU keeps its
    P1 boot stack during P3 (README decision 6) and its header records
    the boot-stack bounds instead.
Preconditions / postconditions: page-aligned per P2-W04; distinct per
    CPU (isolation invariant); never freed at P3.
State and ownership change: as the area.
Concurrency/allocation context: allocated pre-release; used by exactly
    one CPU.
Errors and failure guarantee: allocation failure fatal; no partial
    installation.
Security/authorization checks: overflow protection is *not* implemented
    (guard pages Reserved); sizing discipline and the recorded limitation
    are the P3 defense — stated, not hidden.
Validation: W04-DV03 distinctness evidence.
```

## 3. `PerCpuHeader`

```text
Name and stability: PerCpuHeader — area-leading structure; internal.
Purpose and caller: self-identification and integrity of the area;
    read by `current()` validation, diagnostics, and audits.
Inputs / outputs: fields — magic (fixed constant), layout_version (u16,
    starts at 1), self_ptr (address of this area), logical:
    LogicalCpuId, hardware: HardwareCpuId, boot: bool,
    stack_bottom/stack_top (addresses), install_state:
    AtomicU8 { NotInstalled=0, Installing=1, Installed=2 }.
Preconditions / postconditions: magic + self_ptr + identity validated at
    install (§5 of the access file) and auditable any time after;
    install_state written only by the owning CPU, never after Installed.
State and ownership change: per architecture §4.
Concurrency/allocation context: identity fields written pre-release;
    install_state is the only atomic.
Errors and failure guarantee: validation failure at install takes the
    failure path (W02/W03 reporting); a post-Installed header mutation is
    an invariant violation (fatal diagnostic path).
Security/authorization checks: n/a.
Validation: W04-DV02.
```

## 4. Cache-line and alignment rules

- The notification and TLB reception slots each start on a cache-line
  boundary (false-sharing defense for the protocols W07/W08 will build);
  the telemetry counter block's per-counter placement follows W11's
  catalog when that design lands — until then, counters are
  u64-aligned slots with reserved capacity, not a frozen catalog.
- All offsets are multiples of their slot alignment; the layout records
  no padding-dependent assumptions (no `offset_of!` chains across
  generic types).
- The layout is versioned via `layout_version`; a layout change is a
  W04 design change, not an implementation detail.

## 5. Slot contracts (reserved for their owners)

### 5.1 Notification-reception slot — owner [P3-W07](../p3-w07-cross-cpu-notification/README.md)

```text
Name and stability: NotificationSlot — fixed capacity and cache-line
    alignment fixed by W04; contents and semantics owned by W07.
Purpose: reserved storage for the cross-CPU notification primitive's
    per-CPU reception state.
Boundaries: W04 zero/fill-patterns it at allocation; W07 initializes it
    at its own init point; W04 code must not read or write it afterwards.
```

### 5.2 TLB-request reception slot — owner [P3-W08](../p3-w08-tlb-shootdown-transport/README.md)

```text
Name and stability: TlbReceptionSlot — same terms as §5.1.
Boundaries: Stage-2 TLB invalidation semantics are P4's; the slot
    reserves transport reception state only (task book Reserved split).
```

### 5.3 Exception/interrupt local slot — owner [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md)

```text
Name and stability: ExceptionLocalSlot — capacity sized for the local
    diagnostic state W09's plan describes; semantics owned by W09 with
    the P1-W05 baseline.
Boundaries: W04 guarantees the slot exists and `current()` is valid from
    the first post-install instruction; W09 guarantees correctness of the
    contents on exceptional paths.
```

### 5.4 Telemetry counter block — owner [P3-W11](../p3-w11-smp-observability/README.md)

```text
Name and stability: TelemetryCounters — u64-aligned counter slots with
    reserved capacity; catalog, rates, and event ids owned by W11 under
    the P0-W13 namespace.
Boundaries: W04 provides placement and lifetime; W11 defines meaning;
    the block is per-CPU private (read aggregation is W11's design).
```

### 5.5 Reserved region — owner: P4+ via [P3-W14](../p3-w14-p4-smp-handoff/README.md)

```text
Name and stability: ReservedRegion — opaque bytes, fill-patterned at
    allocation (0xA5 pattern so accidental use is visible in crashes).
Purpose: capacity for later scheduler/current-vCPU needs without
    defining them (plan requirement).
Boundaries: no type, accessor, or reader is authorized at P3; any
    addition is a W04 design change plus a P4 contract touch via W14.
    This reservation is explicitly NOT a vCPU object; vCPU state is a
    P4/P7 design.
```

## 6. `allocate_all` (boot-CPU allocation)

```text
Name and stability: allocate_all(topology: &TopologyInputs) ->
    Result<PerCpuSet, PerCpuError> — internal; called once during global
    initialization.
Purpose and caller: creates every area + stack, fills headers, builds
    the lookup table; caller is the boot sequence at the point the W05
    design designates (inside global init, pre-release).
Inputs / outputs: frozen topology; returns the set (table + areas) or a
    named error { AllocationFailed, ValidationFailed }.
Preconditions / postconditions: runs on the boot CPU before any
    secondary is released; postconditions — every candidate has a valid,
    filled area and stack; table dense over logical ids; nothing
    published to other CPUs yet (W05's gate).
State and ownership change: P2 allocator ownership → W04; allocations
    are permanent for the boot.
Concurrency/allocation context: single-threaded; page-granular
    allocations per P2-W04; small allocations (if the table needs one)
    per P2-W05.
Errors and failure guarantee: fatal boot-critical on error; no partial
    set is published.
Security/authorization checks: n/a.
Logic (pseudocode):

    allocate_all(topology):
        for cpu in topology.entries():
            area = alloc_pages(area_pages(), align=page)?
            stack = alloc_pages(stack_pages(), align=page)?
            fill header (magic, version, identity from topology,
                         stack bounds, install_state = NotInstalled)
            fill-pattern reserved region
            zero notification/tlb slots
        build lookup table (dense by logical id)
        validate all headers (magic, self_ptr round-trip, bounds sane)
        return Ok(PerCpuSet{ table, areas })

Validation: W04-DV01/DV02 host-side tests with a fake page source;
    allocation-failure and validation-failure paths asserted.
```
