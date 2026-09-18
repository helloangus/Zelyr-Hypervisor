# P4-W07 Repeatability and Stage-2 Telemetry — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The P4 repeat driver (create → run → stop → reinitialize → run
again; cold-boot consistency; no residue dependence), the required event
categories and counters, and fault correlation, as required by
[P4-W07](../../plans/p4-w07-repeatability-telemetry.md) (P4-H01–H03,
P4-I01–I02).  
**Owner/change context:** P4-W07 implementation handoff; this design owns the
episode model and its sequencing authority, the determinism-check surfaces,
the run-record format handed to automation, and the P4 telemetry ledger.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W07. It converts the bounded
work-package plan into the repeat-driver object model and lifecycle, the
determinism/equality contracts, and the telemetry ledger and run-record
contracts, with pseudocode rather than production code. It deliberately does
**not** change the construction/teardown sequencing it coordinates
([P4-W03](../p4-w03-guest-memory-image/README.md) §4 and
[P4-W04](../p4-w04-vcpu-entry-exit/04-code-contracts-vcpu-run.md) §4 stay the
sequencing sources), the fault diagnostics it counts
([P4-W06](../p4-w06-fault-isolation-diagnostics/README.md) stays the
diagnosis authority), the scenario bodies and markers it observes
([P4-W05](../p4-w05-validation-guest/README.md)), the automation harness that
consumes its run record ([P4-W08](../p4-w08-qemu-integration-regression/README.md)),
a production metrics backend or telemetry transport, scheduler statistics,
cross-pCPU Stage-2 shootdown proof, fault-tolerant VM orchestration, or final
VM create/destroy lifecycle semantics.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — scope
  classification, assumed upstream contracts with failure boundaries,
  authority analysis, and the resolved design decisions (D1–D9). Load first.
- [02-architecture-and-state.md](02-architecture-and-state.md) — logical
  modules, the episode and driver state machines, ownership, the concurrency
  model, and the determinism-surface definition. Load for architecture work.
- [03-code-contracts-repeat-driver.md](03-code-contracts-repeat-driver.md) —
  the driver, episode plan, teardown-depth, and determinism-check contracts
  with pseudocode. Load for the repeat-driver work area.
- [04-code-contracts-telemetry.md](04-code-contracts-telemetry.md) — the
  telemetry ledger, counters, fault-correlation records, and the run-record
  grammar (P4-RR v1) with pseudocode. Load for the telemetry work area.
- [05-implementation-workflow.md](05-implementation-workflow.md) — ordered
  implementation steps with acceptance and failure handling.
- [06-validation-and-handoff.md](06-validation-and-handoff.md) — the
  validation matrix (P4-V10, P4-V11, P4-V12 inputs), the
  error/security/observability model, and the handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W07 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W07 plan](../../plans/p4-w07-repeatability-telemetry.md) → this design →
Coding Guidelines. Binding constraints include:

- ADR-048: structured tracing/metrics is a first-class capability from early
  stages; events route through the P0 logging/trace baseline and namespace
  (W01 row R20, assumption A8), are compile-time trimmable, and never become
  ad-hoc prints.
- ADR-003 and task book §8: QEMU is the reference validation environment;
  cold-boot consistency is declared-environment consistency, never a
  hardware-semantics claim; QEMU-observed variance is a Specification
  Investigation item.
- W03 §4 fixes the construction order and states the repeat support duty
  ("fresh allocation and in-place re-init" both supportable); W07 selects the
  repeat forms and owns the driver that sequences them.
- W04 §4 names "the setup/teardown coordinator (W07 repeat driver)" as the
  executor of the stop/teardown sequencing; W07 adds no new teardown step and
  reorders nothing.
- W06 §5–§7 fixes the containment-first ordering and the `diag.*` event set;
  W07 counts events, it does not reclassify or re-render diagnoses.
- Task book §1 Out of scope for W07: final telemetry API, production metrics
  backend, scheduler statistics, cross-pCPU shootdown proof, fault-tolerant
  orchestration, final VM create/destroy lifecycle semantics.

