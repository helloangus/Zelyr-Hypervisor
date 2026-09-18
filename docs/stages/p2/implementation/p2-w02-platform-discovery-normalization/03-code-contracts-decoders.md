# P2-W02 Code Contracts — Cell Decoders and Domain Walkers

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design. "Handle" = W01's `ValidatedBootDtb`; "cursor" = W01's
`StructureCursor` ([W01 contracts](../p2-w01-boot-platform-description-intake/03-code-contracts-intake.md)).

## 1. `dt_cells` — semantic decoding primitives

### 1.1 `CellsParams`

```text
Name and stability: CellsParams { address_cells: u8, size_cells: u8 },
  internal.
Purpose and caller: carries the effective cells in force for a node,
  resolved per the policy in
  [01 §5](01-scope-and-foundations.md); used by every reg/specifier decode.
Inputs / outputs: constructed from decoded u32 cell properties with the
  width policy of README Decision 3 (valid 1..=2).
Preconditions / postconditions: values within 1..=2; a decoded width
  outside that range produces UnsupportedPolicy, not a decode.
Errors: n/a (constructor returns Result).
Security checks: width policy bounds every later shift/compose.
Logic: plain record; `CellsParams::resolve(props) -> Result<_, DecodeError>`.
Validation: unit tests on width boundaries (0, 1, 2, 3+).
```

### 1.2 `decode_reg_entries`

```text
Name and stability: dt_cells::decode_reg_entries(raw: &[u8],
  params: CellsParams) -> Result<RegEntries, DecodeError>. Internal; used
  by memory, reserved, and device walkers.
Purpose and caller: interpret a `reg` property as (base, len) pairs in the
  parent's cells encoding; bounded iteration.
Inputs / outputs: raw property value bytes (from cursor, so in-bounds);
  entry width = (address_cells + size_cells) * 4 bytes; output: iterator/
  array of (u64 base, u64 len) with checked composition, plus count.
Preconditions / postconditions: raw length must be a multiple of the entry
  width; zero entries is valid (empty reg) and returned as such; entries
  are returned in DT order (determinism).
State and ownership: none.
Concurrency/allocation context: no allocation (fixed small buffer or lazy
  iterator); pure function.
Errors and failure guarantee: DecodeError::NotCellMultiple (length not a
  multiple of entry width), DecodeError::Policy (width > 2 rejected at
  params construction, never here); input untouched.
Security/authorization checks: composed values use checked shifts/adds;
  u64 composition of two cells cannot overflow (max 64 bits exactly);
  base/len are records, never dereferenced here.
Logic (pseudocode):
    if params.address_cells not in 1..=2 or size_cells not in 1..=2:
        Err(Policy)                                   # unreachable here by contract
    w = (params.address_cells + params.size_cells) * 4
    if raw.len() % w != 0: Err(NotCellMultiple)
    for each chunk of w bytes:
        base = be_compose(chunk[0 .. a*4])            # checked; ≤ 64 bits
        len  = be_compose(chunk[a*4 .. w])
        yield (base, len)
Validation: W02-DV04/DV05 fixtures incl. 1-cell, 2-cell, mixed, and
  non-multiple lengths.
```

### 1.3 `read_string_prop`

```text
Name and stability: dt_cells::read_string_prop(value: &[u8], cap: usize)
  -> Result<BoundedString, DecodeError>. Internal.
Purpose and caller: bounded, NUL-checked copy of a string property into
  fact storage (`stdout-path`, `bootargs`).
Inputs / outputs: property bytes; maximum length (MAX_FACT_STRING);
  output: fixed-capacity string record + a flag whether content was
  truncated at the cap.
Preconditions / postconditions: value must contain at least one NUL;
  content before the first NUL is copied; cap overflow →
  DecodeError::StringTooLong (caller decides fact state).
Errors: MissingNul, StringTooLong; no panic paths.
Security checks: copy length is min(first NUL index, cap) — never trusts
  the property's own length beyond the cursor-validated value bytes.
Logic: straightforward scan-and-copy.
Validation: string fixtures (exact, truncated, unterminated).
```

