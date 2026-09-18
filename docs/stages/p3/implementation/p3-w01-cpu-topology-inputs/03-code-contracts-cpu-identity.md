# P3-W01 Code Contracts — CPU Identity Types

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

Contracts follow the project function/type template. All names are
design-level identifiers (README decision 6); concrete Rust paths are
reserved to the workspace-owning design. No contract here authorizes runtime
mutation, allocation, or locking.

## 1. `HardwareCpuId`

```text
Name and stability: HardwareCpuId — newtype over u64; internal to the
    hypervisor (not a guest-facing or persistent ABI); name fixed by this
    design as stage-local freedom.
Purpose and caller: opaque carrier of a hardware CPU identity across P3.
    Callers: topology intake, P3-W02 start targeting, P3-W03 registry
    records, P3-W04 per-CPU area headers, P3-W07 targeting (as consumed
    through P3-W03), diagnostics.
Inputs / outputs: constructed only from a validated MpidrValue conversion or
    by the intake adapter from a P2 platform fact; no arithmetic or masking
    operations are exposed.
Preconditions / postconditions: the wrapped value passed validation (for an
    MPIDR-derived value: reserved bits zero, per §2). Equality, ordering,
    and hashing compare the full wrapped value. Two distinct hardware
    identities may have any numeric relationship; nothing may treat the
    value as an index or assume contiguity.
State and ownership change: none (value type).
Concurrency/allocation context: Copy; no allocation; no synchronization.
    Safe to publish cross-CPU after the P3-W05 boot-phase publication.
Errors and failure guarantee: none at use time; all validation happened at
    construction (§2, §4 intake).
Security/authorization checks: none (not guest-reachable; never an
    authorization token).
Logic: `struct HardwareCpuId(u64)` with `new_unchecked` visible only to the
    identity module and the intake adapter; all other construction goes
    through `MpidrValue::to_hardware_id()` or the adapter's validated path.
Validation: host-side unit tests for ordering/equality/display; review that
    no arithmetic accessor exists.
```

## 2. `MpidrValue`

```text
Name and stability: MpidrValue — architecture-side newtype over the raw
    MPIDR_EL1 value; internal; name fixed by this design.
Purpose and caller: the only sanctioned interpreter of MPIDR identity on
    AArch64. Callers: the P1-entry-time boot-CPU observation, the intake
    adapter translating P2 identity facts, diagnostics.
Inputs / outputs: from a raw register read (`read_current()`, unsafe,
    architecture module only) or from a validated u64 delivered by the P2
    fact adapter (`try_from_raw`).
Preconditions / postconditions: bits 63:40 (reserved in the implemented
    architecture revisions targeted by P3) must be zero, else
    `try_from_raw` rejects with MalformedIdentity. Accessors:
    `affinity0/1/2/3` (Aff0 bits 7:0, Aff1 bits 15:8, Aff2 bits 23:16,
    Aff3 bits 39:32), `is_uniprocessor()` (U bit 30), `multithreading()`
    (MT bit 24). `canonical_order_key()` returns the tuple
    (Aff3, Aff2, Aff1, Aff0) used for logical-id ordering. `to_hardware_id()`
    preserves the full validated value.
State and ownership change: none (value type).
Concurrency/allocation context: `read_current()` executes `mrs MPIDR_EL1`
    and is valid only under the P1 entry/runtime contract on the executing
    CPU; it is per-CPU by definition and requires no synchronization.
Errors and failure guarantee: `try_from_raw` rejects reserved-bit pollution
    instead of masking it; masking would silently merge distinct identities.
Security/authorization checks: reserved-bit rejection is the trust boundary
    against malformed platform facts (§6 of the architecture file).
Logic (pseudocode):

    try_from_raw(raw):
        if raw & RESERVED_MASK_63_40 != 0: return Err(MalformedIdentity)
        Ok(MpidrValue(raw))

    canonical_order_key(self):
        (self.affinity3(), self.affinity2(), self.affinity1(), self.affinity0())

Validation: host-side unit tests over fixture raw values (affinity field
    extraction, U/MT bits, reserved-bit rejection, ordering); review that
    `read_current` is the only register access and lives in the
    architecture module.
```

## 3. `LogicalCpuId`

```text
Name and stability: LogicalCpuId — newtype over u16; internal; name fixed by
    this design.
Purpose and caller: stable, dense, boot-stable index of a physical CPU
    within one boot. Callers: every P3 package (bring-up ordering, registry
    indexing, per-CPU table indexing, diagnostics), P4 consumers via the
    P3-W14 handoff.
Inputs / outputs: minted exclusively by topology intake during freeze
    (§4 of the intake contract); exposed as `as_usize()` for indexing and
    `Display` as decimal.
Preconditions / postconditions: invariant — logical ids of a frozen
    TopologyInputs form the exact dense range 0..n-1 where n = entry count
    (n ≤ 8 per README decision 5). The mapping logical→hardware is stable
    across boots with identical declared topology because the assignment
    rule is total and deterministic. A LogicalCpuId is meaningless without
    its TopologyInputs generation; it is never compared across boots by P3
    code.
State and ownership change: none (value type).
Concurrency/allocation context: Copy; no allocation; no synchronization.
Errors and failure guarantee: construction outside intake is prevented by
    module visibility (`new_unchecked` intake-only); no runtime failure mode.
Security/authorization checks: none; a logical id is an index, not a
    capability.
Logic: `struct LogicalCpuId(u16)`; `as_usize()` used only against tables
    whose length equals the topology entry count.
Validation: unit tests that assignment over shuffled input orders yields
    identical logical↔hardware mapping (mapping stability, W01-DV02).
```

## 4. Cross-module invariants

- `MpidrValue::to_hardware_id()` is the only conversion path from an
  architecture identity to the generic boundary; the intake adapter must not
  synthesize `HardwareCpuId` from unvalidated integers. (Adapter-side
  validation lives in [04 §2](04-code-contracts-topology-intake.md).)
- No type in this file has a `Default`, `From<u64>`, or `From<usize>`
  conversion; accidental identity synthesis is a review failure.
- Identity types carry no availability or lifecycle information; coupling
  them would let "present" leak into "online" (the anti-assumption the plan
  forbids).
