# P2-W02 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).

## 1. Logical modules

Module names are stage-local design freedom owned by this design; physical
placement follows the P0-W03 workspace when it exists (platform/discovery
layer, ADR §13 working name `hv-platform`; crate naming pending ADR-054).
`fdt` refers to the W01-validated handle and cursor.

| Module (logical) | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `dt_cells` | Cell-width policy, `reg`/specifier decoding over raw property bytes, bounded string reads | Property value slices (from cursor), cells parameters | Decoded values or `DecodeError` | Node location; semantics of a value |
| `discovery::cpu` | `/cpus` walk: CPU entries, status/enable-method, boot-CPU match | Handle, `dt_cells` | `CpuInventory` facts + boot-CPU relation | Any per-CPU mechanism |
| `discovery::memory` | `/memory@*` walk: RAM banks | Handle, `dt_cells` | `MemoryBank` list | Range claims (W03) |
| `discovery::reserved` | `/reserved-memory` walk + W01 rsvmap passthrough | Handle, rsvmap entries | `ReservedRange` records | Release/reuse policy |
| `discovery::devices` | Singleton fact walks: GICv3, timer, PSCI | Handle, `dt_cells` | GIC/timer/PSCI facts in `FactState` | Any device initialization |
| `discovery::chosen` | `/chosen` + one-hop alias resolution: console, bootargs, initrd artifact | Handle, `dt_cells` | Console/bootargs/boot-artifact facts | Console operation (P1 owns it) |
| `normalize` | Fatal-fact enforcement, `PlatformInfo` assembly, capability summary, determinism | All fact records | `PlatformInfo` or one fatal `DiscoveryDiagnostic` | Interpretation beyond assembly |

Dependency rule: walkers depend on `dt_cells` and the W01 handle only;
`normalize` depends on walkers' fact records — never on the cursor directly.
This keeps every fact independently host-testable and makes the normalized
result the single downstream surface.

## 2. The fact model

One generic state type carries the plan's four required states plus
`NotDiscovered`:

```text
FactState<T>:
  NotDiscovered            -- P2 did not examine this area (PCI, SMMU, ACPI, ITS)
  Absent                   -- examined; the platform does not declare it
  Unsupported(Reason)      -- declared, well-formed, outside P2 support (GICv2, PSCI 0.1)
  Unusable(Diagnostic)     -- declared but required detail missing/malformed/capacity-bound
  Usable(T)                -- declared and complete for P2's needs
```

Rules binding all facts:

- `Unsupported` and `Unusable` always carry a machine-stable reason/diagnostic
  value (reviewable, assertable in tests, renderable by W06); never a bare
  boolean or free string.
- A singleton fact is exactly one `FactState`; collection facts (CPUs, banks,
  reservations, artifacts) are bounded lists plus a summary state for the
  collection as a whole.
- No fact may encode an opinion about another stage's readiness (e.g., the
  GIC fact says nothing about GIC initialization — P6 owns that).

The capability summary (`PlatformCapabilities`,
[04 §2](04-code-contracts-facts.md)) is a projection of facts for
capability-driven queries (ADR-044): it may rename states for consumers
(e.g., `smm`: `NotDiscovered`) but must not change them.

## 3. Core objects and ownership

| Object | Owner | Mutable state | Lifetime |
|---|---|---|---|
| `ValidatedBootDtb` (borrowed) | Boot sequence (W01's publication) | none | boot phase |
| `DiscoveryWorkspace` | Boot sequence (one per boot) | fact lists being filled | construction → normalization |
| `PlatformInfo` | Boot sequence after publication; borrows/copies into consumers later | none after publication | boot phase (P2); extension per W10 |
| Walkers/cursor | Ephemeral | iteration position | one walk |

There is no registry and no global: `DiscoveryWorkspace` is a value, and
`PlatformInfo` is its output. Publication is all-or-nothing: the workspace
either normalizes into a complete `PlatformInfo` or fails with one fatal
diagnostic ([01 §1](01-scope-and-foundations.md)).

## 4. Lifecycle

```text
ValidatedBootDtb (from W01)
   |
   v  normalize(dtb) -> DiscoveryWorkspace
[cpu walk] --fatal condition--> DiscoveryDiagnostic --> boot stop
[memory walk]     (same)
[reserved walk]   (record-only; capacity fatal)
[device walks]    (never fatal)
[chosen walk]     (never fatal)
   |
   v  finalize(workspace)
fatal-fact enforcement -> PlatformInfo published (immutable)
```

Ordering rationale: CPU and memory first because their fatal conditions
should be detected before effort is spent on non-fatal walks (fail-fast);
reserved before devices because reservation recording interacts with
capacity limits (§3 of
[01](01-scope-and-foundations.md)); chosen last because console resolution
may consult `/aliases`, which has no ordering constraint. Determinism does
not depend on this order being intuitive — only on it being fixed.

Failure semantics: any fatal condition aborts with the **first** fatal fact
in pipeline order; nothing is published; the blob and all records are simply
dropped (no unwinding, no cleanup — none is needed since nothing outside the
workspace was touched). Non-fatal fact anomalies accumulate and appear in the
published result (W06/W09 observability).

## 5. Concurrency, allocation, interrupt context

- Single-core boot phase, no locks, no atomics, no IRQ interaction; the
  immutable published `PlatformInfo` needs none even when later stages read
  it from other cores (P3 note; P3-W06 owns any future shared-access
  design).
- No allocation anywhere in W02 (heap does not exist; see
  [01 §3](01-scope-and-foundations.md)). All strings are fixed-capacity
  copies; all lists are fixed-capacity arrays.
- No blocking; total work is O(nodes) with W01's structural caps bounding
  the walk.

## 6. Failure model summary

- Fatal: `CpuInventoryEmpty`, `BootCpuUnmatched`, `NoMemoryBanks`,
  `CapacityExhausted{which}` ([01 §6](01-scope-and-foundations.md)) — boot
  stops with one diagnostic.
- Recorded: `Absent` / `Unsupported(reason)` / `Unusable(diagnostic)` per
  fact, plus skip/anomaly counters.
- Impossible by construction: out-of-bounds access (W01 cursor), panic on
  malformed input (all decode errors are values), nondeterminism
  ([01 §7](01-scope-and-foundations.md)).

## 7. Security model summary

Trust anchors: W01 validation, the DT bindings this design encodes, and the
capacity constants. Untrusted: every DT value. Invariants: no range is
claimed or dereferenced by W02; no board/platform names anywhere; states
never collapse (a consumer cannot accidentally treat `Unsupported` as
`Usable` without a type error). Residual risk: a platform whose *bindings*
are hostile (e.g., a `reg` that overlaps the hypervisor image) is honestly
recorded by W02 — containment is W03's conflict logic, and this division is
carried in the W10 handoff.
