# P8-W19 Validation Guest Dual-Track Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The retained mechanism-suite regression and its Linux-track
relationship — scenario inventory, track mapping, coexistence and
fixture-maintenance rules — required by
[P8-W19](../../plans/p8-w19-validation-guest-dual-track.md).  
**Owner/change context:** P8-W19 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W19. P8 introduces Linux as a
Guest; the plan's purpose is to guarantee that Linux's arrival **does not
displace** the Rust Validation Guest as the mechanism-level test asset, and
to define precisely which check proves what: the Validation Guest track
proves EL2 mechanisms in isolation (ADR-008/ADR-009 ordering), the Linux
track proves OS integration, and neither substitutes for the other. This
design therefore provides: the retained scenario inventory inherited from the
P4–P7 suites with their owned observables
([01](01-dual-track-scenario-matrix.md)), and the coexistence,
fixture-maintenance, lost-coverage, and workflow rules
([02](02-coexistence-and-workflow.md)). It is a validation design: it defines
the regression split and its evidence route; it contains no results and no
claim that any suite currently passes.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
[01](01-dual-track-scenario-matrix.md) for the inventory and track mapping
and [02](02-coexistence-and-workflow.md) for coexistence and execution rules.
Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P8 task
book](../../task-book-v0.1.md), and the P8-W19 plan). This document claims no
test has run.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W19 plan → this
design → Coding Guidelines. In particular:

- ADR-008 ("验证 Guest 先于 Linux") and ADR-009 (the first EL1 Guest is the
  Rust bare-metal Validation Guest, used to verify EL2 mechanisms
  independently) are the architectural reason this package exists: Linux
  integration success must never become the only mechanism evidence.
- The task book binds this package to P8-V25: a dual-track regression plan
  showing the Validation Guest remains the mechanism suite for HVC, Stage-2,
  timer, SGI/vIRQ, MMIO, SMP, and scheduler interaction alongside Linux. It
  is a *plan* row — passing it means the split and route are complete, never
  that a suite ran.
- The plan's exclusions are binding: **no replacing the Validation Guest with
  Linux, no extending its implementation, no redefining P4–P7 semantics, and
  no treating Linux success as mechanism proof.** W19 is a retention,
  mapping, and maintenance package; the moment it starts designing VG
  features it has left its scope.

