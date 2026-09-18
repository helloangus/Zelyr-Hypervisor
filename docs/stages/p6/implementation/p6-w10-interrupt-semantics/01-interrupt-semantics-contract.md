# P6-W10 Interrupt Semantics Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W10 detailed design](README.md).  
**Audience:** the semantic authority for P6 Guest-visible interrupt behavior
and the schema of the Interrupt Semantics v0 record (P6-DOC-02). Load before
any workflow step of [02-workflow-and-validation.md](02-workflow-and-validation.md).

## 1. Artifact groups and ownership

| Artifact | Authoritative owner | Content / non-responsibility |
|---|---|---|
| This design (semantic rules §2–§8) | P6-W10 design | the proposed semantic authority until implementation begins; not a factual behavior statement |
| Interrupt Semantics v0 record (P6-DOC-02) at `docs/stages/p6/implementation/p6-interrupt-semantics-v0.md` | W10 record (created when work starts) | implemented Host IRQ/vIRQ lifecycle meanings, pending/active/completed semantics, masking, priority, maintenance behavior, and known limits, as facts with evidence links; never a machine-ABI claim; never a restatement of W07/W08/W09 contracts (it cites them) |
| W07 vIRQ lifecycle contract | `../p6-w07-virtual-interrupt-core/README.md` (P6-W07) | state machine, queue discipline, capacity; cited, not restated |
| W08 presentation contract | `../p6-w08-gic-virtualization-interface/README.md` (P6-W08) | LR presentation, save/restore, what the Guest-visible interface exposes; cited, not restated |
| W09 maintenance contract | `../p6-w09-maintenance-interrupt/README.md` (P6-W09) | completed-state meaning; cited, not restated |
| W06 Guest timer contract | `../p6-w06-guest-generic-timer/README.md` (P6-W06) | timer ownership, deferred expiry, exit/re-entry; cited, not restated |

Terms used below — **pending** (held by the W07 queue, not in an LR),
**presented** (loaded in an LR per W08), **active** (presented and not yet
completed), **completed** (post-maintenance reconciliation per W09) — are
defined by the owning contracts; this file fixes only their Guest-visible
relations.

## 2. Masking semantics

### 2.1 Layers and the binding rule

- **L1 — architectural CPU mask (PSTATE DAIF) at Guest EL1.** While an
  interrupt-class mask bit is set, presented-or-arriving events of that class
  remain pending or active at the virtual CPU interface per the architecture;
  the Guest observes nothing until unmasked.
- **L2 — priority mask via the Guest-visible virtual CPU interface.**
  Applicable **only if** the W08 design presents such a control to the Guest.
  If it does not, L2 is recorded unavailable in the v0 record and no P6
  scenario may depend on it.
- **L3 — hypervisor-side per-vIRQ `enabled` control** in the P6 virtual-IRQ
  namespace. A disabled vIRQ accumulates pending state per §4 and never
  occupies LR capacity; enabling admits it through the normal presentation
  path.

**Rule S-M1 (binding): masking at any layer never implies completion.** A
masked event stays pending (or active, if already presented) with its
lifecycle accounting intact; unmasking makes it observable without a false
completion of any other event. Completion occurs only through the W09
reconciliation of an actually presented and acknowledged event.

### 2.2 Consequences

- C-M1: Host bookkeeping (pending accounting, completion correlation,
  telemetry) is unaffected by any mask state; masking cannot suppress Host
  records.
- C-M2: L3-disabled accumulation is subject to the §4 capacity rules;
  disabling is not a loss mechanism and not an unbounded buffer.
- C-M3: L1/L2 are Guest-controlled and untrusted: they cannot corrupt Host
  state, and a Guest that never unmasks simply never observes the event
  (observable only in its own VM context).

## 3. Priority semantics

- **Band A:** vCPU virtual-timer events (W06-owned). **Band B:** ordinary
  vIRQs (W07-owned).
- Rule S-P1: at a presentation opportunity where both bands have ready
  events, band A is presented first.
- Rule S-P2: the band relation survives pending state — a band-A event
  waiting behind an already-presented band-B event is not reordered ahead of
  it; P6 claims no preemption of an in-progress presentation.
- Rule S-P3: intra-band order is the W07 queue discipline; W10 claims no
  ordering within a band.
