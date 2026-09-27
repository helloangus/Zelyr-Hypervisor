# P2-W01 Boot Platform-Description Intake — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The validated, diagnosable intake boundary for the boot-supplied
AArch64 device tree blob (DTB) required by
[P2-W01](../../plans/p2-w01-boot-platform-description-intake.md).  
**Owner/change context:** P2-W01 implementation handoff.  
**Supersedes:** None.

Read the [current-baseline amendment](../p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)
before the original assumed contracts or pseudocode below. It reconciles the
completed P1 handoff, source placement and DT specification corrections.
Actual implementation and evidence are linked from the [implementation index](../README.md).

## Purpose and use

This is the implementation-level design for P2-W01. It converts the bounded
work-package plan into a code-bearing design for exactly one mechanism: the
boundary that turns the raw boot-supplied DTB location into a validated,
read-only, structurally bounded description handle that every later P2 consumer
(W02 discovery, W03 map construction, W07 offline checking, W08/W09
regressions) must go through. It deliberately does **not** interpret platform
semantics (CPU, RAM, GIC, PSCI facts are W02), does not construct a memory map
or claim any physical range (W03), does not select an external parser crate or
freeze a permanent public API beyond what W02/W07 need, and does not implement
any host test harness (W08 consumes this design's contracts).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W01 plan](../../plans/p2-w01-boot-platform-description-intake.md). This
document is a proposed design; it contains no implementation or validation
claim.

| Supporting file | Load it for |
|---|---|
| [01-intake-boundary.md](01-intake-boundary.md) | The validated-input outcome: assumed P0/P1 prerequisite contracts, the untrusted-input model, the diagnostic taxonomy, DT encoding and version policy, placement and overlap rules, and DTB lifetime. |
| [02-architecture-and-state.md](02-architecture-and-state.md) | Logical modules, object ownership, lifecycle/state model, failure model, and the single-core boot-time concurrency context. |
| [03-code-contracts-intake.md](03-code-contracts-intake.md) | Exact function/type contracts with pseudocode for the intake boundary. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | The ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | The validation matrix (P2-V01/P2-V02), error/security/observability model, and the handoff checklist. |

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W01 plan](../../plans/p2-w01-boot-platform-description-intake.md) → this
design → Coding Guidelines. Binding constraints:

- The boot DTB is **untrusted input** (task book §2; Coding Guidelines;
  plan-agent guardrail on firmware input). Every offset, length, and pointer
  derived from it is checked before use; malformed input produces a bounded
  diagnostic, never uncontrolled behavior (P2-A04, P2-V01/P2-V02).
- Core must not branch on a board or platform name (ADR-043/ADR-052); intake
  validates structure only and never interprets device-specific content.
- Layering: intake lives in the platform/discovery layer described by ADR-041;
  it must not depend on architecture registers or SoC/Board code. The only
  hardware-facing dependency is the P1-provided host physical-access window
  ([01 §2](01-intake-boundary.md)).
- The task book requires P2 not to freeze "a parser implementation" in plans;
  the parser-level choices that the plan leaves open are fixed here as
  stage-local decisions with recorded rationale (Decisions D2–D4). A later ADR
  supersedes them if the architecture register decides differently.
- `unsafe` is confined to one audited boundary (byte access from the
  P1-supplied physical window) per ADR-006 and P0-W10 governance.

Classification. **Required:** the intake orchestrator, placement/overlap
validation, header/structure/reservation validation, the bounds-guaranteed
read-only cursor, the diagnostic taxonomy, and host-side testability of all
pure validation logic. **Reserved** (recorded triggers, no P2 implementation):
copying and later releasing the original DTB (task book Reserved list);
adopting an external FDT crate; DTB versions below the policy minimum;
accepting DTB-as-code execution paths (never); signed/measured boot description
(ADR-059). **Out of Scope:** platform-specific discovery and normalized
platform data (W02), memory-map construction or any physical-range claim (W03),
allocation (W04/W05), UART/console drivers and any PSCI/timer/GIC mechanism
(later stages), Orange Pi 3B runtime support (P15), and inspection output (W06).

## Requirement-to-design mapping

The tracked sources define P2-A01–A04 at group granularity only. The rows below
are this design's reviewable enumeration derived from the plan's scope wording;
they do not add authority.

