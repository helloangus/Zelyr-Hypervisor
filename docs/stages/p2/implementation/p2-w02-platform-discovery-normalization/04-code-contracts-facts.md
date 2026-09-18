# P2-W02 Code Contracts — Fact Model, Singleton Walkers, Normalization

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code.

## 1. `FactState` — the five-state model

```text
Name and stability: FactState<T> (enum), internal to P2; the W10 handoff
  records the states semantically for P3/P4. Stability: stage-local; the
  state *set* is contract-grade (P2-C02) — changing it is a design change.
Purpose and caller: every singleton fact and collection summary uses it so
  consumers cannot collapse states (README Decision 2).
Inputs / outputs: payload T for Usable; Reason for Unsupported;
  FactDiagnostic for Unusable.
Preconditions / postconditions: construction sites are the walkers only;
  consumers get read access.
Concurrency/allocation context: no allocation; payload by value.
Errors: n/a (this is the diagnostic carrier).
Security/authorization checks: type-level separation — a `Usable` fact
  cannot be produced by a path that did not fully validate the fact's
  required details.
Logic:
    enum FactState<T> {
        NotDiscovered,
        Absent,
        Unsupported(FactReason),   // GicV2, PsciV01, CellWidth, ...
        Unusable(FactDiagnostic),  // MissingMethod, BadReg, AliasUnresolvable, ...
        Usable(T),
    }
Validation: W02-DV08 asserts distinguishability and non-collapsibility.
```

## 2. Fact records and the capability summary

Singleton fact payloads (each carried in a `FactState`):

```text
GicFact        { distributor: PhysSpan, redistributors: BoundedList<PhysSpan, 2>,
                 redistributor_stride: ByteLen, gicd_version_hint: none-claimed,
                 irq_cells: u8 }        # from #interrupt-cells; expected 3
TimerFact      { always_on: bool, ppi_specifier_count: u8 }   # raw; no GIC interpretation
PsciFact       { method: PsciMethod, version_class: PsciVersionClass }
                 # PsciMethod ::= Smc | Hvc; version class 0.2 or 1.0
ConsoleFact    { stdout_path: BoundedString, resolved_node: Option<NodePath> }
BootargsFact   { args: BoundedString }
```

`PlatformCapabilities` — the ADR-044 query surface, a pure projection of
`PlatformInfo` facts:

```text
cpu_topology : Usable(count) | states as per facts        # count = enabled entries
gic          : Usable(GicV3{regions}) | Unsupported(GicV2) | Absent | Unusable | NotDiscovered
timer        : per FactState
psci         : per FactState
console      : per FactState
pci, smmu_iommu, acpi : NotDiscovered (constant in P2, by design)
```

Contract rule: `NotDiscovered` constants exist exactly for areas the task
book puts outside P2 scope; adding any other value there requires a design
change. This prevents false "absent on this platform" claims (P2-V04,
W02-DV08).

## 3. Singleton walkers (`discovery::devices`, `discovery::chosen`)

### 3.1 `gic::walk`

