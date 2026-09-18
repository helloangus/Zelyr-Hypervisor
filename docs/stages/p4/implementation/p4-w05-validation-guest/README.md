# P4-W05 Rust Validation Guest Test Asset — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The maintained Rust `no_std` Validation Guest and its explicit P4
scenario set (P4-D01–D08, P4-J; VG-001–VG-012), as required by
[P4-W05](../../plans/p4-w05-validation-guest.md).  
**Owner/change context:** P4-W05 implementation handoff; this design owns the
Guest runtime structure, the scenario table and marker protocol, and the
Guest-side interpretation of the P4 boot inputs.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W05. It converts the bounded
work-package plan into the Guest runtime modules, the scenario table with
per-scenario observable outcomes, and the guest-side contracts (boot-info
parsing, console output, fault triggers), with pseudocode rather than
production code. It deliberately does **not** design the hypervisor side of
the boundary — image embedding and Guest RAM construction
([P4-W03](../p4-w03-guest-memory-image/README.md)), the Stage-2 mappings the
scenarios exercise ([P4-W02](../p4-w02-stage2-address-space/README.md)), the
world-switch and exit handling ([P4-W04](../p4-w04-vcpu-entry-exit/README.md)),
fault classification and isolation diagnostics
([P4-W06](../p4-w06-fault-isolation-diagnostics/README.md)), QEMU automation
([P4-W08](../p4-w08-qemu-integration-regression/README.md)) — and it does not
define a production Guest ABI, Guest DTB, virtio, virtual interrupts/timer,
or any permanent repository-layout decision.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — ledger,
  assumed contracts with failure boundaries, scope classification, resolved
  decisions, and the scenario-provenance finding (the canonical VG-001–VG-012
  definition text is not tracked in this repository; this design owns the P4
  reconstruction). Load first.
- [02-guest-architecture-and-scenarios.md](02-guest-architecture-and-scenarios.md)
  — Guest runtime modules, the boot-info and console contracts, the scenario
  framework, and the VG-001–VG-012 table with expected host-observable
  outcomes per scenario. Load for architecture and scenario work.
- [03-code-contracts-guest.md](03-code-contracts-guest.md) — Guest-side
  function/type contracts with pseudocode: entry, boot-info validation,
  console writer, scenario dispatch, fault triggers, controlled stop, panic
  handler; plus the host-side scenario-validation contract W04 consumes.
  Load for the code-contract work area.
- [04-implementation-workflow.md](04-implementation-workflow.md) — ordered
  implementation steps.
- [05-validation-and-handoff.md](05-validation-and-handoff.md) — validation
  matrix (P4-V05, P4-V06, P4-V08, P4-V09 scenario inputs), error/security/
  observability model, and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W05 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W05 plan](../../plans/p4-w05-validation-guest.md) → this design → Coding
Guidelines. Binding constraints include:

- ADR-009/ADR-021: the first Guest is a Rust bare-metal Validation Guest at
  EL1, ELF-or-simpler evolution order; ADR-022: EL1 only, no virtual EL2.
- ADR-007: the Guest is *treated as untrusted by the hypervisor* even though
  it is our own test asset — the Guest must never rely on hypervisor trust
  (that is the property the stage tests), and the Guest itself validates its
  boot inputs defensively.
- Task book §1 Required: EL1, normal-memory, unmapped, permission, WFI/WFE,
  and controlled illegal-behavior scenarios; the asset is "maintained."
  Task book §6: VG-001–VG-007, VG-010, VG-012 are mandatory; VG-008, VG-009,
  VG-011 are planned and deferrable only by an explicit stage-review record.
- Task book §1 Out of scope: Linux, libc, firmware dependency, production
  guest ABI, Guest DTB, virtio, virtual interrupts/timer, scheduler, and any
  permanent repository-layout decision.

