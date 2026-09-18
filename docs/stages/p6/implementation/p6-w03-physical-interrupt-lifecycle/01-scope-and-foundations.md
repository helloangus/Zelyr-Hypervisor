# P6-W03 Scope, Prerequisites, and Lifecycle Foundations

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W03 detailed design](README.md).

## 1. Prerequisite and assumed-contract inventory

### 1.1 W02 acknowledged Host IRQ boundary

Assumed source:
[P6-W02](../p6-w02-physical-gic-bring-up/README.md) and its readiness
ledger plus initial-state facts. W03 assumes: distributor `Enabled`,
relevant pCPUs `LocalReady`, every SGI/PPI/SPI disabled with lowest default
priority, SPIs determinately routed (IRM=0), EOImode=0 posture, group
enable done, and the register-access surface of
[W02 03](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md)
available as the only hardware-access path. Failure boundary: if W02's
delivered posture differs (e.g. EOImode=1, partial readiness), the W03
completion and dispatch contracts are unsound; the affected work stops and
the mismatch is recorded against the W02 design (Architecture Change
Request if the posture change is real), never absorbed by a local
re-configuration.

### 1.2 P1 exception-entry contract

Assumed source:
[p1-w05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md) via the
task book entry table (P6-ENTRY-05). W03 assumes an EL2 IRQ vector entry
that saves bounded context, classifies origin, and calls a registered
dispatch target with a stable entry-context object; unexpected-vector and
recursive-entry boundaries exist upstream. Failure boundary: without a
stable entry context and a single IRQ dispatch target, W03 has no defined
receipt point; stop and record against the P1 contract.

### 1.3 P3 CPU-local and synchronization contracts

Assumed sources:
[p3-w09](../../../p3/plans/p3-w09-cpu-local-exception-interrupt.md)
(per-CPU exceptional-path identity and non-overlapping exception state),
[p3-w04](../../../p3/plans/p3-w04-per-cpu-runtime.md) (per-CPU storage),
[p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md)
(IRQ-sensitive critical sections, atomic ordering, lock-order baseline).
W03 places per-pCPU counters in per-CPU storage and uses the P3 lock
classes for the registration path. Failure boundary as in §1.1.

### 1.4 Entry gating

W03 implementation begins only after the task book §2 entry review finds
the W02 readiness evidence and the P1/P3 contracts present. W03–W05/W07
ordering per the plans index: W03 may progress when W02 evidence exists;
W04 requires W02+W03.

## 2. Lifecycle boundary

The W03 lifecycle is exactly:

```text
hardware asserts → P1 vector entry (IRQ class)
  → W03 receipt: read IAR (acknowledge)
  → W03 classification: typed ID / special band
  → W03 dispatch outcome:
       consumer present   → invoke consumer (bounded) → complete
       consumerless       → count + diagnose          → complete
       unknown ID         → count + diagnose          → complete
       spurious/no-pending→ count                     → no completion
  → W03 completion: EOI (combined) by the acknowledging pCPU
  → loop until no-pending or per-entry bound → return to P1 exit path
```

Not in the lifecycle: consumer work beyond the callback return; any
cross-CPU completion; any Guest-visible effect; FIQ and SError paths
(P1-owned diagnostics; W03 treats them as out-of-band); Group-0/Secure
interrupts (never configured by P6; receipt would be an unexpected-entry
diagnostic at P1).

## 3. Scope classification detail

**Required:** typed identification; acknowledge/complete loop with one
completion owner; consumer registry (one slot per supported ID);
registration-before-enable path; consumerless/unknown/spurious named
outcomes; per-entry dispatch bound; per-pCPU and per-ID counters;
rate-limited diagnostics; telemetry events; the enable/disable mechanics
consumers use through W03 (SGI/PPI local-register writes; SPI distributor
writes under the W02/W04 global-lock class).

**Reserved (with triggers):** runtime deregistration or consumer
replacement (trigger: an approved hotplug/device design); EOImode=1 + DIR
deactivate split (trigger: [W08](../p6-w08-gic-virtualization-interface/README.md)
presentation); trigger-type configuration surface (trigger: an approved
device/passthrough design; P6 consumers are timer/maintenance/SGI, all
edge-compatible); priority-preemption policy (trigger: P6-W10); Group-0
handling (trigger: an ADR-level secure-world decision — none exists).

**Out of Scope:** SGI target policy and SPI route changes (W04); timer
deadline semantics (W05); virtual interrupts (W07–W09); Guest injection;
production IRQ-DoS resistance (task book P6-V22 explicitly bounds the
claim); device-driver frameworks; scheduler wakeups (P7); a second IRQ
entry path.

## 4. Isolation and safety rules

- **No invalid index or handler:** every registry access takes a
  validated typed ID ([03](03-code-contracts-identification.md) §2); IDs
  outside the enabled supported range never reach a slot index, a handler
  call, or a GICR/GICD register computation. This is a hard boundary:
  guest-caused or hardware-anomaly IDs are data, never control flow.
- **Guest isolation:** no W03 path reads or writes Guest state, injects
  into a Guest, or differs by VM identity; physical dispatch is
  VM-oblivious. (The task book forbids equating a physical IRQ with a
  Guest IRQ; [W07](../p6-w07-virtual-interrupt-core/README.md) owns the
  virtual side.)
- **Completion totality:** every acknowledged assigned ID is completed
  exactly once by the acknowledging pCPU on every path, including
  consumerless/unknown paths and the dispatch-bound exit; the only
  non-completing outcome is the no-pending band. A leaked active
  interrupt would wedge the priority registers — the loop invariants
  ([02](02-architecture-and-state.md) §5) make leakage structurally
  unreachable.
- **Untrusted inputs:** IAR values are hardware-provided untrusted-adjacent
  data (a misbehaving GIC or firmware residue must not control the
  hypervisor); consumer callbacks are trusted EL2 code but must obey the
  bounded-context rules; no Guest/firmware data is dereferenced.

## 5. Relationship to FIQ/SError

P1's exception baseline owns FIQ and SError entry and their diagnostics.
In the P6 posture Group 0 is never configured and FIQ is therefore
unexpected; if the architecture delivers one, P1's unexpected-vector
diagnostic path owns it. W03 documents this boundary and adds no FIQ
handling, so a stray FIQ is diagnosable rather than silently absorbed.