### 1.4 `compatible_matches`

```text
Name and stability: dt_cells::compatible_matches(value: &[u8], wanted:
  &str) -> bool. Internal.
Purpose and caller: exact-match a DT string-list (`compatible`) property
  against a binding string; used by the device walkers.
Inputs / outputs: raw property bytes (NUL-separated list); wanted binding
  string; output: true iff any element equals `wanted` exactly.
Errors: none (malformed lists simply do not match).
Security checks: element scan bounded by value length; no allocation.
Logic: iterate NUL-terminated elements; byte-equality compare.
Validation: matching fixtures (single, multiple, prefix/suffix near-misses).
```

## 2. Shared record types used by walkers

```text
CpuEntry      { mpidr_aff: u64, status: CpuStatus, enable_method: EnableMethod }
CpuStatus     ::= Enabled | Disabled | UnknownStatus
EnableMethod  ::= Psci | SpinTable | UnknownMethod
MemoryBank    { base: PhysAddr, len: ByteLen }        # page-alignment NOT required here (W03 checks)
ReservedRange { base: PhysAddr, len: ByteLen, source: ReservedSource,
                no_map: bool, reusable: bool }
ReservedSource ::= ReservedMemoryNode | DtbReservationBlock
BootArtifact  { kind: ArtifactKind, base: PhysAddr, end: PhysAddr }
ArtifactKind  ::= Initrd                             # extensible in later designs
```

`PhysAddr`/`ByteLen` are P0 base-crate newtypes (assumed contract;
[W01 §2](../p2-w01-boot-platform-description-intake/01-intake-boundary.md)).
A zero-length bank or reservation is dropped by the walker and counted; a
zero-length boot artifact is `Unusable` (initrd with zero length is
malformed, not absent).

## 3. `discovery::cpu` — CPU inventory and boot-CPU relation

```text
Name and stability: discovery::cpu::walk(handle) -> Result<CpuFacts,
  DiscoveryDiagnostic>. Internal; called by normalize.
Purpose and caller: enumerate /cpus children per the binding policy; match
  the W01 header's raw boot_cpuid_phys against entry reg values.
Inputs / outputs: handle (header value + cursor). Output: CpuFacts {
  entries: BoundedList<CpuEntry, MAX_CPUS>, used: usize, boot_cpu:
  Result<CpuMatch, BootCpuProblem>, skipped_nodes: counter }.
Preconditions / postconditions: pre — handle validated by W01; post — DT
  order preserved; reg parsed with /cpus cells policy (default size_cells 0,
  address_cells 1; explicit values honored within the 1..=2 policy);
  status/enable-method defaults applied per [01 §5](01-scope-and-foundations.md).
State and ownership: fills caller's bounded storage only.
Concurrency/allocation context: no allocation; single-core boot.
Errors and failure guarantee: CapacityExhausted when entries exceed
  MAX_CPUS (fatal upstream); decode problems mark the individual entry
  Unusable-recorded, not fatal ([01 §6](01-scope-and-foundations.md)); boot-CPU
  mismatch is recorded here and enforced fatal by normalize (Decision 4).
Security/authorization checks: reg composition checked; mpidr values are
  records only.
Logic (pseudocode):
    cpus_node = cursor.find_node("/cpus") else return facts{boot: Absent-inventory}
    params = resolve_cells(cpus_node, defaults{addr:1,size:0})?
    for child in cpus_node.children():
        if count == MAX_CPUS: return Err(CapacityExhausted{cpus})
        reg_raw = child.prop("reg") else { entry Unusable; continue }
        (base, _len) = decode_reg_entries(reg_raw, params)?.single()?
             # a CPU reg is exactly one entry; extra entries = Unusable detail
        status   = child.prop("status") -> Enabled|Disabled (default Enabled)
        method   = child.prop("enable-method") -> Psci|SpinTable|Unknown
        push CpuEntry{ base, status, method }
    boot_raw = handle.header.boot_cpuid_phys as u64
    boot_cpu = match entries.find(|e| e.mpidr_aff == boot_raw):
        Some(i) => Ok(CpuMatch{ index: i })
        None    => Err(BootCpuProblem::Unmatched)   # normalize makes this fatal
    return Ok(facts)
Validation: W02-DV02/DV03.
```

