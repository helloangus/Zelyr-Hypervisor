# P3-W10 Audit Framework and Checklist

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W10 detailed design](README.md).

This file is the audit's normative framework: what is in scope (§2), how
each classification is earned (§3), what every record contains (§4), how
findings are routed (§5), and what blocks closure (§6). It contains no
audit results; the executed record is produced under
`../p3-w10-smp-safety-audit-record.md` when the workflow in
[02](02-workflow-validation-and-handoff.md) runs.

## 1. Audit statement (what "audited" means)

An item is *audited* when a reviewer has: identified the state's single
owner, enumerated its access contexts (which CPUs, thread/exception
context, before/after `SmpReady`), classified it under §3 with the
required citations, and linked the evidence. "Audited" never means
"proven race-free": proofs of absence of races are out of scope per the
plan; the audit establishes that every mutable state has a *declared,
standard-cited* synchronization story.

## 2. Artifact groups (authoritative scope)

| Group | Covers | Item seeds (plans/designs; seeds do not preempt the executed audit) |
|---|---|---|
| G1 Logging/console/diagnostics | console emission paths, log buffers, log-level state, early-console state | P0-W12 contract surfaces; P1-W06/P1-W07 plans; W09's Diagnostics-lock rules |
| G2 Physical page allocation | page-frame allocator state (bitmaps/free lists), ownership debug metadata | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md) |
| G3 Heap/object (small) allocation | slab/small-object allocator state, heap wrappers | [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md) |
| G4 Platform information | PlatformInfo, PlatformCapabilities, discovery outputs, topology inputs | P2-W01/W02/W03 plans; [P3-W01](../p3-w01-cpu-topology-inputs/README.md) |
| G5 Global registries | every boot-global registry/table/phase structure | CpuRegistry (W03), W04 lookup table + `PerCpuSet`, W05 `BootPhase`/ready flags/`SmpReadyState`, W02 arrival mailboxes/outcome map, W07/W08 slot publications |
| G6 Diagnostic/fatal state | per-CPU exception/fatal state and any shared diagnostic buffers | W09 slot contents; P1-W07 surfaces |
| G7 Other mutable infrastructure (catch-all) | anything writable not covered above, P0–P2 or P3 | discovery protocol §2.7 |

### 2.7 G7 discovery protocol (normative)

Sweep every tracked source module (per the workspace the approved build
design defines) for: `static` items, `Atomic*` fields, lock instances,
`UnsafeCell`/`static mut`, and global buffers. Each hit becomes a G7
candidate item; a candidate is either classified (§3), merged into an
existing G1–G6 item, or recorded as an open finding. The sweep's
completeness evidence is the module list reviewed, recorded in the audit
record. Unsafe inventory entries (P0-W10 governance) are cross-read as a
secondary discovery source, not a substitute.

## 3. Classification taxonomy and decision rules (normative)

Exactly five classes (plan-fixed). A state that satisfies none is an
`Unclassified` blocker and a design conflict to raise — never a sixth
class.

| Class | Entry rules (all must hold; citations required) | Required citation |
|---|---|---|
| Immutable-after-boot | Written only during construction; exactly one publication point; no code path writes afterwards (checked by review of all writers) | the publication point (design/file reference) |
| CPU-local | Exactly one writer CPU; other-CPU access limited to documented read-only diagnostic surfaces; storage per-CPU (W04 area or equivalent) | the owning design's privacy guarantee (e.g., W04 area model) |
| Atomic | Lock-free; every access ordering justified | W06 AP pattern id ([P3-W06](../p3-w06-concurrency-synchronization/README.md) §3); for multi-variable cases, the AP-4 justification |
| Lock-protected | Every access under one lock instance with a stated class | W06 ladder class/rank (LOL) + the lock instance's construction site; flavor (plain/irq-save) justified per CR-2 |
| Boot-only | Every write occurs in a single-threaded pre-publication region or is gated by the boot phase authority; read-after-publication only | W05 `BootPhase` gate (`GlobalInitPublished`/`SmpReady`) or the named single-threaded region |

