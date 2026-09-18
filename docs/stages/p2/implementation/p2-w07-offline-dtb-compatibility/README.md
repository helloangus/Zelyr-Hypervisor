# P2-W07 Offline DTB Compatibility Checking — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The host-side, offline DTB readiness checker — fixture corpus,
readiness-report model, and objective per-fact outcomes for the QEMU `virt`
and Orange Pi 3B/RK3566 fixtures — required by
[P2-W07](../../plans/p2-w07-offline-dtb-compatibility.md).  
**Owner/change context:** P2-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W07. It converts the bounded
work-package plan into a design for one offline mechanism: running the *same*
W01 intake validators and W02 normalization pipeline on host against tracked
fixture DTB images, and mapping their outcomes onto an objective readiness
report with PASS/WARN/FAIL/NOT-P2 classes, binding expectations, and an
explicit no-board-support boundary. It deliberately does **not** implement a
second parser or re-derive discovery semantics (the boot code is the only
semantics), does not define a user-facing checker CLI or command surface
(plan out-of-scope), does not analyze SoC drivers beyond the P2-required fact
set, does not claim Orange Pi 3B EL2 runtime support in any classification,
and does not create QEMU automation (P0-W09 owns the runner; W09 owns
integration runs).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W07 plan](../../plans/p2-w07-offline-dtb-compatibility.md). This
document is a proposed design; it contains no implementation or validation
claim.

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed W01/W02/P0 contracts, the offline input model, single-semantics policy, and tooling placement. |
| [02-readiness-report-model.md](02-readiness-report-model.md) | The report schema, outcome classes, the mechanical mapping from W01 diagnostics and W02 fact states to report rows, verdict rules, and the no-support-claim wording. |
| [03-fixture-and-expectation-matrix.md](03-fixture-and-expectation-matrix.md) | The fixture contract (image + expectation + provenance), the QEMU `virt` and RK3566 fixture definitions, the unrelated-device warning policy, and negative-fixture handoff to W08. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | The ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | The validation matrix (P2-V09), error/security/observability model, and the handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W07 plan](../../plans/p2-w07-offline-dtb-compatibility.md) → this design
→ Coding Guidelines. Binding constraints:

- ADR §11 names a "host-side DTB compatibility checker" as a platform
  bring-up accelerator; the P2 task book bounds it to P2-required facts and
  an Orange Pi 3B/RK3566 *offline fixture only* (task book Reserved list).
  This design never widens the check surface to driver-level analysis.
- Single-semantics rule (continuing W01/W02's single-validation-authority
  decisions): the checker *is* the boot intake + normalization code running
  on host. Any semantic implemented twice would let checker and boot
  disagree — the exact defect class P2-V10's determinism rows guard
  against.
- The task book's P2-V09 wording sets the honesty boundary: fixture
  evidence "does not assert Orange Pi EL2 runtime support". Report classes
  and verdict wording are designed so no output can be quoted as a board
  support claim ([02 §5](02-readiness-report-model.md)).
- Coding Guidelines: offline tooling is still repository Rust; it uses no
  `unsafe` beyond what it inherits from the host-side fabrication boundary
  already designed by W01, adds no dependency without P0-W18 governance,
  and stays outside the EL2 target build.

Classification. **Required:** the offline check entry over DTB byte images,
the readiness-report model with mechanical state mapping, the QEMU `virt`
fixture with binding expectations, the RK3566/Orange Pi 3B fixture with
shared-semantics expectations, the unrelated-device aggregate warning, and
host-regression stability of the whole corpus. **Reserved** (recorded
triggers, no P2 implementation): a user-facing CLI/name for the checker,
driver-level or binding-coverage analysis beyond the P2 fact set, ACPI or
x86_64 fixture classes, additional board fixtures, report diffing across
QEMU releases. **Out of Scope:** runtime boot behavior (W09), Orange Pi EL2
support (P15), SoC/BSP driver work, malformed-input *harnesses* (W08 owns
the negative regression; W07 only fixes the fixture format it reuses),
CI wiring (P0-W20), and any Core code change.

## Requirement-to-design mapping

