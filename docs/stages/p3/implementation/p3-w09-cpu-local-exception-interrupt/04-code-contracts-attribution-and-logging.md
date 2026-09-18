# P3-W09 Code Contracts — Attribution and Concurrent Logging

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design.

## 1. `CpuAttribution`

```text
Name and stability: CpuAttribution — enum { Local(AttributionSnapshot),
    MpidrOnly { hardware: HardwareCpuId, logical: Option<LogicalCpuId> },
    Unknown } — internal value; produced by resolve_attribution.
Purpose and caller: the identity content of every exceptional-path
    emission; callers: the diagnostic paths, the fatal path ([P1-W07]
    field set), W11 surfaces.
Preconditions / postconditions: n/a (value).
Concurrency/allocation context: fixed capacity; no allocation.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: plain enum.
Validation: W09-DV03.
```

## 2. `resolve_attribution` (the three-rank rule)

```text
Name and stability: resolve_attribution() -> CpuAttribution — internal;
    executed once per exceptional path, as early as the entry path's
    contract allows.
Purpose and caller: correct CPU identity on supported boot and secondary
    exceptional paths (P3-V09); caller: entry-path diagnostic/fatal
    integration.
Inputs / outputs: none; the attribution value.
Preconditions / postconditions: pure read path — no writes of any kind
    before a rank is resolved (a wrong-write here could corrupt another
    CPU's area if TPIDR_EL2 is stale/foreign).
State and ownership change: none.
Concurrency/allocation context: register reads and acquire-loads only;
    no allocation; no locks; must not itself take locks before rank
    resolution (it runs on paths where locks may be poisoned).
Errors and failure guarantee: rank 2/3 degrade honestly; the function
    cannot fail — it returns Unknown instead.
Security/authorization checks: n/a (host-only).
Logic (pseudocode):

    resolve_attribution():
        # Rank 1: locality register validated against the area header
        ptr = read_register(TPIDR_EL2)                # unsafe: arch boundary
        if header_valid(ptr, expect_installed = true):
            slot = current().exception_slot
            return Local(snapshot_rank1())            # logical, hardware,
                                                      # boot, W03 lifecycle
        # Rank 2: MPIDR fallback (pre/during install; W02 mechanism)
        mpidr = MpidrValue::read_current()            # unsafe: arch boundary
        hw = mpidr.to_hardware_id()
        return MpidrOnly { hardware: hw,
                           logical: topology.logical_of(hw) }
        # Rank 3 falls out of MPIDR read/lookup failure handling:
        #   any fault during the reads above is contained by the entry
        #   path's rank-3 landing (single unknown line, terminal).

Validation: W09-DV03 (rank table tests with injected header states).
```

## 3. `fatal_with_attribution` (P1-W07 integration)

```text
Name and stability: fatal_with_attribution(fields: P1FatalFields) ->
    (!) — internal; diverges; the P3-side integration of the P1-W07
    fatal path.
Purpose and caller: CPU-attributed fatal diagnostics that cannot
    deadlock and cannot recurse; caller: terminal paths across P3
    (invariant violations, corruption, terminal nesting) and any
    approved fatal call site.
Inputs / outputs: the P1-W07 field set (by reference to that contract)
    plus the attribution value; no return.
Preconditions / postconditions: precondition — attribution already
    resolved (rank recorded). Postconditions — the per-CPU FatalRecord
    is written first (lock-free, CPU-private, write-once); console
    emission follows the CR-5 strategy (§4 rules below); the CPU ends in
    the terminal state. The P1-W07 bounded/non-recursive properties hold
    per CPU.
State and ownership change: own slot's fatal record; terminal marker.
Concurrency/allocation context: no allocation (P1-W07 bounded capture);
    the only lock ever attempted is Diagnostics, bounded try-lock; no
    other lock, no wait, no WFE (CR-5/MIS-11).
Errors and failure guarantee: this is the failure guarantee; a failed
    console acquisition degrades to the marked best-effort emission, and
    the slot record remains the durable source.
Security/authorization checks: n/a (host-only; invariant violations are
    hypervisor-internal by definition at P3).
Logic (pseudocode):

    fatal_with_attribution(fields):
        slot = own_slot_if_rank1()                    # rank 2/3: skip
        if slot: write FatalRecord(fields, attribution, TERMINAL)
        emit_fatal(fields, attribution)               # §4 rules; bounded
                                                      # try-lock, fallback
                                                      # with marker
        terminal_loop()                               # arch-defined park

Validation: W09-DV05 (attribution present in every captured report;
    write-once; bounded behavior under console contention).
```

## 4. Concurrent logging rules (normative)

- CL-1 (normal diagnostics): any non-fatal diagnostic emission acquires
  the console's Diagnostics-class lock (W06 §2 flavor choice per CR-2;
  upgrade to the irq-save flavor when interrupt sources arrive) and
  emits complete lines; line construction happens before acquisition
  where possible to keep the critical section minimal.
- CL-2 (fatal path): the fatal emitter attempts the same lock with a
  bounded try-lock spin; on failure it emits best-effort without the
  lock, prefixing and suffixing each line with an explicit interleave
  marker and the attribution fields — bounded noise, never silence,
  never a hang (README decision 6).
- CL-3 (attribution mandatory): every exceptional-path line carries
  logical id (as available per rank), hardware id (as available),
  boot/secondary role, and lifecycle state (rank 1) — the field set
  W11's contract needs. Unattributed exceptional-path output is a
  review failure.
- CL-4 (handler context): emission from exception context obeys W06
  CR-4 — no allocation, no non-Diagnostics locks, bounded work; heavy
  formatting is deferred to fixed-capacity buffers prepared lock-free
  before acquisition.
- CL-5 (no new sinks): the P3 console is the P1-baseline console; W09
  adds no transport, storage, or remote sink (P1-W07 out of scope;
  W11 owns catalog/formats).

## 5. Rank-2/3 landing (normative behavior)

A CPU that cannot resolve rank 1 emits at most one console line
(attribution per rank), writes no per-CPU state, and takes the terminal
path (rank 3) or continues its W02-defined flow (rank 2 during entry,
where W02's outcome map/mailbox rules govern). This keeps pre-install
faults diagnosable without letting unattributed CPUs touch per-CPU
state — the guardrail from the P3-W02 identity rule, extended to
exceptional paths.
