# P2-W02 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).

## 1. Package outcome

One call — the normalization pipeline of
[04 §5](04-code-contracts-facts.md) — turns a `ValidatedBootDtb` into a typed
`PlatformInfo`. The outcome is "one normalized result" in the strong sense:
after normalization succeeds, no P2 consumer reads the DTB again for a P2
fact; they read `PlatformInfo` (or, offline, re-derive an equal one). If
normalization fails, it fails on one of the fatal facts of §6 with a
distinct diagnostic, and no partial `PlatformInfo` is published.

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W02 relies on | Failure boundary |
|---|---|---|---|---|
| A1 | `ValidatedBootDtb` + bounds-guaranteed `StructureCursor` + W01 diagnostic style | [W01 design](../p2-w01-boot-platform-description-intake/README.md) | Read-only iteration; in-bounds by construction; boot-CPU raw value; validated reservation list | If W01's handle cannot express a walk W02 needs (e.g., subtree skip), the change is a W01 contract revision — record as a design conflict; do not bypass the cursor |
| A2 | Diagnostic channel and failure classes usable pre-allocation | P0-W12/W14 (assumed, unimplemented) | One-line fact diagnostics; fatal-stop path | If absent, W02 diagnostics have no channel: upstream defect, blocked |
| A3 | Address/length newtypes from the P0 base crate | P0 plans (unimplemented) | `PhysAddr`/`ByteLen` for bank and reservation records | If not delivered, blocked upstream defect; no naked `usize` |
| A4 | W07 will run walkers on host fixtures | [W07 plan](../../plans/p2-w07-offline-dtb-compatibility.md) | All walkers host-runnable with synthesized blobs | If a walker needs target-only state, that is a design violation caught by W02's own host tests |

## 3. Boot-storage policy (no heap)

All discovery state lives in fixed-capacity arrays owned by the normalization
workspace value. Capacities are stage-local constants fixed by this design:

| Capacity | Value | Rationale |
|---|---|---|
| `MAX_CPUS` | 16 | P3 validates up to 8 QEMU CPUs; RK3566 has 4; headroom without being a permanent model |
| `MAX_MEMORY_BANKS` | 8 | QEMU emits one bank per memory slot; real boards 1–2 |
| `MAX_RESERVED_RANGES` | 32 | Covers DTB rsvmap entries (W01-capped at 1024 — see note) plus `/reserved-memory` children with margin |
| `MAX_BOOT_ARTIFACTS` | 4 | initrd plus future firmware artifacts |
| `MAX_FACT_STRING` | 256 | `stdout-path`, `bootargs`, compatible strings copied into fact records |

Note on the reservation asymmetry: W01 can validate up to 1024 rsvmap
entries (DoS bound), but W02 records at most 32 across all sources. This is
deliberate: beyond the cap, normalization fails with `CapacityExhausted`
rather than silently dropping firmware reservations — a platform describing
more than 32 protected ranges is outside P2's reviewed envelope and must
stop with a diagnostic, not truncate. The task book's prohibition on fixed
static arrays becoming the *permanent runtime model* is answered here: these
arrays are the documented temporary pre-heap model (README Decision 1); the
permanent dynamic model is the W05 heap, and consumers after the boot phase
receive data copied or re-derived through it. This boundary is re-reviewed
in W05's design.

## 4. Untrusted-input continuation

W01 guarantees structural safety; W02 adds semantic distrust. Every value
decoded from the DTB (cells, strings, compatible lists) is treated as
hostile: bounded string copies (length-capped, NUL-checked), checked
arithmetic on every composed cell value, and no interpretation of an address
as dereferenceable by W02 itself — discovered ranges are records, not
accesses. Decoding failures are fact-state transitions
(`Unusable`), or fatal per §6 — never panics, never out-of-bounds reads
(impossible through the cursor), never guesses.

## 5. DT semantic decisions owned by this design