Classification: the episode/driver model, determinism checks, telemetry
ledger, and run-record contracts ([02](02-architecture-and-state.md),
[03](03-code-contracts-repeat-driver.md), [04](04-code-contracts-telemetry.md))
are **Required** for P4-H01–H03 and P4-I01–I02. A persistent event log or
transport, host-side test orchestration beyond the QEMU environment
(W08's), counters beyond the P4 event set, multi-VM or multi-vCPU episode
interleaving, and cross-pCPU consistency proof are **Reserved** with recorded
re-entry points. A production metrics backend, a telemetry wire API, scheduler
statistics (P7), lifecycle semantics beyond the P4 episode (P7+), and any
hardware-behavior claim from repeat evidence are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-H01 same-session create → run → stop → reinitialize → run without residue | [repeat-driver contracts](03-code-contracts-repeat-driver.md) §3–§4; [architecture](02-architecture-and-state.md) §3–§4 | P4-V10 evidence (DV01–DV03) |
| P4-H02 declared cold-boot consistency | [foundations](01-scope-and-foundations.md) D3, D7; [telemetry contracts](04-code-contracts-telemetry.md) §4; [validation](06-validation-and-handoff.md) DV04 | P4-V11 evidence (with W08 execution) |
| P4-H03 cleanup/initialization determinism | [architecture](02-architecture-and-state.md) §5; [repeat-driver contracts](03-code-contracts-repeat-driver.md) §5 | P4-V10/V11 determinism rows |
| P4-I01 required event categories and counters | [telemetry contracts](04-code-contracts-telemetry.md) §2 | P4-V12 evidence |
| P4-I02 VM/vCPU/PC/IPA fault correlation | [telemetry contracts](04-code-contracts-telemetry.md) §3 | P4-V12 evidence |
| Repeat/event expectations handed to W08 | [telemetry contracts](04-code-contracts-telemetry.md) §4–§5 | W08 manifest consumes without reinvention |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs`):
documentation-only repository — no runtime, no allocator, no run path, no
telemetry code. The sibling designs W02–W06 exist as proposed designs with no
implementation records; P0–P3 packages are planned and undelivered except
P0-W01. The fine-grained P4-H/I requirement wording of the superseded root
source task book is not tracked (W01 item A9 provenance pattern); requirement
meaning here comes from the tracked task book §5 rows and the W07 plan scope
sentences. Every foundation below is an assumed contract or a P4-internal
deliverable.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Same-session restart does not rely on prior Guest RAM or global CPU residue (P4-V10) | Nothing exists; W03/W04 define per-episode sequencing only as construction-order facts | The episode model with two teardown depths and per-episode object construction ([03](03-code-contracts-repeat-driver.md) §3–§4) | "no residue" must be exercised by construction, and proven by accounting + equality checks, not assumed | W07 driver; W03/W04 sequencing (assumed contracts M2/M3) | DV01–DV03; P4-V10 |
| Cleanup/initialization deterministic (P4-H03) | W03's deterministic-init contract is a proposal; nothing measures it | Declared determinism surfaces + equality checks incl. the local digest ([03 §5](03-code-contracts-repeat-driver.md)) | determinism is a measured equality property across episodes, not an intention | W07 checks; W03 init contract (M2) | DV03/DV05 |
| Repeated cold boots yield consistent entry, Stage-2, and fault-syndrome results (P4-V11) | No cold-boot repetition exists anywhere; P1-W10 defines the boot-level precedent only | Declared repeat minimums and stability criteria ([01 D7](01-scope-and-foundations.md)); run-record stability fields ([04 §4](04-code-contracts-telemetry.md)) | consistency needs a defined comparison set and a declared environment | W07 expectations; W08 executes; P1-W10/P0-W09 environment (M5) | DV04; P4-V11 via W08 |
| Required event categories observable: VM creation/memory assignment, Stage-2 mutation, vCPU enter/exit/stop, faults (P4-V12) | W02/W03/W04/W06 emit only as proposed designs; no counter exists | The telemetry ledger over the routed event set with per-domain counters ([04 §2](04-code-contracts-telemetry.md)) | observability requires an aggregation point; individual events alone do not give counts | W07 ledger; sibling event sets (M2–M4) | DV06 |
| Counts and fault correlation available (P4-I02) | No correlation record exists | `FaultCorrelation` records and per-episode count vectors ([04 §3](04-code-contracts-telemetry.md)) | correlation is a keyed aggregation the run must expose, not a human reading of raw logs | W07; W06 diagnostic fields (M4) | DV07 |
| W08 can consume repeat/event expectations | W08 has no declared input yet | The versioned run-record grammar P4-RR v1 and minimum-repeat constants ([04 §4–§5](04-code-contracts-telemetry.md)) | automation needs a stable machine-readable surface agreed before both are implemented | W07 grammar; W08 consumption | grammar review; W08 manifest rows |

No row above requires a decision outside this design's authority; repeat
forms, determinism surfaces, repeat minimums, and the run-record format are
stage-local design freedom inside the plan's declared scope ("without
prescribing its internals" refers to telemetry internals, which this design
does fix as P4-local, temporary, unfrozen shapes).

## Resolved design decisions and their authority

Summarized here; full rationale and authority citations in
[01 §4](01-scope-and-foundations.md):

1. **Episode model:** one Guest episode = construct → run → stop → teardown;
   the driver is the setup/teardown coordinator W04/W03 name; episodes are
   strictly sequential; exactly one live episode exists.
2. **Two teardown depths:** `FullRebuild` (everything rebuilt, required) and
   `RamReuse` (address space rebuilt, `GuestRam` retained and re-initialized
   in place, supported per W03's re-init duty); both evidenced (P4-H03).
3. **Determinism is measured, not asserted:** five declared surfaces
   (post-load RAM digest, initial vCPU context, marker stream, exit-class/
   stop-cause sequence, count vector) compared for exact equality across
   episodes and cold boots.
4. **Local FNV-1a-64 digest, no new dependency:** the digest detects
   inequality; it is not an integrity or cryptographic primitive.
5. **Telemetry is an in-memory ledger + run-record emission:** no transport,
   no backend, no persistence beyond the run record; trimmed per P0 rules.
6. **Run-record grammar P4-RR v1:** a versioned, line-oriented,
   machine-readable record emitted through the logging baseline at episode
   boundaries and run end; it is the W08 consumption surface and a recorded
   P4 fact, never a frozen API.
7. **Declared repeat minimums:** same-session ≥ 3 episodes including ≥ 1
   `RamReuse`; cold-boot ≥ 5 iterations; stability = identical determinism
   surfaces + determinate verdicts; the W08 manifest may raise but not lower
   them.
8. **Counters are derived from routed events only** (W02 `s2.*`, W03 `gm.*`,
   W04 `vcpu.*`, W06 `diag.*`); W07 invents no second emission point.
9. **W07 adds no `unsafe`:** the driver and ledger are safe Rust in setup
   context; any discovered need is a recorded design deviation.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): the
   Required/Reserved/Out-of-Scope detail, assumed contracts M1–M6 with
   failure boundaries, contested-area analysis, decisions D1–D9, open items.
2. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module map, the driver/episode state machines, ownership, concurrency,
   and the determinism-surface definition the contracts implement.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md),
   loading [03-code-contracts-repeat-driver.md](03-code-contracts-repeat-driver.md)
   for the driver/episode/determinism work and
   [04-code-contracts-telemetry.md](04-code-contracts-telemetry.md) for the
   ledger and run record.
4. Record validation in
   `../../verification/p4-w07-repeatability-telemetry-verification.md` and
   implementation facts in `../p4-w07-repeatability-telemetry-record.md` only
   when work starts; neither this design nor the records may claim W07
   complete.

## Explicitly excluded interfaces

Not designed or authorized by W07: a telemetry transport, wire format, or
metrics backend (P0-W12/W13 own the baseline; W09 records P4 facts); the
final VM/vCPU lifecycle state machines (P7+ supersede the P4 subsets); any
change to W03's construction order, W04's stop/teardown sequencing, or W06's
diagnosis and verdict semantics (W07 sequences and counts, never redefines);
scenario bodies or marker grammar (W05); the automation harness and its
verdict rules (W08); scheduler-visible accounting or steal-time statistics
(P7); cross-pCPU shootdown proof (Reserved seam in W02); and any
hardware-behavior claim derived from repeat evidence.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W08** (design: `../p4-w08-qemu-integration-regression/README.md`)
  receives the run-record grammar P4-RR v1 with its field list and version
  rules, the declared repeat minimums, the determinism-surface list, and the
  per-scenario expected count patterns it matches.
- **P4-W09** (design: `../p4-w09-closeout-p5-handoff/README.md`) receives
  only implemented facts: the repeat forms delivered, determinism results,
  the event/counter set, and the recorded limitations (no transport, no
  backend, declared-environment scope).
- **P5** inherits the factual observability baseline (event categories,
  counters, run record) as evidence; it owns all new telemetry semantics and
  may not treat P4-RR v1 or the P4 counter set as frozen APIs
  ([P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md)
  is the expected future consumer of the baseline's conventions).