Classification: the Guest runtime modules, scenario table, marker protocol,
and Guest-side contracts ([02](02-guest-architecture-and-scenarios.md),
[03](03-code-contracts-guest.md)) are **Required** for P4-D01–D08/P4-J. The
planned scenarios (VG-008/VG-009/VG-011) are **Required-if-feasible with an
explicit deferral record path** per the task book rule. Linux Guest, libc,
firmware-dependent boot, production ABI, DTB, virtio, interrupt/timer
scenarios (P6+), scheduler scenarios (P7+), and hypercall scenarios (P5) are
**Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-D01 directly loadable Rust bare-metal Guest | [foundations](01-scope-and-foundations.md) D2; [architecture](02-guest-architecture-and-scenarios.md) §2 | P4-V03/V05 (loads and runs via W03 route) |
| P4-D02 observable EL1 marker | [scenarios](02-guest-architecture-and-scenarios.md) VG-001; [contracts](03-code-contracts-guest.md) §3 | P4-V05 |
| P4-D03 CurrentEL confirmation | [scenarios](02-guest-architecture-and-scenarios.md) VG-001; [contracts](03-code-contracts-guest.md) §3.2 | P4-V05 |
| P4-D04 RAM/stack/code execution proof | [scenarios](02-guest-architecture-and-scenarios.md) VG-002/VG-003 | P4-V05 |
| P4-D05 unmapped and permission-fault triggers | [scenarios](02-guest-architecture-and-scenarios.md) VG-004/VG-005/VG-006; [contracts](03-code-contracts-guest.md) §4 | P4-V06, P4-V08 |
| P4-D06 WFI/WFE behavior | [scenarios](02-guest-architecture-and-scenarios.md) VG-007/VG-008 | P4-V09 |
| P4-D07 controlled illegal behavior | [scenarios](02-guest-architecture-and-scenarios.md) VG-010/VG-011 | P4-V06, P4-V09 |
| P4-D08/P4-J VG-001–VG-012 explicit scenario set | [scenario table](02-guest-architecture-and-scenarios.md) §5 (with provenance finding) | P4-V05/V06/V08/V09; mandatory/planned split per task book §6 |
| Test-asset maintenance boundary | [architecture](02-guest-architecture-and-scenarios.md) §6; [handoff](05-validation-and-handoff.md) §5 | maintenance review; W09 records facts |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs`
at `4e631ee`): documentation-only repository. `guests/validation-aarch64/`
contains only `.gitkeep`; no crate, no target definition, no Guest binary, no
W03 route, no W04 path. Additionally: the canonical VG-001–VG-012 per-scenario
definition text ("the source task book" cited by the P4 task book §6) is **not
tracked anywhere in this repository or its git history** — the superseded
root-level document was never committed. The task book itself (governing
source) states the mandatory/planned split, and the W05 plan's scope sentence
enumerates the scenario classes; both are used in [02 §5](02-guest-architecture-and-scenarios.md)
as the reconstruction basis, with the provenance gap recorded as an open
question (W01 ledger item A9).

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Directly loadable `no_std` Rust Guest | No Guest crate exists | Guest crate structure and flat-binary build route (assumed M5 build/target baseline; W03 D3 route) | "Directly loadable" means the W03 embedding route consumes its artifact unchanged | W05 owns the crate; build governance assumed | build produces the artifact (P4-V03 path) |
| Observable EL1 marker, CurrentEL proof | Nothing exists | Console writer over the W03 console-page mapping + VG-001 scenario | markers are the stage's observation channel | W05 (Guest side); W03 (mapping) | P4-V05 |
| Scenario set exercising the boundary | No scenario definitions tracked (provenance gap) | The reconstructed VG-001–VG-012 table ([02 §5](02-guest-architecture-and-scenarios.md)) as the P4 test contract | W06/W08 need stable triggers and expected markers | W05 (this design), pending stage-owner confirmation | scenario-table review; per-scenario evidence |
| Maintained test asset | Nothing exists | Marker protocol versioning + scenario-table ownership statement | "Maintained" implies reviewable, versioned change rules | W05 | maintenance review row |
| WFI/WFE and controlled illegal scenarios produce diagnosable results | Nothing exists | Scenario triggers coordinated with W04 exit classes | W04's classes are the outcome vocabulary the triggers map to | W05 triggers; W04 classes; W06 diagnosis | P4-V09 |

No row requires inventing hypervisor mechanisms; the Guest is strictly a
consumer of the W02–W04 boundaries.

## Resolved design decisions and their authority

Summarized; full rationale in [01 §4](01-scope-and-foundations.md):

1. **Single image, scenario by register:** one Guest image; the scenario id
   arrives in the initial register (x0) set by W04 construction and is echoed
   in the W03 boot-info block; the Guest dispatches on the validated value.
2. **Marker protocol:** fixed ASCII `VG-<id>:<EVENT>` lines over the console
   page; versioned protocol string in the boot-info echo path; W08 consumes
   the grammar.
3. **Boot-info defensive parsing:** the Guest validates magic, version,
   size, and checksum before using any field; invalid boot info is a
   Guest-visible failure marker, not a Guest hang.
4. **Fault triggers are data-driven:** each fault scenario reads its target
   address from the boot-info layout echo (never a Guest-side invention),
   so Stage-2 layout changes cannot silently break scenario semantics.
5. **Controlled stop:** scenarios end in defined terminal behavior (WFI for
   completion, deliberately-trapped fault for fault scenarios) that W04's
   action policy maps to a defined stop; no Guest code ever performs an
   operation whose outcome is unclassified.
6. **Guest panic policy:** the Guest panic handler emits a `VG-PANIC` marker
   and halts in a WFI loop — a diagnosable controlled condition for W04/W06,
   never silent corruption.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): ledger,
   assumed contracts (M-series), decisions D1–D7, provenance finding.
2. Load [02-guest-architecture-and-scenarios.md](02-guest-architecture-and-scenarios.md)
   for the runtime module map and the authoritative P4 scenario table with
   expected outcomes.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md),
   loading [03-code-contracts-guest.md](03-code-contracts-guest.md) for each
   Guest-side contract; the host-side scenario-validation duty is cited to
   [P4-W04](../p4-w04-vcpu-entry-exit/README.md) where noted.
4. Record validation in
   `../../verification/p4-w05-validation-guest-verification.md` and facts in
   `../p4-w05-validation-guest-record.md` only when work starts; no
   completion claims.

## Explicitly excluded interfaces

Not designed or authorized by W05: any hypercall/HVC convention (P5 owns the
HVC ABI; the P4 Guest makes no HVC calls), capability/handle semantics (P5),
virtual timer/interrupt scenarios (P6), scheduler-related scenarios (P7),
Guest DTB/PSCI/boot-protocol work (P8), virtio (P9), a production guest
console subsystem, and any QEMU-specific Guest behavior beyond the
W03-recorded console-page convention (the Guest addresses hardware only
through the layout-record contract). The scenario table is a P4 test
contract, never a machine ABI (W01 A4).

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W06** receives the controlled triggers (per-scenario expected trap/
  fault site, syndrome class) needed to prove distinguishable classification
  and the Guest-vs-hypervisor fault boundary.
- **P4-W08** receives the marker grammar and the stable per-scenario expected
  marker sequences for automation.
- **P4-W07/W09** receive the scenario/maintenance facts (protocol version,
  scenario table version, any deferred-scenario record path per the task
  book's deferral rule).
- **P4-W04** (upstream consumer of this design's table) receives the
  scenario-id validation table used by vCPU construction.
- **P5** inherits a maintained validation asset only; hypercall scenarios
  are P5's own work ([P5-W07](../../../../stages/p5/plans/p5-w07-validation-guest-isolation-suite.md)
  is the future consumer of this asset's conventions).