```text
Name and stability: discovery::devices::gic::walk(handle) -> FactState<
  GicFact>. Internal; never fatal.
Purpose and caller: locate the interrupt controller; record GICv3 regions
  or an honest Unsupported/Absent state.
Inputs / outputs: handle + cursor. Walks all nodes; candidates = nodes with
  a `compatible` matching "arm,gic-v3" (Usable path) or known GICv2
  binding strings "arm,gic-400"/"arm,cortex-a15-gic"/"arm,cortex-a9-gic"
  (Unsupported path).
Preconditions / postconditions: pre — validated handle; post — DT-order
  first-candidate rule ([01 §5.5](01-scope-and-foundations.md)); extra
  candidates counted as anomalies; ITS children recorded as NotDiscovered
  sub-fact counters (README Decision 7).
State and ownership: fills the caller's fact slot only.
Concurrency/allocation context: no allocation.
Errors: none returned — every outcome is a FactState; internal decode
  problems yield Unusable(FactDiagnostic::BadReg|BadCells).
Security/authorization checks: reg decoded with parent cells; regions are
  records; #redistributor-regions/#redistributor-stride clamped to binding
  defaults when missing; a stride of 0 is Unusable (binding violation).
Logic (pseudocode):
    for node in cursor.walk_tree():
        compat = node.prop("compatible") else continue
        if compatible_matches(compat, "arm,gic-v3"):
            params = resolve_cells(node.parent, root_defaults)?
            reg = node.prop("reg") else return Unusable(BadReg)
            spans = decode_reg_entries(reg, params)?
            gicd = spans[0]; gicrs = spans[1..]      # 1 + #redistributor-regions expected
            n_rsr = node.prop_u32("#redistributor-regions") unwrap_or 1
            stride = node.prop_u32("redistributor-stride") unwrap_or (2*64KiB)
            if gicrs.len() != n_rsr: return Unusable(RegionCountMismatch)
            if stride == 0: return Unusable(BadStride)
            if node.prop_u32("#interrupt-cells") != Some(3): return Unusable(BadIrqCells)
            return Usable(GicFact{ ... })
        if compatible_matches(compat, any GICv2 binding):
            gic_seen_v2 = true                        # continue scanning for a v3
    return if gic_seen_v2 { Unsupported(FactReason::GicV2) } else { Absent }
Validation: W02-DV06.
```

### 3.2 `timer::walk` and `psci::walk`

```text
Name and stability: discovery::devices::timer::walk / psci::walk ->
  FactState<TimerFact|PsciFact>. Internal; never fatal.
Purpose and caller: record ARMv8 timer presence and PSCI version/method;
  no mechanism is touched (Out of Scope).
Timer logic: candidate = node whose compatible matches "arm,armv8-timer";
  record always_on flag and the interrupt count of `interrupts` divided by
  the parent #interrupt-cells (raw, 4 expected per binding; mismatch →
  Unusable detail, still recorded, not fatal). Compatible
  "arm,armv7-timer" → Unsupported(TimerV7). Else Absent.
Psci logic: candidate = /psci node. compatible list → version class:
  contains "arm,psci-1.0" → V1_0; else "arm,psci-0.2" → V0_2; else
  "arm,psci" (0.1) → Unsupported(PsciV01) per README Decision 5.
  method prop "smc"|"hvc" → Usable{method}; missing/other →
  Unusable(MissingMethod|BadMethod). No node → Absent.
Concurrency/allocation context: no allocation; single walk each.
Validation: W02-DV06.
```

### 3.3 `chosen::walk`

```text
Name and stability: discovery::chosen::walk(handle) -> ChosenFacts {
  console: FactState<ConsoleFact>, bootargs: FactState<BootargsFact>,
  artifacts: BoundedList<BootArtifact, MAX_BOOT_ARTIFACTS> }. Internal;
  never fatal (README Decision 6).
Purpose and caller: /chosen facts plus initrd boot artifact for W03
  protection.
Inputs / outputs: handle + cursor + dt_cells.
Preconditions / postconditions: pre — validated handle; post — strings
  bounded per [01 §3](01-scope-and-foundations.md); alias resolution at
  most one hop.
State and ownership: bounded storage fill only.
Concurrency/allocation context: no allocation.
Errors: none returned; all outcomes are fact states.
Security/authorization checks: initrd start/end composed with checked
  arithmetic; start > end → Unusable artifact record (W03 will reject it;
  W02 records honestly); string caps enforced by read_string_prop.
Logic (pseudocode):
    chosen = cursor.find_node("/chosen") else return all-Absent facts
    if let p = chosen.prop("stdout-path"):
        (path_part, _options) = split_at_first_colon(p)
        resolved = if path_part starts with '/': Some(path_part)
                   else resolve_alias("/aliases", path_part)   # one hop, else None
        console = Usable{stdout_path: path_part, resolved_node: resolved}
            # unresolved alias → resolved_node None; console stays Usable as a
            # recorded path, W06 renders resolution state (Decision 6 rationale)
    if let b = chosen.prop("bootargs"): bootargs = Usable{copy(b)}
    if let (s, e) = (chosen.prop_u64_pair("linux,initrd-start", "linux,initrd-end")):
        artifacts.push(if s <= e { Usable-range } else { Unusable-flagged })
    return facts
Validation: W02-DV07.
```

