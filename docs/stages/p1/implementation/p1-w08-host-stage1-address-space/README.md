# P1-W08 Host Stage-1 Address Space — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The controlled EL2 Stage-1 address space required by
[P1-W08](../../plans/p1-w08-host-stage1-address-space.md): the runtime
region inventory drawn from W02–W07 seams, the logical mapping classes with
prohibited ambiguous attributes, the static page-table objects, the
controlled MMU-enable transition with continuity and bounded failure
behavior, and the post-MMU environment handed to W09–W12 and P2 — without
making identity mapping a permanent contract.  
**Owner/change context:** P1-W08 implementation handoff.  
**Supersedes:** None. (Its post-MMU `SCTLR_EL2` value supersedes W04's
pre-MMU value through this recorded design, per W04's ownership matrix —
not by local edit.)

## Purpose and use

This is the implementation-level design for P1-W08. W08 gives every byte
the P1 runtime touches an explicit permission, execution, memory-type and
cacheability class, and moves the stage onto mapped memory once, under
assertions, with a defined failure route. It deliberately does **not**
build a dynamic virtual-memory manager, a map/unmap service, a physical
allocator, Guest Stage-2, huge-page support, NUMA awareness, memory
ownership objects, or any future address-space policy.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the region
  inventory, the mapping-class model, the page-table objects and their
  ownership, the transition state machine, the address-arithmetic model,
  the concurrency model, and the assumed-contract table. Load this first
  for any step.
- [02-code-contracts-address-types-and-tables.md](02-code-contracts-address-types-and-tables.md) —
  contracts for the address types and checked arithmetic, the descriptor
  and table types, the table build and verification functions, and the
  audited `unsafe` boundaries.
- [03-code-contracts-mapping-and-transition.md](03-code-contracts-mapping-and-transition.md) —
  the class-to-region assignment table, the prohibited-attribute rules,
  the `enable_host_stage1()` transition contract with pseudocode, the
  post-MMU verification, the failure routing, and the W11 continuation
  point.
- [04-implementation-and-review.md](04-implementation-and-review.md) —
  ordered workflow, validation matrix, error/security/observability model,
  and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W08
plan](../../plans/p1-w08-host-stage1-address-space.md). This document is
the proposed detailed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W08 plan → this
design → Coding Guidelines. In particular:

- The task book requires a "controlled Host Stage-1 address-space
  transition and post-MMU stability" with P1-V13/P1-V14: required
  code/data/stack/vector/MMIO mapping classes have explicit attributes,
  and MMU-enabled execution, console, vectors and fatal diagnostics work
  after the transition without an identity-map semantic promise.
- The plan scopes executable code, read-only data, writable/zero data,
  boot stack, vectors, boot-time data and early-console MMIO; permission,
  execution, memory-type and cacheability classes; and the transition with
  post-MMU behavior — and explicitly excludes the dynamic manager,
  map/unmap service, physical allocator, Guest Stage-2, huge pages, NUMA,
  memory ownership and future address-space policy. Any allocator,
  dynamic mapping API, or Stage-2 register work would violate that
  exclusion; none is designed here.
