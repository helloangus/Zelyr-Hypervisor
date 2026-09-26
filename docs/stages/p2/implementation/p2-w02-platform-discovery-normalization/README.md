# P2-W02 Platform Discovery and Normalization — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Capability-driven discovery of P2-required platform facts and one
normalized, host-usable platform result, required by
[P2-W02](../../plans/p2-w02-platform-discovery-normalization.md).  
**Owner/change context:** P2-W02 implementation handoff.  
**Supersedes:** None.

Read the [current-baseline amendment](../p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)
before the original assumed contracts or pseudocode below. It reconciles the
completed P1 handoff, source placement and DT specification corrections.
Actual implementation and evidence are linked from the [implementation index](../README.md).

## Purpose and use

This is the implementation-level design for P2-W02. It converts the bounded
work-package plan into a code-bearing design with two halves: semantic
decoders and per-domain walkers that read the W01-validated DTB (CPU
inventory and boot-CPU relation, RAM banks, firmware reservations, GICv3
description, timer, PSCI, `/chosen`/console, boot artifacts), and a
normalization layer that produces exactly one typed `PlatformInfo` result
with an explicit five-state capability model. It deliberately does **not**
initialize any device (GIC, timer, UART are later-stage mechanisms), does not
execute PSCI, does not start APs, does not construct a memory map or claim
physical ranges (W03), does not parse any configuration format, and does not
use dynamic allocation (the allocators do not exist yet in boot order —
[01 §3](01-scope-and-foundations.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the linked supporting file needed for its assigned step. Before editing
it must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W02 plan](../../plans/p2-w02-platform-discovery-normalization.md).

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed W01/P0 contracts, the no-heap boot-storage policy, determinism policy, and the fatal-versus-recorded fact rules. |
| [02-architecture-and-state.md](02-architecture-and-state.md) | Logical modules, the fact/capability model, `PlatformInfo` composition, lifecycle, ownership, and the single-core concurrency context. |
| [03-code-contracts-decoders.md](03-code-contracts-decoders.md) | Contracts and pseudocode for DT cell decoding and the CPU/memory/reserved-memory walkers. |
| [04-code-contracts-facts.md](04-code-contracts-facts.md) | Contracts and pseudocode for the fact-state model, GIC/timer/PSCI/chosen walkers, `PlatformInfo` normalization, and the capability summary. |
| [05-implementation-workflow.md](05-implementation-workflow.md) | The ordered implementation steps with acceptance and failure handling. |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | The validation matrix (P2-V03/P2-V04), error/security/observability model, and handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W02 plan](../../plans/p2-w02-platform-discovery-normalization.md) → this
design → Coding Guidelines. Binding constraints:

- ADR-042: DTB is the discovery source and everything becomes a unified
  `PlatformInfo`-shaped intermediate representation; ADR-043/052: Core never
  branches on QEMU/RK3566/board names; ADR-044: consumers query
  capabilities, not platform names. The design enforces this by making the
  normalized result the only output — walkers vanish behind it.
- ADR-025/031/032 context: ACPI, PCI/SMMU discovery, and GIC bring-up are not
  P2 facts; the capability model must represent their absence honestly as
  "not discovered", never as "absent on this platform"
  ([04 §2](04-code-contracts-facts.md)).
- The task book requires P2 not to freeze concrete `PlatformInfo` APIs in
  plans; the type set below is fixed by this design as stage-local decisions
  with recorded rationale. W10 records the P3/P4-visible semantic contract.
- DT parsing choices the plan leaves open (cell-width support, matching
  rules, unknown-node treatment, PSCI version policy) are stage-local
  decisions owned here with rationale
  ([01 §5](01-scope-and-foundations.md)).

Classification. **Required:** the seven domain walkers, the cell decoders,
the five-state fact model, `PlatformInfo`/`PlatformCapabilities`
construction, deterministic behavior, and host testability of everything.
**Reserved** (recorded triggers, no P2 implementation): cell widths above 2,
PSCI 0.1 function-ID parsing, alias trees deeper than one hop, NUMA/socket
topology facts, additional capability types, device discovery beyond the
required set, ACPI as a source. **Out of Scope:** GIC/timer/UART/PSCI
execution and initialization, AP startup, memory-map construction (W03),
allocation (W04/W05), inspection output (W06), offline fixture checking
(W07), and any Orange Pi 3B runtime behavior.

## Requirement-to-design mapping