## 4. `discovery::memory` — RAM banks

```text
Name and stability: discovery::memory::walk(handle) -> Result<MemoryFacts,
  DiscoveryDiagnostic>. Internal; called by normalize.
Purpose and caller: collect /memory@* banks with root cells; W03's RAM
  input.
Inputs / outputs: handle; output: banks list (MAX_MEMORY_BANKS), dropped_
  zero_size count, DecodeError details.
Preconditions / postconditions: pre — validated handle; post — DT order;
  each bank carries raw base/len as declared; alignment/overlap judgment
  explicitly NOT made here (W03 owns it).
State and ownership: bounded storage fill only.
Concurrency/allocation context: no allocation.
Errors and failure guarantee: CapacityExhausted fatal; malformed reg →
  bank-level Unusable recorded; empty list is NOT an error here (normalize
  enforces the no-banks fatal).
Security checks: checked composition; values are records.
Logic (pseudocode):
    params = resolve_cells(root, defaults{addr:2,size:2})?
    for node in cursor.walk_tree():
        if node.name.has_unit_prefix("memory"):
            reg = node.prop("reg") else { record Unusable; continue }
            for (base, len) in decode_reg_entries(reg, params)?:
                if len == 0 { dropped_zero += 1; continue }
                push MemoryBank{ base, len } (cap-checked)
    return Ok(...)
Validation: W02-DV04.
```

## 5. `discovery::reserved` — firmware reservations

```text
Name and stability: discovery::reserved::walk(handle) -> Result<
  ReservedFacts, DiscoveryDiagnostic>. Internal; called by normalize.
Purpose and caller: merge the two DT reservation sources into one record
  list for W03: (a) W01's validated rsvmap entries, (b) /reserved-memory
  children with their binding properties.
Inputs / outputs: handle; output: ReservedRange list (MAX_RESERVED_RANGES,
  [01 §3](01-scope-and-foundations.md)), source tags, no_map/reusable
  flags, anomaly counts.
Preconditions / postconditions: pre — validated handle and validated
  rsvmap; post — DT order for /reserved-memory children, rsvmap entries in
  blob order before them (fixed order = determinism); entries verbatim; no
  overlap/claim judgment (W03).
State and ownership: bounded storage fill only.
Concurrency/allocation context: no allocation.
Errors and failure guarantee: CapacityExhausted fatal per the §3 asymmetry
  rule; a child without reg → Unusable-recorded entry (present but
  unusable), never silently skipped.
Security checks: values are records; flags are booleans read only as
  declared properties.
Logic (pseudocode):
    for r in handle.reservations.entries():          # source A
        push ReservedRange{ base: r.address, len: r.size,
                            source: DtbReservationBlock, flags: none }
    rsv = cursor.find_node("/reserved-memory")
    if rsv exists:
        params = resolve_cells(rsv, inherit-from-parent)?
        for child in rsv.children():
            reg = child.prop("reg") else { push Unusable entry; continue }
            for (base, len) in decode_reg_entries(reg, params)?:
                push ReservedRange{ base, len, source: ReservedMemoryNode,
                    no_map: child.has_prop("no-map"),
                    reusable: child.has_prop("reusable") }
    return Ok(...)
Validation: W02-DV05.
```

These three walkers constitute the range-producing surface of W02; W03
consumes their records verbatim. The singleton fact walks (GIC, timer, PSCI,
chosen) and the normalization entry point are specified in
[04-code-contracts-facts.md](04-code-contracts-facts.md).
