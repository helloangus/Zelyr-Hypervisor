# P8-W18 Security and Isolation Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Guest-untrusted security and isolation regression — scenario
matrix, security properties, containment expectations, and crash/failure
oracles — required by [P8-W18](../../plans/p8-w18-security-isolation-regression.md).  
**Owner/change context:** P8-W18 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W18. It defines the regression
that must show the P8 Linux integration *retains* the Guest-untrusted
containment boundary established by P4–P7 rather than regressing it: a
scenario matrix in which every row states its security property, attack or
fault vector, controlled expected result, containment expectation, and
explicit crash/failure oracles ([01](01-isolation-scenario-matrix.md)), plus
the execution workflow, oracle review, and escalation rules
([02](02-workflow-oracles-and-handoff.md)). It is a validation design: it
contains no results and no containment claim. Governing principle, taken from
the plan: **a Linux panic is never equivalent to a Hypervisor panic** —
Guest-scoped failure is the expected outcome of most rows, and only
Hypervisor/Host/other-context harm is a failure.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
[01](01-isolation-scenario-matrix.md) for scenario content and
[02](02-workflow-oracles-and-handoff.md) for execution, classification, and
escalation. Before editing, the agent must also follow the Coding Guidelines
preflight (repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P8 task
book](../../task-book-v0.1.md), and the P8-W18 plan). This document claims no
test has run.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W18 plan → this
design → Coding Guidelines. In particular:

- ADR-007 (Guests are potentially malicious), ADR section 12 (untrusted
  boundaries; resource-ownership invariant; panic policy distinguishing
  Guest fault from Hypervisor invariant), and ADR section 19 (`MUST`:
  unvalidated Guest addresses never used; Guest-caused faults stay
  VM-scoped; capability-check failure never degrades to implicit allow) are
  the properties this regression observes. They are not re-derived or
  weakened here.
- The task book binds this package to P8-V24: malformed Guest requests,
  panic, loop, and storm remain contained to the VM context and cause no
  Hypervisor/Host/other-VM irreversible failure. The conditions under which
  P8-V24 could pass are defined here; nothing asserts they were met.
- The plan's exclusions are binding: **no new capability policy, no device
  assignment, no IOMMU isolation, no global-recovery design, and no
  declaring all Guest errors harmless.** Every row must name the boundary it
  protects; a row that cannot name one is invalid.