Classification. **Required** for W19 closure: the retained-scenario
inventory with owned observables and source authority, the track-separation
mapping, the coexistence and fixture-maintenance rules, the lost-coverage
block rule, and the evidence route through the W16 envelope. **Reserved**
with recorded triggers: new VG scenarios for P8-era mechanisms (trigger: an
approved future design owning them — W19 itself may not extend the VG),
dual-track execution on hardware (trigger: P15), and automated dual-track
scheduling in CI (trigger: the P0 CI baseline package). **Out of Scope:**
Validation Guest implementation changes (P4–P7 ownership), Linux regression
content (W09–W12 ownership), performance measurement (W17), isolation
scenario content (W18), machine-ABI definition (W02/W14), and fixture
recipes (W15).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W01, W09–W10, W15–W16, and existing Validation Guest evidence (work seq 1) | README ledger; [workflow](02-coexistence-and-workflow.md) §1 | prerequisite-contract review (W19-DV01) |
| Inventory the retained mechanism scenarios and their expected observables (work seq 2) | [matrix](01-dual-track-scenario-matrix.md) §3 | P8-V25 (W19-DV02) |
| Map Linux integration checks separately from precise Validation Guest checks (work seq 3) | [matrix](01-dual-track-scenario-matrix.md) §4 | P8-V25 (W19-DV03) |
| Define regression coexistence and fixture-maintenance expectations (work seq 4) | [workflow](02-coexistence-and-workflow.md) §2–§3 | policy review (W19-DV04) |
| Review any lost mechanism coverage as a dependency block (work seq 5) | [workflow](02-coexistence-and-workflow.md) §4 | W19 closure review (W19-DV05) |
| Hand the maintained regression split and evidence route to W20 and P9+ | README handoff; [workflow](02-coexistence-and-workflow.md) §8 | consumability review (W19-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation-only P0 scaffold. The Validation Guest asset itself
(`guests/validation-aarch64/`) is an empty `.gitkeep` marker; P4-W05
(VG-001–VG-012 scenarios), P5-W07 (ABI-security suite with two contexts),
P6-W11 (VG-TIMER/VG-IRQ suites), and P7-W10 (scheduler workload suite) are
planned packages with no implementation or verification records; P8-W01's
reconciliation, W15's fixture, and W16's envelope are P8-planned contracts.
Every input below is an assumed contract with a stated failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P8-V25: Validation Guest remains the mechanism suite for HVC, Stage-2, timer, SGI/vIRQ, MMIO, SMP, scheduler interaction | No consolidated inventory exists across P4–P7 suites | Retained-scenario inventory mapping each suite's scenarios to mechanisms, observables, and source authority ([01](01-dual-track-scenario-matrix.md) §3) | Retention is only provable if what must be retained is enumerated with its owner | Source owners: P4-W05, P5-W07, P6-W11, P7-W10 designs and records; inventory: W19 (this design) | W19-DV02 inventory review; scenario evidence inherited from owners |
| Retained scenarios remain executable in P8 | P8 machine work may change Host-side facts a scenario relied on | Per-scenario applicability rule and lost-coverage block rule ([02](02-coexistence-and-workflow.md) §4) | A retained suite that silently stops running is retention in name only | W19 (rule); owners (scenario fixes) | W19-DV05 review; block records when triggered |
| Linux supplies OS-integration coverage *separately* | W09–W12 integration checks planned only | Track-separation mapping ([01](01-dual-track-scenario-matrix.md) §4) | Without an explicit split, "Linux passed" can silently stand in for mechanism proof — the exact failure ADR-008 forbids | Mapping: W19; Linux check content: W09–W12 | W19-DV03 mapping review |
| Dual-track regression runs as one system | W16 envelope planned only | Coexistence rules binding both tracks into the W16 envelope ([02](02-coexistence-and-workflow.md) §2) | Two uncoordinated execution paths would recreate the divergence the P0 runner governance forbids | W16 envelope; W19 coexistence rules | W19-DV04 review |
| VG fixture maintenance continues | VG asset is P4-owned; W15 owns only the Linux fixture | Fixture-maintenance responsibility statement ([02](02-coexistence-and-workflow.md) §3) | An unmaintained VG asset rots while Linux fixtures evolve | VG asset: P4-W05 maintenance boundary (assumed contract); W19: regression-perspective requirements | W19-DV04 review; **failure boundary:** if P4 delivers no maintenance boundary, W19 records the block against P4 rather than adopting the asset |
| Evidence route | W16 envelope planned only | Evidence destinations via the W16 run record and W19 verification record ([02](02-coexistence-and-workflow.md) §5) | VG evidence must remain first-class P8 evidence, reachable from the W20 index | W16 conventions; W19 | W19-DV06 review |

No row above extends the Validation Guest or redefines any P4–P7 semantic.
The inventory in [01](01-dual-track-scenario-matrix.md) is written so that
each row cites its source suite by plan path; where a source suite is not yet
evidenced, the row is an assumed-contract placeholder and the lost-coverage
rule applies.

## Resolved design decisions and their authority

1. **Two named tracks.** Track VG (mechanism: Validation Guest suites) and
   Track LG (integration: Linux scenarios). The names are stage-local
   design freedom owned here, chosen to match the W16 matrix families so one
   vocabulary spans both designs. No Guest-facing semantics are attached to
   the names.
2. **Retention means executability, not listing.** A scenario counts as
   retained only if it remains executable under the P8 machine contract
   through the W16 envelope, with its source-owned observables unchanged.
   Rationale: the plan's purpose is anti-displacement; a dead inventory would
   satisfy the letter and defeat the purpose.
3. **Observables are inherited, never restated.** [01](01-dual-track-scenario-matrix.md)
   cites each suite's expected observable by source; W19 does not re-declare
   marker text or semantics, so P4–P7 ownership stays intact and a source
   change propagates rather than forks. Rationale: the plan forbids
   redefining P4–P7 semantics.
4. **Lost coverage is always a block.** Any scenario that cannot run in P8
   (machine-contract change, fixture gap, upstream regression) is recorded as
   a dependency block naming the owner — removal or weakening is prohibited.
   Rationale: the plan's work sequence makes lost coverage a block, not a
   tradeoff.
5. **No P8-era VG extension.** If P8 integration exposes a mechanism gap the
   VG cannot observe, the gap is recorded and routed to an authorized design;
   W19 never designs the new VG feature. Rationale: explicit plan exclusion.
6. **Evidence route through the W16 envelope.** VG rows execute as the W16
   `VG-RET` family; evidence lands in the W16 run records plus the W19
   verification record
   (`docs/stages/p8/verification/p8-w19-validation-guest-dual-track-verification.md`),
   so W20's evidence index reaches mechanism evidence without a separate
   toolchain. Stage-local convention owned here.

## Work breakdown and loading order

1. Read [01](01-dual-track-scenario-matrix.md): suite inventory (§3), and
   the mechanism-by-mechanism track mapping (§4).
2. Read [02](02-coexistence-and-workflow.md): preconditions, coexistence
   rules, fixture maintenance, lost-coverage rule, workflow, validation
   matrix, and handoff checklist.
3. Execute the workflow of [02](02-coexistence-and-workflow.md) §5 (steps 1–5) when
   implementation is authorized. Run/not-run status goes to
   `../../verification/p8-w19-validation-guest-dual-track-verification.md`;
   factual decisions go to
   `../p8-w19-validation-guest-dual-track-record.md` — both created only
   when that work begins. Nothing here claims a suite has run.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
or persistent layout is designed or authorized by W19 — including any change
to the Validation Guest's own interfaces, which remain owned by their P4–P7
designs. No new scenario, marker text, fixture recipe, or execution script is
designed here. The only W19-owned artifacts are the inventory, the track
mapping, and the coexistence/maintenance/block rules. Adding anything else
under W19 authority is a scope conflict to stop at review.

## Downstream handoff

Per the [plan index](../../plans/README.md) consumer map:

- **W16** is a prerequisite consumer in the other direction: its `VG-RET`
  family declares W19 the content owner; W19 hands it the retained entry
  set. Any W19 inventory change is a reviewed edit coordinated with
  [W16](../p8-w16-automated-linux-regression/README.md) row references.
- **W18** consumes the retained P5 dual-context suite rows as the
  mechanism-level authority-isolation evidence cited by its other-VM
  boundary ([W18 §4.2](../p8-w18-security-isolation-regression/01-isolation-scenario-matrix.md)).
- **W20** consumes the dual-track split, the lost-coverage register, and the
  evidence route for the P8 evidence index and closure review; W20 must
  present mechanism and integration evidence as separate columns, never
  merged.
- **P9+** receives the maintained split as a standing constraint: virtio-era
  mechanism work must keep both tracks green and must not let Linux
  integration replace mechanism suites. P9 receives no VG implementation
  authority from W19.