## 4. `PlatformInfo` and normalization

### 4.1 `PlatformInfo`

```text
Name and stability: PlatformInfo, the P2 normalized-result type; consumed
  by W03/W06 and recorded semantically for P3/P4 via W10. Stability:
  stage-local; field set is the P2 fact contract (P2-C01).
Purpose and caller: the one typed result; construction only via normalize.
Inputs / outputs: composition of all fact records above plus
  PlatformCapabilities and skip/anomaly counters.
Preconditions / postconditions: exists only for inputs that passed the
  fatal-fact rule; immutable after construction; DT-order lists.
Concurrency/allocation context: no heap; fixed-capacity members.
Errors: none post-construction.
Security checks: contains no platform names; no raw DTB references (only
  decoded records), so consumers cannot reparse through it.
Logic:
    struct PlatformInfo {
        cpus: CpuFacts,                 // incl. boot_cpu match
        memory_banks: BoundedList<MemoryBank, MAX_MEMORY_BANKS>,
        reserved: BoundedList<ReservedRange, MAX_RESERVED_RANGES>,
        gic: FactState<GicFact>, timer: FactState<TimerFact>,
        psci: FactState<PsciFact>, console: FactState<ConsoleFact>,
        bootargs: FactState<BootargsFact>,
        artifacts: BoundedList<BootArtifact, MAX_BOOT_ARTIFACTS>,
        counters: DiscoveryCounters,    // skipped nodes/props, anomalies
        capabilities: PlatformCapabilities,   // projection, §2
    }
Validation: W02-DV09.
```

### 4.2 `normalize::run`

```text
Name and stability: normalize::run(handle: &ValidatedBootDtb) -> Result<
  PlatformInfo, DiscoveryDiagnostic>. The single W02 entry point; called
  once per boot; called fresh per fixture by W07.
Purpose and caller: run walkers in fixed order, enforce the fatal-fact
  rule, assemble PlatformInfo.
Inputs / outputs: W01 handle. Output: the normalized result or one fatal
  diagnostic.
Preconditions / postconditions: pre — handle validated; post — all-or-
  nothing publication ([02 §3](02-architecture-and-state.md)); identical
  input → identical output ([01 §7](01-scope-and-foundations.md)).
State and ownership: owns a DiscoveryWorkspace on its stack; publishes the
  PlatformInfo value.
Concurrency/allocation context: single-core boot; no allocation; no locks.
Errors and failure guarantee: fatal set of
  [01 §6](01-scope-and-foundations.md) — CpuInventoryEmpty,
  BootCpuUnmatched, NoMemoryBanks, CapacityExhausted{which}; first in
  pipeline order; nothing published on Err.
Security/authorization checks: fatal rule enforcement; capabilities
  projection consistency (a Usable capability must have a Usable fact).
Logic (pseudocode):
    cpus     = discovery::cpu::walk(handle)?
    banks    = discovery::memory::walk(handle)?
    reserved = discovery::reserved::walk(handle)?
    gic      = devices::gic::walk(handle)        # never fatal
    timer    = devices::timer::walk(handle)
    psci     = devices::psci::walk(handle)
    chosen   = chosen::walk(handle)
    if cpus.entries.usable_count() == 0: return Err(CpuInventoryEmpty)
    if cpus.boot_cpu is Err: return Err(BootCpuUnmatched)
    if banks.list.is_empty(): return Err(NoMemoryBanks)
    return Ok(assemble(cpus, banks, reserved, gic, timer, psci, chosen))
Validation: W02-DV01 (pipeline), DV02–DV07 (walks), DV09 (assembly rules),
  DV10 (determinism).
```

## 5. Explicitly unauthorized interfaces

No walker may call another walker's node search (single traversal ownership
per walk); no fact may expose raw DT byte slices; `PlatformCapabilities`
must not be mutable; no Debug/Display may render property values beyond
path/class summaries (log-content hygiene, as in W01). W06 renders facts
through its own design; W02 provides the types only.
