# P1-W03 AArch64 Capability Inventory — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The capability report required by
[P1-W03](../../plans/p1-w03-aarch64-capability-inventory.md): fact extraction
from AArch64 identification registers, required/optional/future
classification, the fail-fast policy for absent required facts, and the
report interface consumed by W04, W09 and handed to P2.  
**Owner/change context:** P1-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

The [implementation reconciliation](05-implementation-reconciliation.md)
corrects identification-field semantics and records concrete module/test
placement against the delivered W02 runtime. Read it with the contracts.
Implementation and evidence are separate in the
[record](../p1-w03-aarch64-capability-inventory-record.md) and
[verification](../../verification/p1-w03-aarch64-capability-inventory-verification.md).

This is the implementation-level design for P1-W03. W03 is the stage's
knowledge mechanism: it reads the CPU's self-description, classifies every
fact it reports, blocks continuation on absent required facts, and hands a
queryable report to every later mechanism so that no package branches on a
platform name. It deliberately does **not** configure anything it discovers
(that is W04's and later packages' work), implement Stage-2 or GIC or timer
behavior, parse the DTB, or define a platform-capability framework for P2.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the fact
  model (fact set, classification × observation taxonomy), the report
  object's lifecycle and ownership, the concurrency model, and the
  assumed-contract table. Load this first for any step.
- [02-code-contracts-fact-extraction.md](02-code-contracts-fact-extraction.md) —
  contracts for every register read and extraction function, with the
  audited-`unsafe` boundary and per-fact rationale.
- [03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md) —
  contracts for the classification, the required-fact check and its
  fail-fast route, the published report and its query API, and the
  render/emit contract consumed through the W06 channel.
- [04-implementation-and-review.md](04-implementation-and-review.md) — ordered
  workflow, validation matrix, error/security/observability model, and
  handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W03
plan](../../plans/p1-w03-aarch64-capability-inventory.md). This document is
the proposed detailed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W03 plan → this
design → Coding Guidelines. In particular:

- The task book requires that required capability absence fail fast, and that
  supported/optional/future facts remain distinguishable (T03, P1-V05,
  P1-V06). ADR-044 ("capability-driven behavior, never platform names") is
  the architectural reason this package exists.
- The plan scopes Current EL, CPU identity/affinity, architecture version,
  PA/VA and translation limits, Stage-2 capability, granules, timer and
  relevant virtualization extensions, plus absence and failure
  classification — a bounded fact set, explicitly *not* a discovery
  framework: P2 owns DTB-to-PlatformInfo (task book §2 Reserved).