The tracked sources define P2-I01–I05 at group granularity only; the rows
below are this design's reviewable enumeration from the plan's scope
wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-I01 | Offline input: the checker consumes DTB byte images through the unmodified W01 intake boundary (placement inputs substituted by offline parameters) and runs entirely on host | [01 §3](01-scope-and-foundations.md), [02 §3](02-readiness-report-model.md) | P2-V09 (W07-DV01, DV06) |
| P2-I02 | Required-fact checks: report rows cover CPUs, RAM, reservations, GIC, timer, PSCI, chosen/console at W02's semantic level | [02 §2, §4](02-readiness-report-model.md) | P2-V09 (W07-DV02) |
| P2-I03 | Objective readiness reporting: per-row classes and the overall verdict are mechanically derived from W01/W02 outcomes; tolerated gaps WARN, boot-fatal outcomes FAIL | [02 §3, §5](02-readiness-report-model.md) | P2-V09 (W07-DV03) |
| P2-I04 | QEMU `virt` fixture: reference DTB image with binding expectations derived from the P1 boot path; regression-stable | [03 §3](03-fixture-and-expectation-matrix.md) | P2-V09 (W07-DV01, DV04) |
| P2-I05 | RK3566/Orange Pi 3B fixture: same shared semantics exercise; unsupported unrelated devices produce a WARN aggregate; no runtime-support claim | [03 §4](03-fixture-and-expectation-matrix.md), [02 §5](02-readiness-report-model.md) | P2-V09 (W07-DV02, DV03, DV05) |
| Fixture handoff (plan step 6) | W08 can extend the corpus with mutation fixtures using the same format and harness entry | [03 §6](03-fixture-and-expectation-matrix.md) | W07 closure review (W07-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no W01/W02
implementation, and no tracked DTB fixture of any kind. W07 is designed
against W01's intake contracts and W02's normalization contracts as assumed
prerequisites; both are host-runnable by construction (W01 README Decision 8,
W02 assumed contract A4), which is the property that makes offline checking
possible at all.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Objective readiness reports for both fixtures (P2-V09) | No checker, no fixtures | Report model + `check` entry + fixture corpus with expectation files | Without a mechanical mapping, "readiness" would be prose opinion | W07 (this design), consuming W01/W02 code | W07-DV01–DV04 host runs |
| Checks expose platform-description gaps before EL2 boot (plan goal) | Nothing offline exists | Host invocation of the unmodified boot pipeline over fixture bytes | The same code path is what makes the offline result predictive of boot | W01/W02 own semantics; W07 owns invocation + reporting | W07-DV06 code-identity review |
| QEMU `virt` fixture coverage (P2-I04) | No fixture tracked | QEMU `virt` DTB image + binding expectation file with provenance | The reference platform is P2's validation anchor (ADR-003) | W07; provenance at implementation time | W07-DV01/DV04 |
| RK3566 fixture semantics (P2-I05) | No fixture tracked | RK3566 DTB image + class-level expectations + unrelated-device policy | Task book lists the board as an offline fixture only | W07 | W07-DV02/DV03/DV05 |
| Warning for unsupported unrelated devices without board-support claims (task book §6) | n/a | Aggregate WARN row + fixed report disclaimer ([02 §5](02-readiness-report-model.md)) | Honesty boundary is a report-design property, not a wording afterthought | W07 | W07-DV05 |
| W01/W02 pipeline available (plan step 1) | Designed, not implemented | Assumed contracts with failure boundaries ([01 §2](01-scope-and-foundations.md)) | Task book §2 upstream-defect rule | W01/W02 owners; W07 consumer | Source verification records when they land; synthetic fixtures meanwhile |

No ledger row invents a crate, target, CLI, or CI decision. The absent
upstream implementations are ordered prerequisites; the fixture images are
implementation-time artifacts with provenance, not design-time inventions.

## Resolved design decisions and their authority

1. **The checker has no semantics of its own.** Offline checking = W01
   `intake` over fixture bytes (with offline placement parameters per
   [01 §3](01-scope-and-foundations.md)) + W02 `normalize` + the W07 report
   mapping. Rationale: single-validation-authority is already the
   repositories' design stance in W01 (Decision 9 of W02); duplicating
   semantics for offline use would let fixtures pass while boot fails.
2. **Offline placement inputs are explicit parameters, not a fake P1.**
   Intake's boot-time inputs (A1 pair, A2 window, A3 image range) are
   substituted by offline check parameters: blob length is the file length;
   the offline check runs the placement rules that are meaningful without a
   machine (size sanity, alignment, self-consistency of `total_size`) and
   marks machine-relative rules (window reachability, image overlap) as
   `NOT-P2` informational rows rather than pretending a window exists.
   Rationale: W01's A1–A3 are *boot* contracts; fabricating them offline
   would produce meaningless PASS rows; suppressing the rules silently
   would diverge checker from boot.
3. **Report classes mirror boot outcomes exactly.** FAIL ⇔ a diagnostic that
   is fatal at boot (W01's taxonomy, W02's fatal set); WARN ⇔ a recorded
   non-fatal state (`Unsupported`/`Unusable`/anomalies) — boot continues;
   PASS ⇔ `Usable`; NOT-P2 ⇔ `NotDiscovered` areas and offline-inapplicable
   rules. Rationale: the report's predictive value ("would this boot?") is
   exactly the mapping's faithfulness; P2-V09 demands objective, not
   interpretive, outcomes.
4. **Expectations are data with binding flags, not code.** Each fixture
   carries an expectation file whose rows state the expected class and,
   optionally, expected payload facts (e.g., CPU count); rows are marked
   binding or informational. The checker reports deltas; binding deltas
   fail the fixture test, informational deltas are recorded findings.
   Rationale: QEMU-version and vendor-DTB drift must surface as reviewable
   deltas instead of silently rewriting expectations in code.
5. **Unrelated devices are one aggregate WARN row, never FAIL.** Devices
   outside the P2 required set (PCIe, USB, I2C, … on RK3566) are counted via
   W02's skip counter and W01's anomaly counters. Rationale: the task book
   requires warning for unsupported unrelated devices *without* implying
   board unsuitability — P2 does not need them, so their presence is
   informational noise with a safe default class.
6. **Verdict wording is fixed and support-free.** The overall verdict is
   "P2-discoverable" or "not P2-discoverable", and every report carries a
   fixed disclaimer that readiness says nothing about EL2 runtime, board,
   or BSP support. Rationale: P2-V09's honesty clause is a design property;
   making the disclaimer part of the artifact prevents quoting a PASS as a
   port.
7. **Invocation surface for P2 is the host test/evidence path; no CLI.**
   The checker's callable surface is the library-level `check` entry
   ([02 §3](02-readiness-report-model.md)) exercised by a host test binary
   over the fixture corpus. Rationale: the plan lists a checker CLI as out
   of scope; a CLI (name, arguments, exit codes) is Reserved for a later
   design. P0-W18/P0-W03 placement rules decide where the host target
   lives; no EL2-target code is generated.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   offline input model and placement policy.
2. Read [02-readiness-report-model.md](02-readiness-report-model.md) before
   writing any mapping code — the class table is the contract.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md);
   fixture construction and the expectation matrices are specified in
   [03-fixture-and-expectation-matrix.md](03-fixture-and-expectation-matrix.md).
4. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record decisions/deviations in
   `../p2-w07-offline-dtb-compatibility-record.md` and evidence in
   `../../verification/p2-w07-offline-dtb-compatibility-verification.md`
   when that work starts; nothing here claims W07 complete.

## Explicitly excluded interfaces

No CLI, command name, arguments, or exit-code contract (Reserved); no
second DT parser, validator, or fact walker; no modification of W01/W02
contracts (offline parameters enter through W07-owned adapters around the
published entries, and needed additions are sibling-design conflicts, not
local edits); no network, filesystem discovery, or directory-walking
behavior (inputs are explicit byte images); no report serialization beyond
the human-readable render and the expectation-file schema fixed here
(a machine-readable export would be a later design); no Core/EL2-target
code; no CI wiring; no board-name branches — fixtures are *data*, and no
checker code path names QEMU or RK3566 (the fixture files do, as data).

## Downstream handoff

- **W08** ([../p2-w08-host-robustness-regression/README.md](../p2-w08-host-robustness-regression/README.md))
  receives the fixture format (image + expectation + provenance), the
  `check` entry, and the class mapping as the substrate for its
  malformed-input and determinism regression matrices; W08 owns the
  negative harness and may add mutation fixtures in the same format.
- **W10** ([../p2-w10-p3-p4-handoff-contract/README.md](../p2-w10-p3-p4-handoff-contract/README.md))
  receives the fixture corpus and readiness expectations as a recorded P2
  deliverable for platform planners (P15 direction), with the
  no-board-support boundary recorded verbatim.
- **Platform planners** (task book "Orange Pi 3B/RK3566 as an offline DTB
  compatibility fixture only") receive a compatibility signal — which
  P2-required facts a candidate platform DTB expresses — never an
  implementation or BSP contract.
- **W09** inherits the QEMU `virt` expectation file as the cross-check that
  offline expectations match the DTB QEMU actually supplies at boot
  (comparison recorded, differences are findings, not auto-fail).
