# P8-W05 Controlled Failure and Diagnostics Boundary

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W05 detailed design](README.md).

## 1. Purpose and authority

This file fixes what happens when a classified operation is rejected: the
Guest-visible failure, the diagnostic record, the containment rules, the
telemetry, and the line between a Guest-caused fault and a hypervisor
invariant failure. Authority: ADR §13 (error-class distinction), ADR §19
("Guest-caused fault 默认只能影响对应 VM/设备上下文，不应触发全局 panic"; cross-CPU
invalidation and isolation invariants), the plan's out-of-scope line
("no unsupported-operation policy beyond controlled diagnostics"), and the
P5 closeout contract as the assumed owner of the error-class machinery
([P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)).

## 2. Reject-path sequence (normative outline)

Every Reject outcome performs, in order:

1. **Record** — construct the `GuestDiagnosticRecord`
   ([02](02-code-contracts-classification.md) §7) from Host-derived state:
   vCPU/VM identity, classification, syndrome, PC/PSTATE, FAR/HPFAR with
   validity flags, exit reason, trace correlation reference, containment
   action.
2. **Inject** — present the Guest-visible failure: an architecturally legal
   fault at the Guest's own exception level, delivered through the
   established exception/virtual-interrupt contracts
   ([P6-W10](../../../p6/plans/p6-w10-interrupt-semantics.md) semantics for
   masking/pending; the P4 exit flow's exception-return path). The form of
   the injected fault per behavior area is part of the per-area inventory
   ([01](01-classification-model.md) §4, per-register detail routed). The
   hypervisor does not invent a non-architectural error channel.
3. **Contain** — apply the containment rule of §3 to the offending vCPU
   (and, when triggered, the VM).
4. **Observe** — emit one structured telemetry event per the P0 trace
   governance assumed contract: classification counters (per area/class)
   plus the event carrying the diagnostic reference. Counters are bounded
   and cheap enough for exit context; high-overhead payloads ride the
   diagnostic record, not the hot path.
5. **Resume control flow** — return the disposition to the exit flow; the
   scheduler observes the new vCPU state per the P7 lifecycle contract.

Failure at any step: steps 1–3 are mandatory before any Guest resume of the
offending vCPU; a failure to complete them is the InvariantViolation path of
§4 — it is never swallowed and never approximated by "resume anyway".

## 3. Containment rules

| Condition | Containment | Basis |
|---|---|---|
| Default: the trapped operation is attributable to the offending vCPU and its state remains architecturally coherent | vCPU-scoped: the offending vCPU stops (P7 Stopped/Faulted run-state family) with its diagnostic record; other vCPUs and the VM continue | ADR §19 default (Guest faults affect the VM/vCPU context); plan scope (controlled diagnostics) |
| The failed operation leaves the VM's shared state incoherent (per the owning domain contract's criteria, e.g., a vGIC or timer handler that cannot restore consistency) | VM-scoped: the VM enters its faulted state per the established VM lifecycle (P4/P7 contracts); all vCPUs stop with per-vCPU diagnostics | Domain contracts own their consistency criteria; P5 failure classes |
| The Guest repeatedly rejects in a tight loop | Same containment as the underlying case plus the loop-detection telemetry of the [P8-W13](../p8-w13-guest-fault-diagnostics/README.md) scope; no automatic hypervisor-side escalation policy is introduced here | Plan out-of-scope line — W13/W18 own storm/loop scenarios |
| Containment itself fails (state cannot be made safe, record cannot be written) | InvariantViolation per §4 | ADR §13 |

Proposed-status note: the per-class vCPU-stop-versus-VM-fault mapping above
is this design's proposal within "controlled diagnostics"; it must be
reconciled with the P5 closeout contract's implemented failure classes
before implementation relies on it. If P5 delivers different classes or a
different escalation mechanism, this mapping is re-derived in the
implementation design delta and the difference is recorded — the default
(Guest faults stay VM-scoped) is ADR-fixed and does not move.

## 4. Guest fault versus hypervisor invariant

The boundary, restated from the ADR and P5 classification:

- **GuestFault-class** — every outcome reachable from Guest-executable
  behavior that the classification anticipated or failed closed on: Reject,
  Unsupported, Hidden-use, missing handler, handler-recoverable error. These
  are recoverable VM-facing errors. Linux is a guest: its bugs, probes, and
  unsupported-feature attempts are expected events with bounded cost, never
  hypervisor faults.
- **InvariantViolation-class** — hypervisor-side failures: containment that
  cannot complete, a diagnostic that cannot be recorded, registry
  corruption, a handler violating its postconditions, or any state
  inconsistency the owning contract defines as architectural. These follow
  the established fatal-diagnostic path (P1 crash-diagnostics facts, P5
  invariant rules). This path is exceptional and narrow; widening it to
  cover Guest behavior would violate ADR §19 and is a design conflict to
  raise, not a shortcut to take.

## 5. Observability requirements

- Per-class counters (Direct needs none on the normal path; Emulate per
  route; Reject/Unsupported per area) — named, bounded, compile-time
  trimmable per the P0 trace rules.
- One structured event per Reject carrying the diagnostic record reference;
  no raw log dumps as a substitute (P6-W13 precedent).
- Diagnostic completeness is testable: every field either valid or
  explicitly flagged unavailable (P8-V08).
- Correlation: the record's trace reference joins boot logs, exit traces,
  and scheduler events per the established telemetry correlation
  assumptions; if the P0/P6 correlation contracts deliver differently, the
  reference field degrades to exit-sequence numbers, and the difference is
  recorded — completeness of the record itself does not degrade.

## 6. Non-responsibilities

This file does not choose injected-fault encodings (per-area detail stays
routed), design W13's diagnosability breadth, define W18's storm/loop
regression, introduce retry/kill/restart policy beyond the containment
table, or modify the P5 error-class machinery. Where an area's injected
fault is a machine fact (routed), the Reject path references it; it does not
define it.