The tracked sources define P2-B01–B08 and P2-C01–C03 at group granularity
only; the rows below are this design's reviewable enumeration from the plan's
scope wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-B01 | CPU inventory: `/cpus` children with `reg` (MPIDR affinity), `status`, `enable-method`, capacities bounded | [03 §3](03-code-contracts-decoders.md) | P2-V03 (W02-DV02) |
| P2-B01 | Boot-CPU relation: header `boot_cpuid_phys` matched against inventory; no match is fatal | [01 §6](01-scope-and-foundations.md), [04 §4](04-code-contracts-facts.md) | P2-V03 (W02-DV03) |
| P2-B02 | RAM banks: `/memory@*` `reg` parsed with root cells; zero/empty banks dropped with count | [03 §4](03-code-contracts-decoders.md) | P2-V03 (W02-DV04) |
| P2-B03 | Firmware reservations: `/reserved-memory` children (`reg`, `no-map`, `reusable`) recorded verbatim | [03 §5](03-code-contracts-decoders.md) | P2-V03 (W02-DV05) |
| P2-B04 | GIC description: GICv3 distributor/redistributor regions, stride, `#interrupt-cells`; GICv2 → Unsupported | [04 §3](04-code-contracts-facts.md) | P2-V03 (W02-DV06) |
| P2-B05 | Timer: `arm,armv8-timer` presence and raw interrupt-specifier count; others → Unsupported | [04 §3](04-code-contracts-facts.md) | P2-V03 (W02-DV06) |
| P2-B06 | PSCI: compatible version class and `method` recorded; no execution | [04 §3](04-code-contracts-facts.md) | P2-V03 (W02-DV06) |
| P2-B07 | `/chosen`: `stdout-path` (with alias resolution), `bootargs`, initrd boot-artifact range | [04 §3](04-code-contracts-facts.md) | P2-V03 (W02-DV07) |
| P2-B08 | Capability summary preserving states; P2-undiscovered areas are `NotDiscovered` | [04 §2, §5](04-code-contracts-facts.md) | P2-V04 (W02-DV08) |
| P2-C01 | Normalization: one typed result; no reparse by consumers; no board-name choice | [04 §5](04-code-contracts-facts.md) | P2-V04 (W02-DV09) |
| P2-C02 | States distinguishable: absent, unsupported, unusable, usable, not-discovered | [04 §2](04-code-contracts-facts.md) | P2-V04 (W02-DV08) |
| P2-C03 | Deterministic repeated discovery on identical input | [01 §7](01-scope-and-foundations.md), [04 §5](04-code-contracts-facts.md) | P2-V04/P2-V10 (W02-DV10) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no Cargo workspace, no Rust sources, no P1
runtime, and therefore no W01 implementation to consume yet. W02 is designed
against W01's contracts ([../p2-w01-boot-platform-description-intake/README.md](../p2-w01-boot-platform-description-intake/README.md))
as an assumed dependency; if W01's contracts land differently, this design's
adapter surface changes but nothing below normalization does.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| One normalized capability-driven representation (P2-V03/P2-V04) | No discovery code exists | Typed `PlatformInfo`/`PlatformCapabilities` construction over fixed-capacity boot storage | Without a single typed result every consumer would reparse the DTB | W02 (this design); consumed via W10 contract | W02-DV09 review; W02-DV02–DV07 host tests |
| Required facts collected when declared (P2-V03) | Nothing collected | Seven walkers with declared-input coverage | Each required fact group needs exactly one owner | W02 | W02-DV02–DV07 |
| Missing/unsupported/absent/usable states preserved (P2-V04) | No state model exists | Five-state `FactState` with per-fact rules | Collapsing states would hide diagnostics from P3/P4 | W02 | W02-DV08 |
| Core has no board-name decision (P2-V04) | n/a — no code | Capability-driven selection rule and review check | ADR-043/052 are architecture gates | W02 design; enforced at W02-DV09 review | W02-DV09 |
| Deterministic repeated input behavior (plan step 5) | n/a | No-allocation, fixed-order pipeline | Dynamic memory or set iteration would make output order unstable | W02 | W02-DV10 |
| W01 validated input available (plan step 1) | W01 designed, not implemented | Assumed W01 contracts with failure boundary | Discovery must not re-validate or reparse raw bytes | W01 owner; W02 consumer | W01 verification when it lands; until then host fixtures |
| QEMU/RK3566 portability review inputs (plan step 4) | No fixture DTBs tracked | Host fixtures synthesized from public DT shapes; real fixtures are W07 scope | Portability must be reviewable before W09 QEMU evidence | W02 (synthetic), W07 (real fixtures) | W02-DV02–DV08 run on synthetic fixtures |

No ledger row invents a crate, target, or runtime policy. The absent W01
implementation is an ordered prerequisite, not a blocker for designing or
host-testing W02 against synthesized blobs.

## Resolved design decisions and their authority

1. **Discovery runs pre-allocation on fixed-capacity boot storage; the heap
   is never used in W02.** Rationale: boot order is W01 → W02 → W03 → W04 →
   W05, so no allocator exists yet; W05's heap is the permanent dynamic model
   afterwards, and the pre-heap bounded arrays are documented as temporary
   ([01 §3](01-scope-and-foundations.md)); this also delivers determinism for
   free.
2. **Five-state fact model** `NotDiscovered / Absent / Unsupported /
   Unusable / Usable`. Rationale: the plan requires missing, unsupported,
   absent, and usable states to stay distinguishable; `NotDiscovered` is
   added so P2 cannot silently claim knowledge about areas (PCI, SMMU, ACPI)
   it never examined ([04 §2](04-code-contracts-facts.md)).