| Plan requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-A01 (availability/placement) | Absent, misaligned, out-of-window, zero/oversized, and hypervisor-overlapping DTB locations each yield a distinct explicit outcome | [01 §4](01-intake-boundary.md), [03 §3](03-code-contracts-intake.md) | P2-V01 (W01-DV01–DV03) |
| P2-A02 (structural bounds) | Header fields, structure block, strings block, and reservation block are validated within total-size bounds; token stream is well-formed and depth/count-bounded | [03 §4](03-code-contracts-intake.md), [03 §5](03-code-contracts-intake.md) | P2-V02 (W01-DV04–DV06) |
| P2-A03 (foundational encodings) | Required encodings (big-endian u32 tokens, NUL-terminated strings block, length-prefixed properties, 8-byte reservation entries, terminating entries) are interpreted with checked ranges | [01 §5](01-intake-boundary.md), [03 §4–§6](03-code-contracts-intake.md) | P2-V02 (W01-DV05, DV07) |
| P2-A04 (unknown nodes) | Structurally valid unknown nodes/properties are safely ignorable; structurally suspicious shapes are reported, not rejected, at intake | [01 §6](01-intake-boundary.md), [03 §5](03-code-contracts-intake.md) | P2-V02 (W01-DV08) |
| Diagnosability (plan step 2) | Every rejection carries a local, distinct diagnostic naming the field/offset class; a rejected input is distinguishable from uncontrolled behavior | [01 §3](01-intake-boundary.md), [05 §2](05-validation-and-handoff.md) | P2-V01/P2-V02 (W01-DV09) |
| Contract handoff (plan step 6) | W02/W07/W08/W09 can consume the validated handle without re-validating or reparsing | [05 §4](05-validation-and-handoff.md) | W01 closure review (W01-DV10) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
the repository is a P0 documentation scaffold. There is no Cargo workspace,
no Rust source anywhere (`crates/`, `hypervisor/src/` hold only `.gitkeep`
markers), no parser code, and no P1 implementation or verification records.
Only P0-W01 (repository baseline) is implemented and verified; P0-W02 has an
approved-quality design; P0-W03 and later P0 packages and all P1 packages are
planned only. The P1 prerequisites W01 consumes (boot contract, image range,
physical-access window, diagnostics) therefore do not exist as code — they are
assumed contracts with explicit failure boundaries
([01 §2](01-intake-boundary.md)). This design authorizes no inference from
directory names to crates, targets, or module trees.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| DTB availability, location, length, host-access range, overlap yield explicit outcomes (P2-V01) | No boot-input code exists; no P1 boot contract record | Intake placement validator over P1-declared (DTB physical address, length) and image range | Without a placement gate nothing downstream can trust what it parses | P1 supplies inputs (assumed contract); W01 owns the validator | W01-DV01–DV03 host tests; QEMU observation deferred to W09 |
| Malformed structural/encoding inputs are bounded and diagnosable (P2-V02) | No DT validation code exists | Header/structure/reservation validators plus bounds-guaranteed cursor | Untrusted boot input must fail closed with a local diagnostic | W01 (this design) | W01-DV04–DV08 host tests over synthesized blob fixtures |
| Valid required encodings are interpreted without unsafe out-of-range behavior (task book §6) | No DT reading code exists | Cursor whose accesses are bounds-checked by construction; single small `unsafe` window boundary | The only raw-memory access must be one audited boundary | W01; P0-W10 audit process | W01-DV07; unsafe inventory entry at implementation time |
| Validated-intake contract recorded for discovery, offline checking, and negative regression consumers (plan step 6) | No contract exists beyond the plan wording | This design + its record file once implementation starts | Consumers need a named boundary and diagnostics, not plan prose | W01 | W01-DV10 closure review |
| P1 inputs available to the package (plan step 1) | P1 unimplemented; no boot contract, image range, or access window exists | Assumed-contract table with failure boundaries; blocked-upstream-defect rule | Task book §2 makes missing P1 inputs upstream defects; P2 must not work around them | P1 owners (P1-W01, P1-W08) | At integration: P1 verification records; until then, host tests inject synthetic inputs |

No ledger row requires a crate, target, remote, or runtime-policy decision that
no authority has granted; the blocked-item path for absent P1 inputs is stated,
not improvised.

## Resolved design decisions and their authority

1. **In-place DTB consumption; no copy or release.** Intake validates and
   reads the DTB where firmware placed it and keeps it protected for the whole
   boot phase. Basis: the task book lists "a later choice to copy and release
   the original DTB" as Reserved; no P2 authority authorizes a copy path.