Decision aids:

- A state with mixed behavior (e.g., boot-writes then CPU-local) is
  classified by its *most demanding phase* and the record names both
  phases.
- A state whose write set cannot be enumerated is `Unclassified`
  (insufficient ownership clarity) — a finding, not a judgment call.
- `Pending-audit` is a workflow state ([02](02-workflow-validation-and-handoff.md)
  step 3), never a final classification.

## 4. Per-item record schema (normative)

```text
Item ID:        W10-<group>-<nn>          (stable within the record)
Group:          G1..G7
Name:           what the state is
Owner:          the owning package/design (one owner; none = finding)
Source refs:    plans/designs/records/paths where the state is defined
Access contexts: CPUs (all/one); thread/exception; pre/post SmpReady
Classification: one of the five classes (or Unclassified/Blocked)
Citations:      standards required by §3 (AP/LOL ids, phase gate,
                privacy guarantee) — empty citations = unclassified
Evidence:       where the §3 rules were verified (review notes, record
                or verification paths)
Remediation:    needed? owner? design-change required? (none if clean)
Blocker:        no | yes + reason (see §6)
```

The record document also carries: the prerequisite-gate outcome
([02](02-workflow-validation-and-handoff.md) step 1), the G7 sweep module
list, the blocker ledger (§6), and the integration map outcomes (§7).
Schema changes are W10 design changes, not auditor convenience.

## 5. Remediation routing rules (normative)

- RM-1: Every finding names exactly one owner package/design; "shared"
  or "TBD" ownership is itself a finding (ownership clarity is a
  classification precondition).
- RM-2: A remediation inside the owner's existing approved design
  boundary is routed as a finding to that owner (they execute it through
  their own workflow).
- RM-3: A remediation that would alter a frozen contract, an ADR
  constraint, crate layering, or a P0–P2 deliverable is labeled
  `Architecture Change Request` (or `ADR Required` for ADR-level
  changes) with the owner named — W10 does not implement it.
- RM-4: Unsafe-related findings are cross-referenced to the P0-W10
  inventory and its owner; W10 records the linkage, not the fix.
- RM-5: Remediation status is tracked in the blocker ledger until the
  owner closes it; closure evidence lives in the owner's verification
  material, linked from the ledger.

## 6. Blocker rules (normative — P3-V10's teeth)

- BL-1: An item left `Unclassified` at record finalization is a closure
  blocker.
- BL-2: An item `Blocked` by missing predecessor evidence (no P0–P2
  implementation/verification record to audit against) is a closure
  blocker naming the missing evidence.
- BL-3: An open remediation that changes runtime behavior relevant to
  SMP safety is a closure blocker until the owner's evidence exists.
- BL-4: The carried-over P2-ACR-01 item (from the
  [P2 task book](../../../p2/task-book-v0.1.md)) stays on the ledger
  until authorized resolution — visible regardless of classification
  outcomes.
- BL-5: The blocker ledger is consumed by W15 (stage closure) and
  reported into the P4 handoff; "no blockers" must itself be evidenced
  by the completed item list.

## 7. Integration map (consumers of the executed record)

| Consumer | Consumes | Rule point |
|---|---|---|
| [P3-W06](../p3-w06-concurrency-synchronization/README.md) (already delivered) | is *consumed by* the audit as the atomic/lock standard | §3 |
| W09 | its non-overlap enumeration result is an audit input for G6 | G6; [W09 03 §6](../p3-w09-cpu-local-exception-interrupt/03-code-contracts-exception-local-state.md) |
| W11 | classified telemetry/diagnostic items; ledger | G1/G6 |
| W12 | access-context list as stress targets | §4 schema |
| W13 | blocker status as matrix-scope context | §6 |
| W15 | blocker ledger as mandatory closure input | §6 |
| W14/P4 | completed record + "P4 audits new state independently" statement | entry README handoff |
