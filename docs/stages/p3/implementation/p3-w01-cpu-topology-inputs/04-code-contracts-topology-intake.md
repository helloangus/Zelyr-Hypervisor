# P3-W01 Code Contracts — Topology Classification and Intake

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

Contracts follow the project function/type template. All names are
design-level identifiers; concrete Rust paths are reserved to the
workspace-owning design. Nothing here authorizes allocation on a secondary
CPU, runtime mutation, or any lifecycle transition.

## 1. `TopologyClass`

```text
Name and stability: TopologyClass — enum { Present, Possible, Unavailable };
    internal; vocabulary shared with P3-W03 (which owns the runtime states).
Purpose and caller: input-time availability classification of a declared
    CPU. Callers: intake, diagnostics, P3-W02 candidate filtering,
    P3-W03 registry seeding.
Inputs / outputs: attached to each classified entry.
Preconditions / postconditions: see §3 for the exact class definitions and
    rejection-reason mapping.
State and ownership change: immutable after classification.
Concurrency/allocation context: Copy; none.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a (classification is derived, not asserted
    by callers).
Logic: classification is a pure function of validated P2 facts plus intake
    validation results (§3).
Validation: W01-DV03 classification unit tests, including every rejection
    reason.
```

## 2. Intake input adapter boundary

```text
Name and stability: the P2-facts-to-P3 intake adapter (a translation
    function, name platform-owned); the P3-side input record type is
    `CpuInventoryFact` (design-level name).
Purpose and caller: converts the documented P2 semantic contract
    ([P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md)) into
    candidate entries without leaking DTB or MPIDR detail into generic
    topology code (ADR-042; README decision 7).
Inputs / outputs: P2 semantics per CPU: declared identity (u64 that must be
    a valid MPIDR-shaped value), enablement/availability statement,
    boot-CPU statement, PSCI start-capability facts (conduit class and
    function identifiers, presence flag). Output: a list of
    `CpuInventoryFact { identity_word: u64, declared_available: bool,
    is_boot_candidate: bool }` plus `StartCapabilityFacts`.
Preconditions / postconditions: the adapter performs no acceptance
    decisions; it translates and forwards. It must preserve every declared
    CPU (dropping one would hide an Unavailable classification behind
    silence).
State and ownership change: none.
Concurrency/allocation context: runs during boot global initialization on
    the boot CPU; allocation only as permitted by the P2 allocator contract
    ([P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md)) for
    boot-phase small objects; failure to allocate is a fatal boot-critical
    failure (P0 panic policy).
Errors and failure guarantee: translation failure (a P2 fact that cannot be
    expressed, e.g. missing identity) aborts intake before classification
    with a diagnostic naming the source fact.
Security/authorization checks: P2 facts are untrusted (see architecture
    §6); the adapter must not repair or default them.
Logic (pseudocode):

    adapt(p2_facts):
        facts = []
        for each declared cpu in p2_facts.cpu_inventory:
            facts.push(CpuInventoryFact{ identity_word: cpu.identity,
                                         declared_available: cpu.enabled,
                                         is_boot_candidate: cpu.is_boot })
        return facts, p2_facts.psci_start_facts

Validation: adapter review against the P2-W10 semantic contract; fixture
    translation tests using P2-shaped fact lists (not DTB bytes).
```

## 3. Classification and validation rules

### 3.1 Class definitions (binding)

- **Present** — identity well-formed (reserved bits zero, §2 of the identity
  contract), unique within the inventory, declared available, and the CPU is
  a bring-up candidate for this boot.
- **Possible** — declared with a usable identity but not usable in this boot
  (firmware-disabled or declared as capacity beyond this boot's presence
  statement). The identity is reserved (no other CPU may later claim it);
  the CPU is never a bring-up candidate at P3.
- **Unavailable** — declared but rejected for a diagnosed reason. Every
  Unavailable classification carries an `UnavailableReason`.

### 3.2 `UnavailableReason`

```text
Name and stability: enum { MalformedIdentity, DuplicateIdentity,
    ConflictingDeclaration, IdentityCollisionWithBoot } ; internal.
Purpose and caller: diagnostics for excluded CPUs; P3-V01 exclusion
    evidence.
Inputs / outputs: produced by classification; rendered by the enumeration
    diagnostic.
Logic: enumerated exhaustively; adding a variant is a reviewed design
    change, not a local patch.
```

### 3.3 `TopologyError` (intake failure)

```text
Name and stability: enum { InventoryBoundExceeded { declared, bound },
    BootCpuUnknown { boot_identity }, BootCpuNotPresent { boot_identity },
    NoStartCapabilityRecorded, AdapterFailure { detail } }; internal.
Purpose and caller: the all-or-nothing intake failure type; caller is boot
    global initialization, which converts it to a fatal boot diagnostic
    (P0 panic policy; P1-W07 crash path).
Preconditions / postconditions: a TopologyError implies no TopologyInputs
    exist.
```

### 3.4 Anti-assumption rules (binding on classification)

1. Declared order is never read: classification and logical-id assignment
   depend only on identity values and enablement facts, never on the
   position of a fact in the P2 list.
2. Identity contiguity is never assumed: adjacent logical ids may map to
   arbitrarily related MPIDR values; no code may compute "the next CPU" by
   identity arithmetic.
3. Present never implies Online: `TopologyClass::Present` only qualifies a
   CPU for bring-up; only P3-W03's runtime transitions can make a CPU
   Online.
4. Absence is represented by omission: a CPU not declared by P2 has no
   entry, and nothing may synthesize one from an expected count.

