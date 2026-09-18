# P6-W08 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md).

## 1. Governing constraints

1. **Architecture ADR baseline**
   ([../../../../adr/adr-000-architecture-baseline-v0.1.md](../../../../adr/adr-000-architecture-baseline-v0.1.md)):
   GICv3 mainline with the virtualization interface as the growth path
   (ADR-032: software pending queue first, LRs/maintenance next); vCPU
   switch must explicitly handle vGIC state (section 5); capability-driven
   selection (ADR-044/052); Guest untrusted (ADR-007); typed ownership and
   bounded IRQ work (section 12/13); telemetry by design (ADR-048).
2. **P6 task book** ([../../task-book-v0.1.md](../../task-book-v0.1.md)):
   P6-H owned by W08, validated by P6-V11/P6-V16/P6-V18; §1 requires
   hardware presentation through the GICv3 virtualization interface with
   List-Register pressure handling; §8 fixes that the exact supported GIC
   revision and feature interpretation follow authoritative
   specification/platform review, and classifies LR allocation as
   Implementation Choice resolved here; §2 makes W01 capability
   conclusions and the W07 lifecycle entry conditions.
3. **P6-W08 plan**
   ([../../plans/p6-w08-gic-virtualization-interface.md](../../plans/p6-w08-gic-virtualization-interface.md)):
   goal, scope (readiness, presentation capacity, vCPU-context
   preservation, basic priority carriage, safe pressure), acceptance, and
   the explicit exclusions (maintenance policy, Guest vGIC MMIO, machine
   ABI, exact LR allocation algorithm as a *plan-level* item — the design
   fixes a bounded policy here and marks its evolution Reserved).
4. **Coding Guidelines**
   ([../../../../development/coding-guidelines.md](../../../../development/coding-guidelines.md)):
   MMIO/register rules — volatile access, reserved-bit handling, barriers,
   ordering; minimal audited `unsafe` with `SAFETY` justifications; no
   unbounded work in IRQ/maintenance context; QEMU success is not evidence
   the hardware rules can be omitted.

## 2. Current-state findings

Observable facts (worktree `docs/p6-implementation-designs`, audited
2026-09-18): P0 documentation scaffold only; no implemented W01 capability
review, W02 physical GIC bring-up, W07 lifecycle, or P4 vCPU transition
context; no P6 sibling designs in this worktree yet. W08 designs against
the plans' stated handoffs (§5) with an entry review at workflow step 1.
The GIC revision, virtualization feature bits, and LR capacity are treated
as **discoverable facts**, never as assumptions: the design names the
discovery points, the sanity checks, and the failure behavior, and requires
the spec revision to be locked at implementation (task book §8).

## 3. Goal-to-baseline ledger

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P6-V11 share: "supported presentation occurs" for an authorized event | No virtualization interface, LR table, or W07 lifecycle exists | W08-owned readiness + LR presentation consuming the W07 claim protocol; W07 lifecycle itself is an assumed sibling contract | Presentation is meaningless without W07 pending truth; the LR programming is W08's deliverable | W07 (state, assumed); W08 (presentation) | W08-DV02/DV03 with W07 evidence |
| P6-V16: "more pending vIRQs than presentable slots produces no loss or overwrite and leaves excess work pending" | Nothing exists | W08 capacity discovery + bounded fill policy + exit purge that returns LR state to W07 (D4–D6) | The no-loss property must hold structurally: capacity is discovered, fill is bounded, overflow is preserved in W07 | W08 (this design) | W08-DV04 |
| P6-V18 share: vCPU state stays isolated | Nothing exists | Per-pCPU ownership (D2) + full exit purge before another vCPU may run on the pCPU | Cross-vCPU contamination is only possible if interface state survives a transition — the purge/restore discipline closes it | W08 (this design); P4 transition exclusivity | W08-DV03/DV06 |
| Virtualization-interface readiness (plan scope) | W01/W02 unimplemented | Assumed: W01 capability conclusions including virtualization support (`../p6-w01-gic-capability-discovery/README.md`); W02 Host GIC + per-pCPU local readiness (`../p6-w02-physical-gic-bring-up/README.md`) | Enabling the virtual interface on an unverified platform is exactly what W01 exists to reject | W01/W02 (assumed); W08 readiness gate | W08-DV01 |
| Consumed W07 protocol | W07 planned | Assumed: `virq_select_next_pending` / `virq_presentation_returned` / completion path (`../p6-w07-virtual-interrupt-core/README.md`) | The plan requires pending vIRQ selection from the lifecycle; W08 adds no private pending state | W07 (protocol owner) | step-1 reconciliation |
| vCPU transition context | P4 unimplemented | Assumed: P4 entry/exit hooks and vCPU arch-extension area (`../../../p4/plans/p4-w04-vcpu-entry-exit.md`, `../../../p4/plans/p4-w09-closeout-p5-handoff.md`) | Save/restore sequences need defined, exclusive transition points and a home for interface context | P4 (assumed); W08 sequences | W08-DV03 |
| Maintenance enabling boundary | W09 planned | Assumed: W09 processing consumes W08 primitives (`../p6-w09-maintenance-interrupt/README.md`) | Enabling maintenance sources without a consumer would produce unhandled interrupts; the boundary must be co-designed | W09 (policy); W08 (hardware access) | step-1 reconciliation |
| GIC revision/spec lock | Open per task book §8 | Implementation-time authoritative specification review recorded in the implementation record | Register sequences must be checked against a locked revision, not memory | W08 step 1 (record) | recorded revision + evidence |

