# P3-W04 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

## 1. The per-CPU model

Every CPU that passes the lifecycle gate owns exactly one `PerCpuArea` and
exactly one runtime stack. The area is page-aligned, self-identifying
(header magic + self pointer), and reachable by its own CPU through a
dedicated register rather than any global lookup. This is the structural
answer to the plan's prohibition: there is no global "current CPU"
because each CPU's locality is a property of the architecture, not of a
searchable table.

```text
PerCpuArea (page-aligned, allocated at global init)
┌───────────────────────────────────────────────┐
│ header: magic, version, self_ptr,             │
│         logical id, hardware id, boot flag,   │
│         stack bounds, install state           │
│ notification-reception slot   (owner: W07)    │  cache-line aligned
│ tlb-reception slot            (owner: W08)    │  cache-line aligned
│ exception/interrupt local slot (owner: W09)   │
│ telemetry counter block       (owner: W11)    │
│ reserved region (scheduler/current-vCPU,      │
│                  P4+; opaque; fill-patterned) │
└───────────────────────────────────────────────┘
Runtime stack (separate allocation, page-aligned, bounds in header)
```

## 2. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Area layout and allocation | define/allocate/validate areas and stacks | topology count, P2 page allocator | installed-ready areas | the allocations (boot-static) | allocator internals (P2); mapping (P1-W08) |
| B. CPU-local access | register mechanism, `current()`, validation | installed areas | locality for every consumer | `TPIDR_EL2` convention (this design's) | consumer protocols |
| C. Installation | gated, exactly-once per-CPU install and readiness signaling | W03 gate, W01 identity, W02 transfer | usable local environment per CPU | install state word in the header | lifecycle transitions (W03); rendezvous (W05) |
| D. Isolation diagnostics | per-CPU identity/address output; cross-CPU read-only table | installed areas | boot diagnostics, test surface | none (derived) | telemetry catalog (W11) |

## 3. Objects and ownership

- **`PerCpuArea`** — owned by its CPU after installation; before
  installation it is owned by the boot-CPU allocator flow. No object in P3
  frees it. The header's `install_state` word (NotInstalled → Installing
  → Installed) records the transition, written by the installing CPU
  itself; this is the one mutable word in the area and it is never written
  after `Installed` (any later write is an invariant violation).
- **Runtime stack** — same ownership arc; bounds recorded in the header at
  allocation time. W02's provisional stack ceases to be used by a
  secondary at install time (ownership per the W02 transfer contract:
  quarantined, never freed, never reused at P3).
- **CPU-local register convention** — owned by this design as a
  stage-local architecture decision (`TPIDR_EL2`, README decision 2);
  every other module must consume it only through the safe accessor.
- **Lookup table** (`logical id → area address`) — global, built at
  allocation time, published with boot-phase data; read-only after
  publication; exists for diagnostics, tests, and cross-CPU *read-only*
  inspection (W12/W13), never for a CPU to find "its own" area (that is
  what the register is for — using the table for self-lookup is a review
  failure because it re-creates the global-current-CPU assumption).

## 4. Lifecycle of the W04 deliverable

```text
global init (boot CPU, pre-release):
    allocate areas + stacks for every topology candidate
    fill headers (identity from TopologyInputs), fill reserved regions
    build lookup table
    [publication is W05's gate; areas are not yet referenced by any CPU]

per CPU (self), when W03 gate says EligibleForLocalInstall (state
Initializing — the boot CPU included):
    write area self-pointer validation, set install state = Installing
    load TPIDR_EL2 = area address
    switch to runtime stack (secondary: replaces W02 provisional stack)
    mark install state = Installed   (release-store; exactly-once via
                                      W03's gate + local CAS on this word)
    emit isolation diagnostic
    signal W05 local-init-complete   (rendezvous takes over)

failure at any install step:
    report failure through the W02/W03 failure paths; the CPU never
    becomes Installed and is never Eligible; its area stays unused
    (quarantined like W02's stacks; no free at P3)
```

## 5. Concurrency model

- **Boot-phase allocation is single-threaded** (global init on the boot
  CPU, before release) — no allocator concurrency is relied upon.
- **After installation, an area is private to its CPU.** W04 defines no
  atomic fields inside the area other than the `install_state` word.
  Cross-CPU visibility needs (notification, TLB requests, counters read
  by tests) belong to the owning packages, which choose their own
  synchronization on top of the reserved slots; W04 guarantees only
  placement and lifetime.
- **Installation ordering:** header fields written before `TPIDR_EL2` is
  loaded; `TPIDR_EL2` loaded before the runtime-stack switch; install
  state stored with release semantics before the W05 signal. The W05
  gate's acquire side therefore observes a fully installed area.
- **No locks** are used or needed by W04; if implementation appears to
  need one, that is a design error to raise (W06 owns lock semantics in
  general, and nothing here has a critical section).

## 6. Boundaries and prohibited shortcuts

- No global `static mut` current-CPU variable, no `Cell`-based global,
  no "current context" global of any kind — the access contract
  ([04 §1](04-code-contracts-cpu-local-access.md)) is the only sanctioned
  locality source.
- No global array whose entries CPUs implicitly share (the lookup table
  is read-only diagnostic infrastructure, not a communication channel).
- No implicit sharing inside an area: two CPUs never touch each other's
  typed slots; the shared surfaces are the ones the owning packages
  design.
- Reserved region is not a backdoor: no P3 code reads or writes it after
  the fill pattern is laid down.