The plan leaves parsing semantics open; these are fixed here as stage-local
decisions with rationale (each is revisitable by a later design without
touching the fact model):

1. **Defaults per DT specification are applied and recorded:** missing
   `#address-cells`/`#size-cells` on a node means the spec default (2/2 at
   root; `#size-cells = 0` inside `/cpus` context per binding); missing
   `status` means enabled; missing `enable-method` on an ARMv8 CPU node is
   recorded as `Unknown` (P3's concern, recorded not guessed).
2. **`reg` interpretation is always relative to the parent's cells**
   properties; `/memory@*` and `/reserved-memory` children use root or
   `/reserved-memory` cells respectively; CPU `reg` uses `/cpus`
   `#address-cells` (1 → affinity bits [31:0]; 2 → cell 1 holds Aff3 bits
   [39:32] in its low byte per the ARM binding).
3. **Unit addresses in node names are never parsed for facts** — `reg` is
   authoritative; the name is diagnostic detail only. This avoids the
   classic hex/decimal unit-address mismatch class entirely.
4. **Compatible matching is exact string equality** against the binding
   strings (`"arm,gic-v3"`, `"arm,psci-0.2"`, `"arm,psci-1.0"`,
   `"arm,armv8-timer"`, …); no globbing, no version-numeric heuristics.
5. **A fact's node is located by compatible/name from the root; exactly one
   instance is used** for singleton facts (psci, timer, chosen). Multiple
   candidates: first in DT order wins, additional instances are counted as
   anomalies. `/memory@*` and `/reserved-memory` children are per-instance
   facts and never collapse.
6. **`status` applies to CPU nodes only in P2**; `status` on GIC/timer/PSCI
   nodes is recorded as a fact annotation when present, and a disabled
   singleton is `Absent` with an annotation (the platform says it is not
   for use).

## 6. Fatal-versus-recorded rule

| Condition | Outcome | Rationale |
|---|---|---|
| Zero usable CPU entries | Fatal: `CpuInventoryEmpty` | P2's core handoff to P3; continuing would create an unusable result |
| Boot-CPU value matches no CPU entry | Fatal: `BootCpuUnmatched` | Task book names the boot-CPU relation as required outcome; P3 cannot partition topology without it |
| Zero RAM banks after parsing | Fatal: `NoMemoryBanks` | W03 has no input; boot cannot proceed to a map |
| Capacity exhausted (any list or string) | Fatal: `CapacityExhausted{which}` | Silent truncation of platform facts is forbidden; see §3 |
| Any required-set fact `Unsupported` or `Unusable` (GIC, timer, PSCI, console, individual CPU disabled) | Recorded state; boot continues | These have later-stage owners; P2's job is honest reporting, and capabilities let consumers decide |
| `/reserved-memory` or rsvmap entry beyond capacity | Fatal (see §3) | Protected-range fidelity outranks boot continuation |

Fatal diagnostics reuse W01's style: one line, class, node path class,
structured detail; no blob content.

## 7. Determinism policy

Identical input blob ⇒ identical `PlatformInfo` bits. Enforcement: DT-order
iteration only; no sorting at discovery (records keep DT order; ordering
policy for consumers is P3's concern); first-instance rule for singletons
(§5.5); no allocation (allocator could perturb nothing here anyway, but
no-heap makes host/Guest replay bit-identical); no time, RNG, or environment
input; fixed string copies. The property is host-testable by running the
pipeline twice and comparing records — W02-DV10.

## 8. Layering and capability-driven behavior

No module may name a platform, board, SoC, or machine type. Everything that
varies (GIC present or not, PSCI method, CPU count) is expressed in fact
states and capabilities. The review check for P2-V04 is mechanical: the fact
types contain no platform-name field, and no match arm or constant in the
module set references QEMU/RK3566. "Portability" for fixtures (P2-V09) is
then a property of the fact model, exercised by W07.