## 4. `TopologyInputs` (frozen aggregate)

```text
Name and stability: TopologyInputs — immutable aggregate; internal; exactly
    one instance per boot.
Purpose and caller: the single authoritative machine view consumed by
    P3-W02..W05, W10, and (via P3-W14) P4.
Inputs / outputs: built by `build_topology_inputs` (§5); accessors:
    `entries()` in logical order, `count()`, `class_of(LogicalCpuId)`,
    `hardware_of(LogicalCpuId)`, `logical_of(HardwareCpuId)`,
    `boot_cpu() -> (LogicalCpuId, HardwareCpuId)`,
    `counts_by_class()`, `start_capability() -> &StartCapabilityFacts`.
Preconditions / postconditions: after freeze — entries sorted by logical
    id; logical ids dense 0..n-1; exactly one boot-CPU entry; its class is
    Present; all identities unique; count ≤ 8; start-capability facts
    recorded (presence or absence, as delivered by P2). No mutation API
    exists; the type has no interior mutability.
State and ownership change: constructed once during boot global
    initialization; immutable thereafter.
Concurrency/allocation context: allocation at build time only (boot CPU,
    pre-release; P2 allocator contract); reads are lock-free and safe
    cross-CPU after the P3-W05 publication.
Errors and failure guarantee: construction either yields a fully valid
    aggregate or a TopologyError; no partial publication.
Security/authorization checks: all input validation per §3.
Logic: structure only —

    TopologyInputs {
        entries: [CpuTopologyEntry; MAX_PCPU]   // dense, sorted
        boot_logical: LogicalCpuId
        class_counts: { present, possible, unavailable }
        start_capability: StartCapabilityFacts
    }

    CpuTopologyEntry {
        logical: LogicalCpuId
        hardware: HardwareCpuId
        class: TopologyClass            // boot entry is Present
        unavailable_reason: Option<UnavailableReason>
    }

    MAX_PCPU = 8  // README decision 5; raising is a recorded trigger

Validation: W01-DV01/DV02 unit tests (invariant checks over fixture fact
    sets); review that no mutation accessor exists.
```

## 5. `build_topology_inputs`

```text
Name and stability: build_topology_inputs(facts, start_capability,
    boot_mpidr) -> Result<TopologyInputs, TopologyError>; internal; called
    exactly once per boot by boot global initialization.
Purpose and caller: the intake pipeline (architecture §3); caller is the
    boot-CPU global-initialization step.
Inputs / outputs: as above; `boot_mpidr` is the `MpidrValue` observed on
    the executing CPU under the P1 entry contract.
Preconditions / postconditions: runs on the boot CPU before any secondary
    is released (ordering enforced by the P3-W05 phase model, asserted
    here as a documented precondition). Postconditions: those of §4.
State and ownership change: constructs and freezes the aggregate.
Concurrency/allocation context: single-CPU boot-phase context; allocation
    per P2 contract; no synchronization.
Errors and failure guarantee: TopologyError per §3.3; input arguments are
    unchanged on failure.
Security/authorization checks: full validation pass per §3; boot identity
    must match an inventory entry (rules below).
Logic (pseudocode):

    build_topology_inputs(facts, cap, boot_mpidr):
        # 1. identity validation and de-duplication
        seen = {}
        candidates = []
        for f in facts:
            m = MpidrValue::try_from_raw(f.identity_word)?     # Malformed
            if seen.contains(m.to_hardware_id()):
                record DuplicateIdentity; continue_as_unavailable
            seen.insert(...)
            candidates.push(...)

        # 2. classification per §3.1
        for c in candidates:
            if not c.declared_available:            c.class = Possible
            else if c.has_rejection_reason:          c.class = Unavailable(reason)
            else:                                    c.class = Present

        # 3. boot-CPU integration
        boot_hw = boot_mpidr.to_hardware_id()
        if boot_hw not in identities(candidates): return Err(BootCpuUnknown)
        if class_of(boot_hw) != Present:          return Err(BootCpuNotPresent)

        # 4. bound check
        if len(candidates) > MAX_PCPU: return Err(InventoryBoundExceeded)

        # 5. logical assignment: sort by canonical order key of the full
        #    identity (§2 of the identity contract); assign dense ids
        candidates.sort_by(canonical_order_key)
        assign logical ids 0..n-1

        # 6. freeze
        t = TopologyInputs { ... , boot_logical = logical_of(boot_hw) }
        return Ok(t)

Validation: W01-DV02 mapping-stability tests (same facts in shuffled order
    produce identical output); W01-DV04 boot-match tests (unknown boot
    identity, boot identity classified Possible — both must fail with the
    named error); review that step 3 precedes step 4 (an oversized
    inventory that also lacks the boot CPU reports the boot failure —
    the machine-model violation is the more fundamental diagnostic).
```

## 6. Start-capability facts record

```text
Name and stability: StartCapabilityFacts — record { available: bool,
    conduit: ConduitKind (Hvc | Smc as delivered by P2),
    cpu_on_function_id: Option<u32> } ; internal.
Purpose and caller: carries the P2 PSCI start facts W02 needs; W01 neither
    validates their usability nor guesses missing identifiers.
Preconditions / postconditions: recorded verbatim from P2 facts; `No`
    availability is a record, not an error, at W01 (W02 fails closed on
    it).
State and ownership change: immutable inside TopologyInputs.
Errors and failure guarantee: none.
Security/authorization checks: function identifiers are consumed only by
    the W02 start path; W01 must not hardcode QEMU's identifiers (ADR-044).
Validation: review that no PSCI constant appears in W01 code.
```
