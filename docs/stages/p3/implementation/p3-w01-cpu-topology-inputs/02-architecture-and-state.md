# P3-W01 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

## 1. Logical modules

W01 is code-bearing but deliberately stateless at runtime: it produces one
immutable view of the machine. Four logical modules cover it. Module names
are design-level identifiers (README decision 6); the workspace-owning design
places them in crates.

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. CPU identity typing | Define and compare hardware and logical identity values; interpret MPIDR fields on AArch64 | Raw MPIDR reads (arch side), logical-id assignment rule | Identity values used by all P3 packages | None (value types only) | Any mutation; any registry of live CPUs |
| B. Topology classification | Map P2 semantic facts to `TopologyClass`, compute rejection reasons | P2 CPU-inventory facts (via the intake adapter), validation rules | Classified entries | None (classification is a pure function of inputs) | Starting or excluding CPUs at runtime |
| C. Topology intake and freeze | Validate, order, assign logical ids, designate the boot CPU, freeze `TopologyInputs` | Classified candidate entries, boot MPIDR, start-capability facts | The frozen `TopologyInputs` aggregate | The aggregate itself (immutable after freeze) | Any post-freeze mutation; any runtime query service |
| D. Topology diagnostics | Emit the enumeration report and intake-failure diagnostics | `TopologyInputs`, intake errors | Log/trace lines per the P0 diagnostics governance | None | Trace-event catalog ownership (P3-W11); log transport (P0-W12) |

Dependency direction is A → B → C → D; nothing in W01 depends on a later P3
package. The platform/architecture integration adapter (README decision 7)
feeds A and B and is itself a thin translation boundary, not a fifth owner of
topology facts.

## 2. Objects and ownership

- **`HardwareCpuId` / `MpidrValue`** — value objects. No owner mutation is
  possible; `Copy`/`Eq`/`Ord` semantics are fixed by contract
  ([03 §2–§3](03-code-contracts-cpu-identity.md)). Construction authority:
  `MpidrValue` may be produced only by the architecture side (from the
  MPIDR register or from validated platform facts); `HardwareCpuId` may be
  produced only by `MpidrValue` conversion or by the intake adapter from P2
  facts.
- **`LogicalCpuId`** — value object minted only by the intake step during
  freeze. The density invariant (`0..n-1`, no holes) is an intake
  postcondition that [P3-W04](../p3-w04-per-cpu-runtime/README.md) relies on.
- **`TopologyInputs`** — the single authoritative machine view. Exactly one
  instance is constructed during boot (global-initialization phase, before
  any secondary is released; the phase ordering is
  [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s contract). After
  freeze it is immutable and safely publishable to all CPUs.
- **Intake errors** — pure data describing the first rejection encountered;
  they never leave partial aggregates behind (§4 below).

There is deliberately **no** `CpuRegistry`, no online-set, and no lifecycle
record in W01; those are P3-W03's objects and must not be pre-created here.

## 3. Lifecycle of the W01 deliverable

W01 has a construction lifecycle only:

```text
intake inputs available (P2 facts + boot MPIDR under the P1 entry contract)
  -> classify candidates (B)                    [pure, repeatable]
  -> validate + assign logical ids + designate boot CPU (C)
       on any rejection: return TopologyError, no aggregate exists
  -> freeze TopologyInputs                      [immutable from here]
  -> emit enumeration diagnostic (D)
  -> aggregate is published to other CPUs by the boot-phase publication
     step owned by P3-W05 (W01 defines immutability, not publication)
```

Failure at any point before freeze leaves no observable artifact: intake is
all-or-nothing. This guarantees downstream packages never see a half-built
topology, which is what makes the construct-validate-freeze-publish model a
substitute for runtime synchronization in this package.

## 4. Concurrency model

- Construction happens on the boot CPU before secondary release; no
  concurrent access exists during intake. The boot-phase ordering that
  guarantees this is owned by P3-W05; W01's obligation is that the aggregate
  is ready before any second CPU can observe it.
- After freeze, the aggregate is read-only shared state. Safe publication to
  other CPUs uses the release/acquire publication of the boot phase
  (P3-W05); W01 states the requirement ("frozen before publication") and
  does not implement the barrier.
- W01 introduces no atomics, locks, or `unsafe` of its own beyond the
  arch-side MPIDR read, which is part of the `MpidrValue` contract
  ([03 §3](03-code-contracts-cpu-identity.md)) and inherits the P1 baseline
  entry context.

## 5. Relation to the runtime vocabulary

The task book requires the P3 stage to "classify present, possible, online,
and failed" CPUs. The division of authority is:

```text
W01 input classification (immutable, decided at intake)
    Present      declared, enabled, well-formed, unique  -> bring-up candidate
    Possible     declared but not usable in this boot    -> identity reserved, never started
    Unavailable  declared but rejected for a reason      -> excluded with diagnostic

P3-W03 runtime states (mutable, owned by the lifecycle registry)
    Present -> Starting -> Initializing -> Online
                        -> Failed (terminal in P3)
```

`Online ⊆ attempted Present` and `Failed ⊆ attempted` are cross-package
invariants W01 states and P3-W03 enforces; neither package may redefine the
other's vocabulary. If the P3-W03 design needs an input class this design did
not anticipate, that is a design-conflict to resolve between the two
designs, not a local extension.

## 6. Security and trust boundaries

P2 platform facts and firmware-declared CPU descriptions are **untrusted
input** for the purposes of intake: malformed identities, duplicate
identities, over-bound inventories, and contradictory enablement facts are
rejected with diagnostics rather than coerced. The boot MPIDR read is trusted
only because it arrives under the P1 entry contract; if it fails to match the
inventory the mismatch is a machine-model violation and intake fails fatally
before bring-up. No guest-originated data can reach W01 at any point; guest
CPU identity is a P4+ concept.