Classification. **Required** for W18 closure: the scenario matrix with
security properties and oracles, the cross-VM and Host-protection
observations as scoped in §4 of [01](01-isolation-scenario-matrix.md), the
failure-classification and escalation rules, and the evidence destinations.
**Reserved** with recorded triggers: multi-VM co-existence containment on one
Hypervisor instance (trigger: P10+ management-domain work), IOMMU/DMA
isolation rows (trigger: P14), real-hardware execution of the matrix
(trigger: P15), and fuzz-style automation beyond the declared scenarios
(trigger: an approved fuzzing design; P5's host-side fuzzable parser layer
remains the P5-scoped predecessor). **Out of Scope:** implementing or
changing containment mechanisms; new security policy; treating QEMU
containment as hardware containment; and W16's harness ownership (W18
consumes it).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W05–W13, W16, and P5 security facts (work seq 1) | README ledger; [workflow](02-workflow-oracles-and-handoff.md) §1 | prerequisite-contract review (W18-DV01) |
| Define each malicious or abnormal scenario and controlled expected result (work seq 2) | [matrix](01-isolation-scenario-matrix.md) §3 | P8-V24 (W18-DV02) |
| Relate failures to VM/vCPU diagnostics and scheduler/interrupt containment (work seq 3) | [matrix](01-isolation-scenario-matrix.md) §3 containment columns | P8-V24 (W18-DV02) |
| Define cross-VM and Host-protection observations (work seq 4) | [matrix](01-isolation-scenario-matrix.md) §4 | P8-V24 (W18-DV03) |
| Review unresolved failure classes as security or architecture blocks (work seq 5) | [workflow](02-workflow-oracles-and-handoff.md) §3 | W18 closure review (W18-DV05) |
| Hand factual containment limits to W20 and P9+ only after evidence exists | README handoff; [workflow](02-workflow-oracles-and-handoff.md) §6 | consumability review (W18-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation-only P0 scaffold — no hypervisor, no Guest assets, no regression
harness. P5's capability/HVC security facts, W05's behavior classification,
W06's PSCI boundary, W13's fault taxonomy, and W16's harness are all planned
contracts, not delivered evidence. Every input below is an assumed contract
with a stated failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P8-V24: malformed requests, panic/loop/storm contained to the VM context | No scenario set exists in any tracked file | Scenario matrix with per-row security property, vector, expected result, containment expectation, oracles ([01](01-isolation-scenario-matrix.md) §3) | A containment claim is only reviewable when the attack surface is enumerated row by row | W18 (this design); expected-result content from W05/W06/W13 classifications | W18-DV02 matrix review |
| Protection of Hypervisor and Host | No Host-protection observation defined | Host-protection oracle set ([01](01-isolation-scenario-matrix.md) §4.1) | "Hypervisor/Host unaffected" must be an observable, not an assumption | W18; diagnostics content owned by W13 | W18-DV03 review |
| Protection of other VM and other vCPU | P8 Linux scope is a single Linux VM; the P5 dual-context suite is the only planned cross-context asset | Proof-boundary definition: in-P8 substitutes plus retained dual-context mechanism evidence via W19 ([01](01-isolation-scenario-matrix.md) §4.2) | P8 cannot fabricate a multi-VM claim its scope does not support; the boundary must be recorded, not implied | W18 (boundary); P5-W07 dual-context suite (mechanism evidence); W19 (retention) | W18-DV03 review; dual-context evidence via W19 rows |
| Containment tied to diagnostics and scheduler/interrupt behavior | W13 taxonomy and W11 integration checks planned only | Per-row linkage to W13 diagnostic classes and W10/W11 scheduler/interrupt containment expectations | A containment regression is only detectable if the diagnostic and scheduling context is declared observable | W13 classes; W10/W11 expectations; W16 envelope | row linkage present in matrix |
| Fault-injection capability | No declared injection mechanism exists | Injection mechanisms consumed from W15 fixture and W13-declared triggers | An isolation row without a declared trigger is not executable | W15 fixture; W13 triggers; P4–P7 VG triggers | **failure boundary:** missing trigger ⇒ row blocked, never improvised |
| Unresolved failure classes → blocks | n/a | Escalation rules ([02](02-workflow-oracles-and-handoff.md) §3) | The plan requires boundary violations to block, not to be patched | W18 | W18-DV05 review |

No row above selects security policy or invents a mechanism. Concrete v1
machine values remain an inherited `ADR Required` item (task book §8, routed
via [P8-W02](../p8-w02-machine-contract-governance/README.md)); isolation
rows reference the approved boundaries, never fixed addresses.

## Resolved design decisions and their authority

1. **Property-first rows.** Every scenario row is registered against one of
   four ADR-derived security properties — P-1 Host memory isolation, P-2
   Guest-visible surface integrity (controlled rejection of unsupported or
   out-of-contract operations), P-3 VM-scoped failure (Guest faults never
   escalate), P-4 resource liveness under abuse (scheduler/interrupt
   responsiveness). Rationale: the plan requires "each malicious or abnormal
   scenario and controlled expected result"; naming the property per row
   makes the expected result derivable from authority instead of taste.
   Stage-local structure owned by this design.
2. **Oracle asymmetry.** Row failure is defined by harm (Hypervisor panic,
   fatal EL2 state, harness/control-path death, cross-scenario
   contamination, unexpected fault class); Guest-visible breakage inside the
   attacked VM is the *expected* controlled outcome of most rows. Rationale:
   the plan's acceptance sentence; prevents "Guest crashed ⇒ test failed"
   inversion, which would re-declare Guest errors harmful.
3. **In-P8 substitute oracles for other-VM containment.** Within P8's
   single-Linux-VM scope, "other VM unaffected" is observed as: (a) the
   harness control path and EL2 diagnostics remain responsive during and
   after abuse, (b) a subsequent clean boot of the Guest succeeds in the
   same session, and (c) the retained P5 dual-context mechanism suite
   (via [W19](../p8-w19-validation-guest-dual-track/README.md)) continues to
   show one context cannot use another's authority. Full multi-VM
   co-existence containment is explicitly **not claimed** by P8 and remains
   Reserved. Rationale: honest proof boundary within the approved scope; the
   alternative (claiming more than P8 can observe) would be a security
   overstatement.
4. **Track pairing per row.** Each row declares whether it executes on the
   Linux track, the Validation Guest track, or both, with the reason. Where
   both exist, the VG row tests the mechanism precisely and the Linux row
   tests OS-driven integration of the same boundary. Rationale: ADR-008/009
   ordering (mechanism first) and the plan's refusal to let Linux success
   stand in for mechanism proof.
5. **Repetition and storm discipline.** Abuse rows declare bounded repetition
   counts and storm durations as recorded run parameters (no fixed magic
   number in the contract, consistent with W16's policy); a storm row ends
   by declared termination, not by harness timeout alone, so that "stopped
   because the wall clock ran out" is distinguishable from "contained by the
   Hypervisor". Stage-local policy owned by this design.
6. **Evidence destinations.** Verification record
   `docs/stages/p8/verification/p8-w18-security-isolation-regression-verification.md`
   plus transcripts and run records under
   `docs/stages/p8/verification/assets/p8-w18/`, following the W16
   evidence conventions. Containment evidence must retain the full diagnostic
   transcript, not a summary verdict, because the *content* of the diagnostic
   (VM/vCPU context, no Host leakage) is part of the oracle.

## Work breakdown and loading order

1. Read [01](01-isolation-scenario-matrix.md): properties (§2), the scenario
   matrix (§3), Host/cross-VM observations (§4), and oracle definitions (§5).
2. Read [02](02-workflow-oracles-and-handoff.md): preconditions, execution
   under the W16 envelope, classification, escalation, validation matrix,
   observability model, and handoff checklist.
3. Execute the workflow of [02](02-workflow-oracles-and-handoff.md) §2 (steps 1–5)
   when implementation is authorized. Actual runs, transcripts, and
   run/not-run status go to
   `../../verification/p8-w18-security-isolation-regression-verification.md`;
   factual decisions go to
   `../p8-w18-security-isolation-regression-record.md` — both created only
   when that work begins. Nothing here claims containment was demonstrated.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
or persistent layout is designed or authorized by W18. The Guest-side
trigger mechanisms (VG fault triggers, Linux fault-injection hooks) remain
owned by the P4–P7 suite designs and the W15 fixture; W18 only *requires* a
declared trigger per row and consumes it. No new hypercall, diagnostic
format, policy engine, or containment code path is designed here. Adding any
of these under W18 authority is a scope conflict to stop at review.

## Downstream handoff

Per the [plan index](../../plans/README.md) consumer map:

- **W20** consumes the scenario matrix, oracle definitions, evidence paths,
  and — only if evidence exists — the factual containment-limit statement for
  the P8 evidence index; W20 must carry the limits (single-VM scope, QEMU
  scope, declared storm bounds) next to any containment statement.
- **P9+** receives the factual containment limits and the oracle set as
  inputs for virtio-descriptor and later security work; P9 must not cite
  P8-V24 as multi-VM, DMA, or hardware isolation evidence — those are
  Reserved (P10+ management, P14 IOMMU, P15 hardware).
- **Future maintenance stages** receive the property taxonomy (P-1..P-4) as
  the stable registration scheme for future abuse scenarios.