3. **Cell-width support fixed at 1–2 cells for `#address-cells` and
   `#size-cells`.** Rationale: the DT spec allows 1–4, but the reference
   platform and the RK3566 fixture use ≤ 2; supporting 4 doubles decoder
   surface for no consumer. Widths outside 1–2 are `Unsupported` states,
   not parse errors.
4. **Fatal-versus-recorded rule.** Facts whose absence makes the P2 result
   unusable are fatal: CPU inventory empty, boot-CPU mismatch, no RAM bank.
   All other required-set facts are recorded states and boot continues.
   Rationale: task book P2-W02 scope names "CPU inventory and boot-CPU
   relation" as the discovery outcome's core; without them P3's input
   contract (p3-w01) cannot exist, and without RAM, W03 has no input
   ([01 §6](01-scope-and-foundations.md)).
5. **PSCI policy: `arm,psci-0.2` and `arm,psci-1.0` with a `method` property
   are `Usable{method}`; PSCI 0.1 is `Unsupported` (legacy function-ID
   properties are Reserved); no `/psci` is `Absent`.** Rationale: 0.2+ carry
   standard function IDs, so P3 needs only the method; 0.1 parsing would add
   surface for a legacy path no P2 consumer exercises.
6. **Console fact records `stdout-path` with an explicit resolution
   state; `/aliases` resolution is attempted at most one hop, and an
   unresolvable path leaves `resolved_node` empty while the fact stays
   recorded.** Rationale: P1 already owns the working early console; P2
   records the fact for inspection and W06/W10 handoff, so a fancy
   `stdout-path` must be visible without being able to stop boot or
   masquerade as resolved.
7. **GICv3 required shape: compatible `arm,gic-v3`, distributor +
   redistributor `reg` entries, `#interrupt-cells = 3`; `#redistributor-regions`
   (default 1) and `redistributor-stride` (default 2×64 KiB) recorded; GICv2
   compatibles are `Unsupported{gicv2}`; ITS sub-nodes are ignored
   (`NotDiscovered` sub-fact).** Rationale: ADR-032 makes GICv3 the baseline;
   ITS is P6+ scope; the defaults come from the DT binding and are recorded
   rather than guessed at consumption time.
8. **Unknown nodes/properties outside the required set are skipped with a
   counter; unknown properties inside required nodes are ignored.** Rationale:
   the task book requires tolerating safely ignorable unknowns; W01 already
   guarantees structural safety, so skipping is provably safe; the counter
   keeps the ignorance observable (W06/W09).
9. **Discovery reads only through the W01 cursor; no second validation, no
   raw byte access.** Rationale: single validation authority
   ([../p2-w01-boot-platform-description-intake/README.md](../p2-w01-boot-platform-description-intake/README.md));
   duplicating validation would allow divergence between checker and boot
   paths.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   storage, determinism, and fatal-versus-recorded policies.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module decomposition and fact model before writing any walker.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md);
   decoder/walker contracts are in
   [03-code-contracts-decoders.md](03-code-contracts-decoders.md), fact
   model and normalization contracts in
   [04-code-contracts-facts.md](04-code-contracts-facts.md).
4. Validate per [06-validation-and-handoff.md](06-validation-and-handoff.md).
   Record decisions/deviations in
   `../p2-w02-platform-discovery-normalization-record.md` and evidence in
   `../../verification/p2-w02-platform-discovery-normalization-verification.md`
   when that work starts; neither this design nor a record claims W02
   complete.

## Explicitly excluded interfaces

W02 authorizes no allocator calls, no physical-range claims, no device
register access, no PSCI call, no logging channel of its own beyond the P0
diagnostic channel, no serialization of `PlatformInfo` (a versioned wire
format for machine-readable export, if ever required, is a later design; raw
struct serialization is forbidden by the Coding Guidelines), and no public
API beyond the fact types and normalization entry points in
[03](03-code-contracts-decoders.md)/[04](04-code-contracts-facts.md).

## Downstream handoff

- **W03** ([../p2-w03-boot-memory-map-ownership/README.md](../p2-w03-boot-memory-map-ownership/README.md))
  consumes RAM banks, `/reserved-memory` ranges, and boot-artifact ranges
  (initrd) plus W01's DTB range and rsvmap entries. W02 claims nothing.
- **W06** ([../p2-w06-platform-memory-inspection/README.md](../p2-w06-platform-memory-inspection/README.md))
  reports the actual `PlatformInfo`; W02 supplies the fact types it renders.
- **W07** ([../p2-w07-offline-dtb-compatibility/README.md](../p2-w07-offline-dtb-compatibility/README.md))
  reuses walkers host-side on fixture DTBs; P2-V09 fixture evidence lands
  there.
- **W08/W09** receive determinism and fact-stability expectations as
  regression/integration criteria.
- **P3** consumes the CPU inventory, boot-CPU relation, PSCI, and capability
  facts only as recorded through the W10 handoff contract — never a
  particular parser or type instance; p3-w01 owns the topology-input
  reconciliation.
- **P4** consumes capability and reservation facts likewise (p4-w02/p4-w03
  plans), through W10.
