# P2-W09 QEMU Platform Integration Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reference-platform integration evidence design for P2 —
configuration matrix, expected observations, repeated-boot stability
scenarios, evidence capture, and pass conditions (P2-K01–K05) — required by
[P2-W09](../../plans/p2-w09-qemu-integration-regression.md).  
**Owner/change context:** P2-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W09. It is a
validation-design package: its deliverables are (a) the required QEMU
`virt` configuration matrix (CPU-count and RAM-size variants), (b) the
expected-observation contract per configuration — discovery facts,
protected-map accounting, allocation totals, and W06 inspection
consistency — (c) the repeated-boot scenarios that detect
uninitialized-state dependence, and (d) the evidence-capture contract and
destinations for P2-V11. It deliberately does **not** execute QEMU (no
result exists until the implementing agent runs the matrix), does not
create or replace the QEMU runner (P0-W09 owns the single entry; P1-W10
owns the boot regression that uses it), does not treat QEMU behavior as
architectural authority or as real-hardware proof, does not start APs or
initialize a GIC or run a guest, and does not claim P2-V01–V10 evidence
("this plan does not claim their execution" — plan acceptance wording).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W09 plan](../../plans/p2-w09-qemu-integration-regression.md). This
document is a proposed design; it contains no implementation or validation
claim.

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed contracts (P0 runner, P1 boot path, W01–W08 outputs), and the QEMU-vs-hardware proof boundary. |
| [02-configuration-matrix.md](02-configuration-matrix.md) | The required configurations, per-configuration expected discovery/map/allocation/inspection observations, and the accounting domain definition. |
| [03-scenarios-and-evidence.md](03-scenarios-and-evidence.md) | Scenario definitions (boot chain, variants, repeated boots, inspection consistency, hard-gate observation), the evidence-capture contract, and failure handling. |
| [04-validation-and-handoff.md](04-validation-and-handoff.md) | The package's own validation matrix (reviewing the integration design and the executed run), the P2-V11 mapping, and the handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W09 plan](../../plans/p2-w09-qemu-integration-regression.md) → this
design → Coding Guidelines. Binding constraints:

- ADR-003 makes QEMU `virt` the reference/CI platform for deterministic
  tests; ADR-049 places QEMU integration in the validation layering. The
  task book exit criterion 5 requires QEMU integration evidence and the
  offline fixture check; W09 supplies the former only.
- The plan's boundary is explicit: QEMU is not an architectural authority,
  and repeated-boot evidence must be separated from hardware-semantic proof
  (plan work sequence 4). Every scenario row carries a proves/does-not-
  prove statement, and no QEMU result validates the omission of hardware
  rules (Coding Guidelines, MMIO/barrier clause).
- The task book's hard gate is observed here at integration strength
  (accounting equation and inspection consistency on live boots); its
  property proof remains the host soak (W08 S308). Neither substitutes for
  the other — recorded in every relevant row.
- P0-W09 owns the one QEMU runner entry and its parameter carrying, serial
  capture, timeout, exit status, and evidence conventions. W09 configures
  and consumes; it never builds a second invocation path (plan work
  sequence 1).

Classification. **Required:** the configuration matrix
([02 §2](02-configuration-matrix.md)), per-configuration expected
observations and the accounting domain, the boot-chain and repeated-boot
scenarios, the inspection-consistency scenario (W06 render + C1–C4 in
every boot), the evidence-capture contract with per-run records, and
run/not-run statuses. **Reserved** (recorded triggers, no P2
implementation): additional machine types or SoC models, KVM/TGL-type
acceleration variance runs, QEMU version matrix expansion beyond the
recorded baseline, migration/snapshot behaviors, SMMU-enabled `virt`
variants (P14 scope), and CI scheduling of the matrix (P0-W20 owns CI;
W09 owns the runnable contract it would schedule). **Out of Scope:** AP
startup, GIC initialization, guest execution (later stages); real-hardware
runs (P15); offline checker fixture evidence (W07); host negative
regression (W08); any completion claim.

## Requirement-to-design mapping