No row requires inventing a crate, dependency, machine ABI, or scheduler
policy; no decision blocker is outstanding for this design.

## 4. Resolved design decisions and their authority

**D1 — Capability-gated readiness.** The per-pCPU virtual interface is
enabled only when (a) W01's capability conclusions declare virtualization
support for the platform, (b) W02's per-pCPU local GIC readiness is
acknowledged, and (c) the pCPU's own interface discovery (system-register
interface availability, virtual-interface identification, LR capacity,
priority/preemption widths) matches the declared capability envelope. Any
mismatch or absence disables presentation for that pCPU with a precise
diagnosis — W08 never enables on a guess and never falls back to a
non-LR path (that fallback would silently redefine P6 semantics; W07's
pending state is the correct resting place). Authority: task book §1/§8;
W01/W02 assumed contracts; ADR-044.

**D2 — Per-pCPU ownership, running-pCPU-only access.** Every
virtualization-interface register (system-register interface controls,
virtual-interface control, VMCR image, active-priority registers, LRs,
maintenance status) is per-pCPU hardware and is written/read only while
that pCPU is running the vCPU being loaded/unloaded, or during per-pCPU
readiness. No cross-pCPU interface access exists in any path. Rationale:
the architecture binds these registers to the physical interface; remote
manipulation is impossible by design, so the ownership rule removes the
race class rather than locking against it. Authority: ADR section 5; task
book §1 isolation requirement.

**D3 — Software (hw=0) presentation only in P6.** Every LR is programmed as
a software virtual interrupt (hardware-mapped bit clear); the vINTID comes
from W07's claim and no pINTID is referenced. Hardware-mapped (hw=1)
presentation — which couples LR completion to physical interrupt state —
is Reserved for the passthrough stage and its own design. Rationale: keeps
W07's software truth the single pending authority, keeps physical
deactivation out of P6 presentation, and makes the W06 timer conversion
(D6 of W06) the only place physical interrupt state is held. Authority:
task book §8; ADR-032 growth path; W06/W07 designs.

**D4 — Discovered capacity, no assumed count.** LR capacity and priority/
preemption widths are read from the virtualization-interface identification
register per pCPU at readiness; Core contains no LR count constant. The
table is sized to the discovered count (architecture minimum 4, maximum
16 — bounds used only as sanity checks on discovery, not as assumed
values). A discovery result outside the sanity envelope is a readiness
failure. Authority: task book §8; ADR-052.

**D5 — Fixed transition sequences with barrier placement.** Entry:
restore the vCPU's VMCR image and active-priority registers, load LRs from
W07 selections up to capacity (priority order), then enable the virtual
interface and the maintenance sources the W09 contract names, with a DSB
before Guest entry (ERET). Exit: read and hand off critical maintenance
state per the W09 boundary, purge every LR (returning each to W07 with its
observed outcome), save the VMCR image and active-priority registers into
the vCPU record, disable the virtual interface, DSB, before the vCPU is
regarded as unloaded. Rationale: preserves state across transitions
(P6-V11/V16 basis), makes cross-vCPU contamination structurally impossible
(P6-V18 share), and gives W07 a consistent view at every boundary.
Authority: task book §1 (state save/restore); Coding Guidelines barrier
rules; P4 transition assumed contract.