2. **In-house bounded FDT validator and cursor; no external parser crate.**
   Rationale: dependency governance is P0-W18 and no approved dependency
   decision exists; the boot-critical parsing surface is small and must be
   auditable against the untrusted-input rules with checked arithmetic;
   external crates cannot be assumed to be `no_std`, allocation-free, and
   panic-free on malformed input. Adopting a mature FDT crate later is
   Reserved through P0-W18 governance.
3. **DTB version policy: accept only `version >= 17` and
   `last_comp_version <= 17`.** Rationale: v17 is the current DTB format
   version and the version emitted by the reference platform's toolchain;
   `size_dt_struct` exists only from v17, and a lower bound would widen the
   untrusted-input surface with no P2 consumer. Older versions are Reserved.
4. **All intake failures are fatal boot stops with distinct local
   diagnostics; no partial state is published.** Basis: P0-W14 failure
   classification (boot/platform failure class, stop path) and the task book's
   malformed-input diagnostic requirement. No P2 consumer exists that could
   recover; recovery policy is a later-stage decision.
5. **DTB overlapping the hypervisor image is a fatal rejection, not a
   relocation.** Basis: P2-V01 requires "dangerous hypervisor overlap" to
   yield an explicit outcome; with decision 1 (no copy) there is no safe
   continuation, so the only explicit outcome is a named fatal diagnostic.
6. **Alignment rules follow DT format mechanics, not convention:** blob base
   4-byte aligned (u32 token access), every block offset 4-byte aligned, and
   `base + off_mem_rsvmap` 8-byte aligned (u64 reservation entries). Derived
   from the format, not adopted as folklore.
7. **Host physical access goes through the P1 physical-access window
   (assumed contract).** W01 defines its own side of the contract (a
   checked byte-slice fabrication at one `unsafe` boundary); if P1 delivers no
   such window, W01 is blocked as an upstream defect — an identity map must
   not be improvised (P1-W08 prohibits an identity-map promise).
8. **Pure validation logic is host-testable by construction:** all validators
   and the cursor operate on byte slices; QEMU wiring is a thin adapter. Basis:
   P0-W08 host-test baseline and the task book's host-side negative
   validation (P2-V10 consumer).

## Work breakdown and loading order

1. Read [01-intake-boundary.md](01-intake-boundary.md) to understand the input
   contracts, the untrusted-input model, and the diagnostic taxonomy.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module decomposition, ownership, lifecycle, and failure model.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md)
   in order; each step points back into
   [03-code-contracts-intake.md](03-code-contracts-intake.md) for the exact
   contracts and pseudocode.
4. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record implementation decisions and deviations in
   `../p2-w01-boot-platform-description-intake-record.md`, and commands,
   environments, and results in
   `../../verification/p2-w01-boot-platform-description-intake-verification.md`
   — both files are created only when the corresponding work starts; neither
   this design nor a written record may claim W01 complete.

## Explicitly excluded interfaces

W01 authorizes no public API beyond the intake contracts in
[03](03-code-contracts-intake.md): no platform-fact types (W02 owns
`PlatformInfo`-shaped output), no physical-range or map types (W03), no
allocator types (W04/W05), no crate or workspace manifest, no CI or command
surface, no console driver, and no guest/device interface. Any consumer need
outside the listed contracts is a design change, not a local addition.

## Downstream handoff

- **W02** ([../p2-w02-platform-discovery-normalization/README.md](../p2-w02-platform-discovery-normalization/README.md))
  receives `ValidatedBootDtb` plus the bounds-guaranteed cursor and the
  diagnostic taxonomy; it must not re-validate the blob or bypass the cursor.
  W02 owns all semantic interpretation (cells, `reg`, compatible strings).
- **W03** ([../p2-w03-boot-memory-map-ownership/README.md](../p2-w03-boot-memory-map-ownership/README.md))
  receives the validated DTB physical range and the validated reservation
  entries as protected-range inputs; W01 claims no range itself.
- **W07** ([../p2-w07-offline-dtb-compatibility/README.md](../p2-w07-offline-dtb-compatibility/README.md))
  reuses the same validators and cursor on host for offline DTB checking; the
  contracts are host-runnable by construction (Decision 8).
- **W08/W09** ([../p2-w08-host-robustness-regression/README.md](../p2-w08-host-robustness-regression/README.md),
  [../p2-w09-qemu-integration-regression/README.md](../p2-w09-qemu-integration-regression/README.md))
  receive the negative-case taxonomy (which malformed inputs must be rejected
  with which diagnostics) as regression fixtures and the boot-phase
  observability expectations; they own the harnesses.
- **P3/P4** consume only what W10 later records; W01 exposes nothing to them
  directly.