- The [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime and the
  [W09](../p1-w09-initialization-sequencing/README.md) lifecycle are
  accepted sibling contracts consumed directly: the inventory runs inside
  W09's `capabilities` phase, its fail-fast routes via W02's panic route
  (the matrix's `capabilities` row), and its textual output renders through
  W06's channel per W09 marker rule M4. W05–W08 and P2 consume the report
  as an assumed downstream surface.

Classification: the fact set, the classification taxonomy, the extraction
contracts, the required-fact check with its named fail-fast route, the
published report with its query API, and the render contract are
**Required**. Extension of the fact set for later packages (new facts, new
consumers), a P2-shaped `PlatformCapabilities` type, and any reconsideration
of the required set are **Reserved** with recorded triggers. Platform
discovery, DTB parsing, GIC/timer/Stage-2 mechanisms, CPU topology
bring-up, VM capability policy, board-specific behavior, and any allocator
are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inventory the facts required by P1; reserve later facts (work seq 1) | [Architecture](01-architecture-and-state.md) §2–§3 | P1-V05 (W03-DV01) |
| Define required-versus-optional validation and reporting categories (work seq 2) | [Architecture](01-architecture-and-state.md) §4; [classification contracts](03-code-contracts-classification-and-report.md) §1–§2 | P1-V05, P1-V06 (W03-DV02) |
| Integrate the report with boot diagnostics and fail-fast policy (work seq 3) | [Classification contracts](03-code-contracts-classification-and-report.md) §3–§5 | P1-V06 (W03-DV03) |
| Review that consumers query capabilities, not platform names (work seq 4) | [Workflow](04-implementation-and-review.md) step 4 | P1-V05 (W03-DV05) |
| Define normal, missing-required and missing-optional acceptance evidence (work seq 5) | [Workflow](04-implementation-and-review.md) §3 | P1-V05, P1-V06 (W03-DV06) |
| Hand off the capability contract to baseline, lifecycle and P2 planning (work seq 6) | Downstream handoff below; [workflow](04-implementation-and-review.md) §5 | W03 closure review (W03-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no Rust sources, no identification-register access anywhere,
no report type, no consumer. The W02 runtime and W09 tracker this package
runs inside exist as accepted designs in this branch, not as code; W06's
channel and W04's baseline are parallel designs. Every executing prerequisite
is therefore a contract, not a present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Capability report distinguishes required/optional/future facts (P1-V05) | No fact set, no taxonomy, no report | The fact set and two-axis taxonomy of [01-architecture-and-state.md](01-architecture-and-state.md) §2–§4 | A report cannot "distinguish" categories the design never separated | W03 (this design) | W03-DV01/DV02 |
| Report is produced from a live runtime (plan prerequisite: W02) | No runtime exists | The extraction contracts of [02-code-contracts-fact-extraction.md](02-code-contracts-fact-extraction.md), implemented in the capabilities phase | Facts must be read at EL2 from running code; W02 supplies that context | W03 (extraction); W02 (runtime) | W03-DV04; execution via W09 phases |
| Required absence blocks normal startup with a named reason (P1-V06) | No fail-fast mechanism exists | The required-fact check and rejection contract of [03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md) §2–§3 | "Fail-fast" needs a check, a named reason, and an established route (W02's panic route) before it is a behavior | W03 (check); W02 (route); W09 (matrix row) | W03-DV03; NC2 execution via W11 |
| Optional absence remains distinguishable and non-fatal (P1-V06) | Nothing to observe | The observation axis (Present/Absent/Unreadable) and its non-fatal reporting rule | Distinguishability is a property of the data model, checked by review and scenario | W03 | W03-DV03, DV06 |
| Report is renderable through boot diagnostics (work seq 3) | No channel exists | The render/emit contract of [03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md) §5 | W09 M4 routes capability *content* through W06's rendering; the content must be channel-compatible | W03 (content/lines); W06 (channel/format) | W03-DV03 |
| Consumers query capabilities, not platform names (work seq 4) | No consumers exist yet | The query API and the consumer-conduct review | ADR-044 compliance is a property of the consuming code, reviewable only against a stable API | W03 (API); consumers (conduct) | W03-DV05 |
| Capability knowledge handed to P2 (plan handoff) | Nothing to hand over | The published report's survival to `stable` and the handoff wording | P2 consumes knowledge, not a discovery implementation (task book §7) | W03; P2 plans name the consumers | W03-DV07 |

No row requires designing a platform framework or a later-stage mechanism;
the required-set variability question for W11's NC2 is a recorded
coordination item, not a blocker for this design.

## Resolved design decisions and their authority

1. **Two-axis fact model: classification (Required / Optional / Future) is
   orthogonal to observation (Present(value) / Absent / Unreadable).**
   Rationale: the task book's distinguishability requirement (P1-V06) needs
   both "what kind of fact is this" and "what did we observe" independently;
   folding them into one enum (e.g. `RequiredAbsent`) hides the observation
   and makes reporting ambiguous. Authority: task book T03; plan work seq 2.
2. **The Required set is exactly the facts whose absence makes P1
   continuation meaningless: EL2 execution, 4 KiB granule support, and a
   readable non-zero counter frequency.** Rationale: these are the only
   variable, load-bearing preconditions of P1's own mechanisms; identification
   registers that are architecturally always present (MPIDR, the ID_AA64*
   group) are recorded as facts but not dressed up as "required checks" —
   that would fabricate test surface. Later P1 phases (e.g. W08's PA-range
   use) query their own facts from the report and fail through their own
   routes, per W09's H1/H4. Authority: W09 routing matrix; ADR-044; plan
   scope.
3. **Fail-fast names the fact and the reason; it never becomes an unrelated
   panic.** The rejection carries two `&'static str` (fact label, reason) so
   the early panic route can emit it through the pre-console writer, exactly
   what W11's NC2 expects to observe. Authority: task book P1-V06; W09
   matrix `capabilities` row; W11 NC2 pass condition.
4. **The report is written once during the `capabilities` phase into a
   published static, then read-only for the rest of the stage.** One owner
   (the phase), one publication, monotone like the lifecycle itself; the
   audited-`unsafe` boundary is the once-publication cell, justified by the
   single-CPU, interrupts-masked boot context. Authority: W09 single-owner
   discipline; Coding Guidelines unsafe rules; P0-W10 governance.
5. **Extraction is one audited `unsafe` per raw register read, wrapped by
   safe, named extraction functions.** The `unsafe` is the architectural
   register access itself (a `mrs` the compiler cannot type); everything
   above — field decoding, range sanity, classification — is safe code.
   This is the smallest honest unsafe boundary and lands in the P0 unsafe
   inventory. Authority: Coding Guidelines; P0-W10.
6. **The report renders as fixed label-token lines through a caller-supplied
   emit callback; W03 owns content and line format, not the transport.**
   Rationale: W06 owns the channel and marker format (W09 §8); a callback
   seam avoids W03 defining (or depending on) a channel trait owned by
   another design. Lines are bounded and allocation-free. Authority: W09
   marker rules M4; W06 plan scope ("capability output").
7. **The fact set is frozen at the P1 mechanisms' needs plus the explicitly
   named future facts; growing it is a design change, not a local edit.**
   Rationale: a capability inventory that grows by convenience becomes a
   discovery framework — P2's job. The Reserved trigger names who may add
   facts and how. Authority: plan out-of-scope list; task book §2.
8. **NC2 coordination requirement recorded, not engineered around.** The
   [W11 correction](../p1-w11-negative-fault-validation/README.md) of
   2026-09-24 permits an explicitly identified validation-image variant;
   this supersedes the historical environment-only blocking disposition
   below. W03's production required set remains unchanged. W11's
   NC2 needs at least one required fact that the reference platform can
   vary; the required set of decision 2 may not be variable on QEMU `virt`
   CPU models. W03 records this honestly: the required set follows P1's
   real needs, and if no required fact is variable, NC2 is blocked with
   that finding per W11's own rule — fabricating a requirement to make a
   scenario passable is prohibited. Authority: W11 NC2 dependency note;
   task book P1-V06 intent.

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for the
   fact set, taxonomy, lifecycle, and assumed contracts. Every step depends
   on it.
2. Load [02-code-contracts-fact-extraction.md](02-code-contracts-fact-extraction.md)
   when implementing reads and extraction, and
   [03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md)
   when implementing classification, the required check, the published
   report, and rendering.
3. Execute the steps in the order given in
   [04-implementation-and-review.md](04-implementation-and-review.md):
   prerequisite confirmation, extraction, classification and report, route
   integration, consumer-conduct and negative-path reviews, evidence and
   handoff.
4. Record implementation decisions and deviations in
   `../p1-w03-aarch64-capability-inventory-record.md` when implementation
   begins, and validation commands, environments, and outcomes in
   `../../verification/p1-w03-aarch64-capability-inventory-verification.md`
   when evidence exists. Neither file may exist yet, and neither this design
   nor a record may claim W03 complete.

## Explicitly excluded interfaces

No platform discovery, DTB parsing, `PlatformInfo`, `PlatformCapabilities`
framework, GIC/timer/Stage-2 mechanism, CPU-topology or VM capability policy,
or board-name conditional is designed or authorized. The report is a
P1-internal boot-scope structure, not a public ABI, wire format, or
persistent layout. W03 configures nothing: a design or implementation that
writes a control register as part of "inventory" is a scope violation
belonging to W04. No channel abstraction, marker format, or console code is
authorized; rendering emits bounded strings to a caller-supplied sink.

## Downstream handoff

- **[P1-W04](../p1-w04-el2-architectural-state-baseline/README.md)** receives
  the published report and its query API as the only legitimate source of
  the facts its baseline categories map to (its plan work seq 1), including
  the optional-fact guards (e.g. the EL2 virtual timer's presence).
- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** is **not** a
  named W03 consumer in the plan index; it receives translation-limit
  knowledge transitively through W04's baseline record. If its accepted
  design wants direct query access, that dependency is named and recorded
  in its own design (a Reserved trigger here, not a handoff obligation).
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `capabilities` phase body (the extraction → classification → required
  check → publication sequence) and the rejection contract its matrix row
  already names.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  required-fact list and rejection vocabulary for NC2, plus the recorded
  variability finding (decision 8).
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  classification knowledge (required/optional/future) for the limitations
  and handoff documents.
- **P2** receives the retained report contents as context for discovery
  design (per the P2 plan index's named consumers); P2 owns
  `PlatformInfo`/`PlatformCapabilities` and inherits no P1 type.

A coding agent completing W03 must leave the handoff checklist in
[04-implementation-and-review.md](04-implementation-and-review.md) answerable
without inspecting W03 source code.