**D6 — Deterministic bounded pressure policy.** At load and refill, LRs
are filled in W07 selection order (priority, then lowest vINTID — W07's
order) up to discovered capacity; excess pending work simply stays
pending in W07 and is presented by later refills. No preemption,
eviction-of-loaded-LR, or fairness mechanism exists in P6 (eviction
happens only at exit purge or W9-reported maintenance). Rationale:
"no loss, no overwrite, excess pending" is exactly the P6-V16 wording; a
bounded deterministic policy is reviewable and does not pre-empt W10 or
P7 fairness concerns. Authority: task book §8; W08 plan acceptance.

**D7 — Priority carriage.** The LR priority field receives the W07-claimed
priority, masked/shifted to the discovered priority width; the vCPU's
VMCR image (loaded at entry) carries the Guest's run-control view with
P6 defaults documented. No priority preemption or guest-chosen priority
logic exists here (W10's). Rationale: "basic priority carriage" is plan
scope; everything beyond carriage is W10 semantics. Authority: W08 plan
scope; W10 boundary.

**D8 — Maintenance boundary.** W08 enables the maintenance-interrupt
sources named by the W09 contract (EOI-count and no-pending classes for
P6), exposes atomic reads of the maintenance status and EOI-error state,
and provides the refill primitive W09 calls after processing. W09 owns
recognition, correlation, and policy. W08's own IRQ-context work is
bounded by the discovered LR count. Rationale: one owner per datum —
hardware access (W08) vs. processing policy (W09) — and a bounded
maintenance path. Authority: task book §8; W09 plan (consumes W08).

## 5. Assumed upstream contracts and failure boundaries

| Contract | Source (plan path) | W08 relies on | If delivered differently |
|---|---|---|---|
| Virtualization capability conclusions | `../p6-w01-gic-capability-discovery/README.md` (design) | declared support, feature envelope, rejection precedents for the platform | absence/mismatch → readiness failure with diagnosis (D1); a platform without support is a W01 rejection, not a W08 problem |
| Host GIC + per-pCPU local readiness; group configuration (Group 1, Non-secure) | `../p6-w02-physical-gic-bring-up/README.md` (design) | local interface usable before virtual enablement; Group-1/NS baseline consistent with the physical configuration | if the physical design lands on a different group policy, the W08 LR group field policy ([04] §3) is amended against it — never diverged from |
| W07 claim/return/completion protocol | `../p6-w07-virtual-interrupt-core/README.md` (design) | selection, claim tokens, return outcomes, completion reporting | protocol change → cross-design amendment with W07/W09; W08 never keeps private pending state |
| vCPU transition points and arch-extension area | `../../../p4/plans/p4-w04-vcpu-entry-exit.md`; `../../../p4/plans/p4-w09-closeout-p5-handoff.md` | exclusive entry/exit hooks; storage for the interface context (VMCR image, active-priority image) | no extension point → boundary conflict with the P4 design; W08 does not build a parallel store |
| Maintenance processing | `../p6-w09-maintenance-interrupt/README.md` (design) | the named enable bits and processing split (D8) | unprocessed maintenance classes stay disabled until W09's contract covers them; no enable without a consumer |
| Physical GIC contract (no redesign) | W02 design by plan-path citation (this design consumes; physical bring-up is not re-specified here) | the delivered physical configuration as fact | any need to change physical behavior is W02 scope — raise there |

## 6. Scope classification detail

**Required:** per-pCPU readiness with discovery and gating; the LR table
with slot lifecycle; entry/exit sequences (context save/restore, purge);
software-LR programming with priority carriage; bounded pressure policy;
maintenance-status read and refill primitives for W09; per-pCPU
diagnostics (presented/purged/refilled counts, pressure depth, EOI-error
counts, discovery values recorded); the P6-V11/V16/V18-oriented scenarios
in [06](06-validation-and-handoff.md); the recorded specification-revision
lock.

**Reserved:** hardware-mapped (hw=1) LRs and passthrough (later stage);
LR eviction/preemption policy; fairness beyond D6; Guest-visible priority
semantics (W10); maintenance policy (W09); vGIC MMIO modeling (P8);
serialization of interface state (P16/P17).

**Out of Scope:** physical GIC bring-up (W02); vIRQ state ownership (W07);
Validation Guest scenarios (W11); storm evidence (W12); telemetry
collection/latency baseline (W13); machine ABI/DTB (P8); crate/module tree
and public API beyond the stated contracts.