- Rule S-P4: numeric priority encoding (field widths, values, register
  interface) is a Specification Investigation resolved at implementation and
  recorded in the v0 record; the band relation is the contract, the encoding
  is not.

## 4. Pending and repeated-event semantics

- **S-PD1 (dedup while pending):** a repeat of an already-pending vIRQ
  collapses; pending state is bounded and does not grow with repeats.
- **S-PD2 (repeat while active):** a repeat arriving while the vIRQ is
  presented adds at most one deferred re-pend, delivered through the normal
  path after completion (W09 reconciliation). This bounds Guest-influenceable
  state to one extra record per active event.
- **S-PD3 (distinct events never merge):** distinct vIRQ identities keep
  independent pending state (P6-V12's "distinct pending events" basis).
- **S-PD4 (capacity):** queue capacity, overflow containment, and
  over-capacity progression are the W07/W09 contracts; under this semantics,
  same-ID overflow collapses (consistent with S-PD1) and distinct-ID overflow
  follows the W07 contract's containment — never silent loss.

## 5. Timer-plus-vIRQ concurrency semantics

- **S-C1:** both event classes progress within one Guest entry; neither
  class starves the other in any single entry.
- **S-C2:** simultaneous readiness at one presentation opportunity resolves
  band-A first (S-P1).
- **S-C3:** no other inter-class ordering is guaranteed or claimed.
- **S-C4:** a timer expiry while the vCPU is absent follows the W06
  deferred-expiry contract; the Guest observes it at or after the next entry,
  never lost, never attributed to another vCPU.
- **S-C5:** semantics across Guest exits (HVC calls, controlled exits,
  re-entry) are preserved per the W06/W08 save/restore contracts; the v0
  record states the implemented evidence for each exit class rather than
  assuming it.

## 6. Guest-untrusted controls

All Guest-requested operations on this semantic surface (enable/disable via
the P5-authorized control path, EOI behavior, mask manipulation) are validated
per the P5 boundary ([P5-W02](../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md),
[P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md))
before any state change. A Guest-caused anomaly (spurious EOI, mask of an
unowned event, disabled-forever event) is a GuestFault-class, VM-local
outcome per the W12 boundary; it can never manifest as Host state corruption
or a cross-VM effect. Rule S-M1 ensures even a hostile mask/EOI sequence
cannot fabricate a completion.

## 7. Scenario semantics for P6-V13–P6-V15 (consumed by W11)

These are the normative expected behaviors the W11 scenarios assert; W11 owns
the Guest-side observation and evidence.

| ID | Scenario semantics | Expected observable | Pass condition | Proves / does not prove |
|---|---|---|---|---|
| SEM-MASK (VG-IRQ-03) | mask (L1, and L3 when available), inject vIRQ, observe nothing, unmask, observe | no event while masked; event observable after unmask; exactly one completion | P6-V13: pending preserved under mask; observable after unmask; no false completion | masking rule S-M1 in QEMU; not L2 (unless W08 presents it), not hardware |
| SEM-PRIO (VG-IRQ-05, part) | make band-A and band-B events ready at the same opportunity | both progress; band A observable first per S-P1, or the documented relation holds | P6-V15: both classes progress; documented priority relation preserved | the band relation; not full GIC priority/preemption coverage |
| SEM-MULTI (VG-IRQ-02) | multiple distinct pending vIRQs | each completes exactly once with determinate outcome | P6-V12 basis holds under S-PD3 | distinct-event independence; not unbounded queue counts |
| SEM-REPEAT (VG-IRQ-04) | repeat injection while pending, then while active | repeats collapse per S-PD1/S-PD2; no state corruption; final count determinate | P6-V14: documented policy followed, no corruption | the bounded-accumulation policy; not that the policy is optimal or hardware-conformant beyond the spec |

## 8. Explicit non-freeze and exclusion statement

Reproduced for downstream readers so no P6 semantic can be mistaken for a
machine contract: nothing in §2–§5 freezes the P8 rusthv-arm-virt-v1 machine
ABI, a Guest DTB interrupt layout, a vGIC Distributor/Redistributor MMIO
semantic, an interrupt-controller configuration value, or a scheduler
policy. P6 semantics describe the implemented behavior of the P6 mechanism
layer in the stated environment; P8 defines what Linux guests see, and P7
defines run-state and wakeup policy. This statement is restated in the v0
record and in W13's consumer review.