- The [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime,
  [W04](../p1-w04-el2-architectural-state-baseline/README.md) baseline,
  and [W09](../p1-w09-initialization-sequencing/README.md) lifecycle are
  accepted sibling contracts consumed directly: W08 runs inside W09's
  `stage1` phase, asserts its premises through W04's declaration API, and
  its failures route per the matrix's `stage1` row. The
  [W05](../p1-w05-el2-exception-entry-baseline/README.md) vector region,
  [W06](../p1-w06-early-console-logging/README.md) mapping requirement,
  and [W07](../p1-w07-fatal-crash-diagnostics/README.md) continuity
  obligation are parallel designs consumed through recorded seams.
- The P0 conventions are assumed contracts:
  [P0-W15](../../../p0/plans/p0-w15-address-identifier-type-safety.md)
  (address/identifier type-safety red lines — W08 is the package W02's
  `PhysAddr` contract names as where checked arithmetic enters), and
  [P0-W03](../../../p0/plans/p0-w03-aarch64-build-target-baseline.md)'s
  linker/layout extension points (assumed; failure boundary in the
  ledger).

Classification: the region inventory, the mapping classes, the
prohibited-attribute rules, the static page-table objects, the checked
address arithmetic, the `enable_host_stage1()` transition with its
premise assertions and post-MMU verification, and the failure routing are
**Required**. VA relocation (a linked-at-VA layout), block descriptors,
shared/multi-core shareability policy, a second mapped device window, and
any post-`stable` mapping change are **Reserved** with recorded triggers.
Dynamic mapping, allocators, Guest Stage-2 (`VTTBR_EL2`/`VTCR_EL2` work
beyond W04's zeroing), huge pages, NUMA, memory ownership, and any
identity-map permanence are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inventory runtime regions and prerequisites from W02–W07 (work seq 1) | [Architecture](01-architecture-and-state.md) §2 | W08 closure review (W08-DV01) |
| Define logical mapping classes and prohibited ambiguous attributes (work seq 2) | [Architecture](01-architecture-and-state.md) §3; [mapping](03-code-contracts-mapping-and-transition.md) §1–§2 | P1-V13 (W08-DV02) |
| Define controlled transition, continuity and bounded failure (work seq 3) | [Architecture](01-architecture-and-state.md) §5; [transition](03-code-contracts-mapping-and-transition.md) §3–§5 | P1-V14 (W08-DV03) |
| Integrate vectors, console and crash diagnostics across the transition (work seq 4) | [Transition](03-code-contracts-mapping-and-transition.md) §3, §6; [architecture](01-architecture-and-state.md) §6 | P1-V14 (W08-DV04) |
| Review no-RWX, no-hidden-identity-map and layering constraints (work seq 5) | [Workflow](04-implementation-and-review.md) step 5; [mapping](03-code-contracts-mapping-and-transition.md) §2 | P1-V13, supports P1-V19 (W08-DV05) |
| Define pre/post-MMU acceptance evidence and hand off the environment (work seq 6) | [Workflow](04-implementation-and-review.md) §3; downstream handoff below | P1-V13, P1-V14 (W08-DV06; execution via W10/W11) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no page-table code, no MMU-enable sequence, no address
arithmetic, and no mapping classes exist anywhere in the tree. The region
sources (W02 stack/BSS/static symbols, W05 vector symbols, W06 console
region) exist as sibling designs, not code; no linker script exists (the
P0 target baseline owns the extension points W08 consumes). Every
executing prerequisite is a contract, not a present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Required regions have explicit attributes (P1-V13) | No inventory, no class model | The inventory of [architecture](01-architecture-and-state.md) §2 and the class table of [mapping](03-code-contracts-mapping-and-transition.md) §1 | An attribute that is never stated cannot be explicit; the inventory is the statement | W08 (classes); W02/W05/W06 (regions) | W08-DV01/DV02 |
| Controlled EL2 Stage-1 transition | No MMU code anywhere | `enable_host_stage1()` with premise assertions, barriers, and verification ([transition](03-code-contracts-mapping-and-transition.md) §3) | "Controlled" means asserted premises, ordered steps, and a defined failure route — not an inline `msr` | W08 | W08-DV03 |
| No ambiguous attributes (work seq 2) | Nothing defined | The prohibited-attribute rules of [mapping](03-code-contracts-mapping-and-transition.md) §2 | RWX or accidentally-global mappings would silently defeat the stage's containment posture | W08 | W08-DV02/DV05 |
| Execution, console, vectors, fatal diagnostics continue post-MMU (P1-V14) | No mapping; transports pre-MMU only in effect | The continuity obligations of [architecture](01-architecture-and-state.md) §6 and [transition](03-code-contracts-mapping-and-transition.md) §6 | Each consumer's window must be mapped before `SCTLR_EL2.M` is set, or the transition itself becomes the first crash | W08 (mapping); W06/W07 (windows); W05 (vectors) | W08-DV04; NC5/R1 execution via W11/W10 |
| No identity-map semantic promise (P1-V14) | Nothing to promise | The recorded temporary-assumption decision and its symbol-discipline rule (decision 2 below) | The promise is avoided by contract and discipline, not by choosing a different P1 layout silently | W08; W12 records the limitation | W08-DV05 |
| Checked address arithmetic enters (W02 handoff) | W02's `PhysAddr` defines no arithmetic | The address-type and arithmetic contracts of [address types](02-code-contracts-address-types-and-tables.md) §1–§2 | Table walking and region sizing without checked arithmetic violate the Coding Guidelines on externally influenced addresses | W08 | W08-DV02 |
| Stable environment declared for W09–W12 and P2 (plan handoff) | No declared post-MMU state | The post-MMU environment statement of [transition](03-code-contracts-mapping-and-transition.md) §7 | Consumers need one named, reviewable state to build on | W08; consumers per their plans | W08-DV06 |

No row requires designing an allocator, a discovery mechanism, or Stage-2;
the permanent-identity question is answered by the recorded assumption,
not by new architecture.

## Resolved design decisions and their authority

1. **The P1 mapping is a full 1:1 identity window over exactly the
   inventoried regions, recorded as a temporary assumption.** VA==PA
   makes the transition continuity argument tractable (PC, SP, `VBAR_EL2`
   and every pointer keep their values across `SCTLR_EL2.M=1`), which is
   the bring-up property the plan's "controlled transition" names. The
   *no-promise* half is structural: every access goes through linker
   symbols or the declared region constants — no literal addresses in
   code — and the assumption travels to W12 as a recorded limitation P2
   will supersede through its own design. Authority: plan goal ("without
   making identity mapping a permanent contract"); P1-V14; W12 handoff
   table (host-address-space contract).
2. **4 KiB pages only; block descriptors are not used in P1.** No 2 MiB
   (or larger) aligned region in the inventory can carry a single
   attribute set — code, data, and stack adjoin — so a block would force
   the ambiguous attributes §2 prohibits. `Granule4k` is a W03 Required
   fact precisely because this design leans on it. Authority: W03 fact
   table; stage-local design freedom with recorded rationale.
3. **W08 becomes a named direct consumer of W03's query API** —
   `PaRange` and `Granule4k` — exercising the Reserved trigger W03
   recorded for exactly this case. Rationale: `TCR_EL2` configuration is
   machine-consumed translation-limit knowledge; the transitive path
   through W04's declaration carries controls, not facts. Authority:
   W03 downstream handoff ("if its accepted design wants direct query
   access, that dependency is named and recorded in its own design");
   this is that recording.
4. **The transition is premise-asserted, ordered, single-shot, and
   terminal on failure.** Premises (W04 pre-MMU values via
   `baseline_value`, vectors installed, fatal path armed, facts
   published) are asserted before the first table write; the enable is
   one ordered sequence with architectural barriers; any premise,
   verification, or runtime failure routes to the fatal path with
   `stage1` attribution (W09 matrix row). No rollback exists — a failed
   transition never "partially enables". Authority: W09 T2/T3, H1/H2;
   W04 decision 6 posture.
5. **W08 verifies dynamically what it can and statically what it
   cannot.** Post-enable: `SCTLR_EL2` read-back, a mapped read-only
   read, and a writable write/read. Pre-enable (table walk): every
   mapped entry matches its class; no entry exists outside the
   inventory. Vector and console liveness are *not* dynamically probed
   by W08 — voluntary faulting and MMIO access belong to other owners —
   their post-MMU proof is W10's R1 markers and W11's NC3/NC5, by
   contracted wiring. Authority: ownership discipline (W04 decision 8
   family); W06/W11 seams; stage-local design freedom.
6. **Cache-enable ordering is recorded reasoning, not incidental code.**
   The tables are written while the MMU and caches are off (W04 C8
   verified `M/C/I=0`), so their contents are in RAM and hardware reads
   them without maintenance; enabling `I` requires `ic iallu` + barriers;
   enabling `C` requires no clean/invalidate for these tables under the
   verified baseline. The reasoning is recorded so any baseline change
   re-opens it. Authority: Coding Guidelines (cache/barrier semantics;
   QEMU success is not evidence the reasoning can be skipped).
7. **Checked arithmetic and the `VirtAddr` newtype enter here, extending
   W02's `PhysAddr` exactly as W02's contract anticipates.** All address
   computations (region sizing, table indexing, next-page stepping) use
   the typed arithmetic of [address types](02-code-contracts-address-types-and-tables.md)
   §1–§2; a naked-integer address computation is a review failure.
   Authority: W02 `PhysAddr` contract ("checked arithmetic enters with
   W08's mapping work"); P0-W15 red lines; Coding Guidelines.
8. **The firmware DTB is not mapped.** The plan's "boot-time data" item
   is satisfied by the boot-parameter retention (W02's `BOOT_CONTEXT` in
   the writable class); the DTB itself is retained as an unexamined
   pointer (W01) and P1 performs no access to it — P2 owns its discovery
   and mapping. Mapping unread data would be an unused, unreviewable
   surface. Authority: W01 §4 (DTB treatment); task book §2 (discovery
   is P2 scope).

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for
   the inventory, classes, objects, state machine, and assumed contracts.
   Every implementation step depends on it.
2. Load [02-code-contracts-address-types-and-tables.md](02-code-contracts-address-types-and-tables.md)
   for the types, arithmetic, and build/verify contracts, and
   [03-code-contracts-mapping-and-transition.md](03-code-contracts-mapping-and-transition.md)
   for the class assignments, transition, and handoff environment.
3. Execute the steps in the order given in
   [04-implementation-and-review.md](04-implementation-and-review.md):
   prerequisite and inventory confirmation, types and tables, class
   assignment, transition and verification, constraint reviews, evidence
   and handoff.
4. Record implementation decisions and deviations in
   `../p1-w08-host-stage1-address-space-record.md` when implementation
   begins, and validation commands, environments, and outcomes in
   `../../verification/p1-w08-host-stage1-address-space-verification.md`
   when evidence exists. Neither file may exist yet, and neither this
   design nor a record may claim W08 complete.

## Explicitly excluded interfaces

No map/unmap API, no dynamic region registration, no allocator, no heap,
no physical-memory discovery, no `PlatformInfo`, no Stage-2
(`VTCR_EL2`/`VTTBR_EL2` beyond W04's recorded zeroing), no huge-page or
contiguity hints, no TLB-shootdown machinery (single CPU, monotone
mapping), no memory-ownership objects, and no board-name conditional
 anywhere. No public ABI, wire format, or persistent layout is
introduced; the descriptor encoding is hardware-facing (recorded against
the architecture revision), not a software contract. W08 touches the
console only through W06's declared region constants — never the UART
registers themselves.

## Downstream handoff

- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `stage1` phase body (the mechanism entry its `stage1_step` adapter
  calls), the ordering it already fixes (console and fatal path complete
  before `stage1`), and the failure route its matrix names.
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** and
  **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receive the
  post-MMU continuation point for NC5 (the insertion contract of
  [transition](03-code-contracts-mapping-and-transition.md) §6) and the
  guarantee that post-MMU faults reach the armed fatal path through
  unchanged vectors.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  temporary mapping assumptions (identity window, fixed console window,
  single-CPU shareability), the class table, and the recorded limitations
  for the host-address-space contract document.
- **P2** (named consumers per the W12 handoff table: P2-W03 boot memory
  map, P2-W04 physical page allocation, P2-W05 dynamic small allocation)
  receives a stable Host Stage-1 runtime whose every mapped byte has a
  recorded class, plus the supersession obligation: any relocation,
  new region, or mapping change is P2 design work recorded against this
  package's assumptions — not an extension of this design. The page-table
  *code* is P1-local and sets no precedent for P2's manager (P2 owns its
  own format and policy decisions).
- **[P1-W03](../p1-w03-aarch64-capability-inventory/README.md)** has its
  recorded Reserved trigger exercised by this design's direct query
  dependency (decision 3) — a recorded note for its consumer-conduct
  review, not a change to its contracts.

A coding agent completing W08 must leave the handoff checklist in
[04-implementation-and-review.md](04-implementation-and-review.md)
answerable without inspecting W08 source code.