The tracked sources define P2-K01–K05 at group granularity only; the rows
below are this design's reviewable enumeration from the plan's scope
wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-K01 | Standard `virt` configuration boots the full P2 chain (intake → facts → map → allocators → heap → inspection) with expected facts | [02 §3](02-configuration-matrix.md), [03 §2](03-scenarios-and-evidence.md) | P2-V11 (W09-C1 scenario) |
| P2-K02 | CPU-count variants: CPU inventory and topology observations track `-smp` | [02 §2, §3](02-configuration-matrix.md) | P2-V11 (W09-C2) |
| P2-K03 | RAM-size variants: RAM facts and map accounting track `-m`, including a multi-bank configuration | [02 §2, §3](02-configuration-matrix.md) | P2-V11 (W09-C3) |
| P2-K04 | Map accounting within its documented expected domain: equation exact; protected total inside the declared interval | [02 §4](02-configuration-matrix.md) | P2-V11 (W09-C4) |
| P2-K05 | Repeated-boot stability: repeated boots produce identical expectations; no uninitialized-state dependence | [03 §3](03-scenarios-and-evidence.md) | P2-V11 (W09-C5) |
| Evidence/limitations record (plan work sequence 6) | Per-run evidence with run/not-run statuses; limitations recorded for W10 | [03 §5–§6](03-scenarios-and-evidence.md) | W09 closure review; P2-V12 inputs |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no hypervisor
image, no QEMU runner, and no CI. The P1 EL2 runtime and P0 automation that
W09 consumes are plans only. W09 is therefore designed entirely against
assumed contracts with explicit blocked boundaries; this is the same stance
as every P2 design and is restated here because W09 is the package where
those assumptions become runtime-facing.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Automated QEMU evidence across the stated matrix and repeated boots (P2-V11) | No runner, no image, no evidence | Matrix + scenarios + capture contract, executed against the P1 image | Automation must have one owner and a declared parameter surface before runs mean anything | P0-W09 (runner), P1-W10 (boot regression precedent), W09 (matrix/scenarios) | W09 evidence record rows |
| Stable P2 facts per configuration (P2-K01–K03) | No P2 chain exists | Expected-observation tables pinned to configuration parameters | Without pinned expectations, "stable" is unfalsifiable | W09 (this design) | Per-configuration boot logs + inspection renders |
| Map accounting in its documented domain (P2-K04) | No map exists | Accounting-domain definition (equation exact; protected-total interval) | The domain must be declared before results are judged | W09; W03 supplies the equation | Per-boot map sections |
| Repeated-boot evidence detecting uninitialized-state dependence (plan step 4) | Nothing | Repeated-boot scenario with per-boot comparison | Drift across boots is the signature of uninitialized state | W09 | W09-C5 rows |
| W06 inspection consistency active-state check (P2-V11 wording) | W06 designed, not implemented | Inspection render + C1–C4 pass required in every boot log | Integration-strength check of the active-state property | W06 (mechanism), W09 (expectation) | Per-boot inspection sections |
| P0 runner entry with parameter carrying, capture, timeout, evidence conventions (plan step 1) | P0-W09 planned, unimplemented | Assumed contract with blocked boundary | W09 must not recreate automation (plan work sequence 1) | P0-W09 owner | Runner verification when it lands |
| P1 boot path delivering EL2 entry + DTB handoff (W01 A1/A2/A3) | P1 planned, unimplemented | Assumed contracts with blocked boundaries | No boot, no evidence | P1 owners | P1 verification records |

No ledger row invents QEMU behavior as authority or hardware claims; the
multi-bank configuration is pinned from the actual dumped DTB at
implementation time ([02 §2](02-configuration-matrix.md)), not asserted
from memory.

## Resolved design decisions and their authority

1. **One runner, configured only through its declared parameter surface.**
   W09 creates no script, wrapper, or invocation variant outside the
   P0-W09 entry. Rationale: the task book and P0-W09 exist precisely to
   prevent "multiple mutually drifting QEMU commands"; a regression package
   that forks the runner would reintroduce the defect.
2. **The matrix is bounded and pinned: 3 CPU variants, 3 RAM variants,
   baseline repeated.** CPU counts {1, 2, 4}; RAM {baseline per P1 recipe,
   512 MiB, 2 GiB with the high-memory configuration actually producing a
   second bank pinned from the dumped DTB}. Rationale: P2-V11 requires
   CPU-count/RAM-size variation; 8 CPUs is P3's stress count (p3-w13) and
   adds nothing to P2's facts; the multi-bank cell exercises multi-region
   map support (P2-E04) at integration strength. Cells are pinned from
   observed dumps, never from memory of QEMU internals.
3. **Expectations are data files bound to configurations, mirroring W07's
   expectation pattern.** Each configuration has an expectation file
   (fact classes + payload marks + accounting domain) with provenance
   (first-run dump). Rationale: one pattern across W07/W08/W09 keeps the
   drift policy identical; deltas are findings, binding deltas fail.
4. **Repeated boots: baseline ×5, variants ×2, all compared for identical
   expected rows.** Rationale: repeated-boot stability (P2-K05) needs
   enough repetitions to catch uninitialized-state dependence (metadata
   placement, ordering) without pretending statistical power; ×2 on
   variants still catches boot-to-boot drift there.
5. **Inspection is the observation instrument.** Expected observations are
   read from the W06 inspection render in the boot log (platform, map,
   allocator, heap sections) — not from ad-hoc debug prints. Rationale:
   reuses the active-state projection (P2-V11 wording: "active-state
   inspection consistency") and proves the render path on target as a
   side effect; W06's host evidence remains separate.
6. **Accounting domain: equation exact, protected total bounded.** The map
   equation must hold exactly; the protected total must fall inside a
   per-configuration declared interval `[min, max]` derived from the P1
   image size bound + DTB size + known reservation magnitudes (initial
   interval from the first dump run, then pinned). Rationale: the equation
   is exact by construction (W03), so only the *domain* of protected
   totals is an empirical expectation; pinning it makes regressions
   (metadata growth, new reservations) visible.
7. **QEMU results are integration evidence, never semantics authority.**
   Every scenario row's proof boundary states that success does not prove
   hardware behavior (cache/TLB/DMA/timing), does not validate omitted
   hardware rules, and does not extend to Orange Pi or any real board.
   Rationale: plan work sequence 4 and the Coding Guidelines' MMIO clause;
   P15 exists because this boundary is real.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   assumed contracts and proof boundary.
2. Read [02-configuration-matrix.md](02-configuration-matrix.md) and pin
   the configuration cells from actual dumps per its §2 procedure.
3. Execute per [03-scenarios-and-evidence.md](03-scenarios-and-evidence.md)
   in its order; record per
   [03 §5–§6](03-scenarios-and-evidence.md).
4. Validate the package per
   [04-validation-and-handoff.md](04-validation-and-handoff.md); record
   decisions/deviations in
   `../p2-w09-qemu-integration-regression-record.md` and evidence in
   `../../verification/p2-w09-qemu-integration-regression-verification.md`
   when that work starts; nothing here claims W09 complete or contains a
   result.

## Explicitly excluded interfaces

No runner, script, Makefile target, or CI workflow (P0-W09/P0-W20 own
those surfaces); no modification of the hypervisor image or boot code
beyond what W01–W06 already designed (an integration failure is evidence
against the owning package, never a reason for W09-local product patches);
no guest, AP, GIC-init, or SMMU scenarios; no real-hardware claims; no
expectation rewritten to make a run pass (revision rules of
[03 §6](03-scenarios-and-evidence.md) apply). W09's published surface is
the configuration matrix, expectation files, scenario IDs
(`W09-C<nn>`), and the evidence-row schema.

## Downstream handoff

- **W10** ([../p2-w10-p3-p4-handoff-contract/README.md](../p2-w10-p3-p4-handoff-contract/README.md))
  receives the evidence location, the run/not-run distinction, the
  configuration matrix, and the QEMU-vs-hardware boundary verbatim for the
  stage-gate map and known limitations.
- **P3** (via W10; p3-w13 QEMU SMP regression, p3-w02 secondary bring-up)
  receives the reference-platform input baseline: runner usage pattern,
  configuration/expectation file pattern, and the observed `virt` facts —
  explicitly not a proof of SMP or real-hardware behavior.
- **P4** (via W10; p4-w08 QEMU integration regression) receives the same
  baseline for its own matrix design, plus the observed host-RAM/allocator
  totals that bound Stage-2 memory planning inputs.
- **W07** cross-check: the QEMU `virt` boot-time DTB (dumped during W09
  runs) is compared against W07's fixture expectation; differences are
  recorded findings, not auto-fail ([../p2-w07-offline-dtb-compatibility/README.md](../p2-w07-offline-dtb-compatibility/README.md)).
