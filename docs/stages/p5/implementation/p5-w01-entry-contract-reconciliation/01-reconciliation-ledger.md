# P5-W01 Reconciliation Ledger

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W01 detailed design](README.md).

## 1. Ledger conventions

Every reconciled input is one row in a fact-domain table (§2). The row schema
is fixed so that the review output is mechanical to check:

```text
Fact:        the single upstream statement P5 consumes, phrased as P5 may rely on it
Source:      the governing upstream plan or ADR path (cited by path, never by name inference)
Consumer:    the P5-Wxx package(s) that rely on the fact
Evidence:    where proof lives today, or the explicit record of its absence
Failure:     what the consumer must do if the implemented upstream delivers differently
```

Rules:

- `Evidence` names tracked paths only. In the current scaffold the honest
  value for most rows is: plan text at the cited path; implementation and
  verification records absent as of the review date. Absence is recorded,
  never averaged away.
- `Failure` uses exactly one of the §5 labels. A row whose failure label is
  `Contract Conflict` or `ADR Required` stops the affected consumer decision;
  it never licenses a local reinterpretation of the upstream contract.
- No row may phrase an upstream fact more strongly than its source plan does.
  Where a source plan states a boundary ("does not prove...", "remains
  unimplemented"), the row carries that boundary forward verbatim in
  substance.

## 2. Fact-domain ledgers

### FD-1 — Execution and trap boundary (consumed by W02, W06)

| Fact | Source | Consumer | Evidence | Failure |
|---|---|---|---|---|
| A stable AArch64 Non-secure EL2 execution state exists with synchronous-exception entry paths for all exception categories, syndrome and location capture, and a recoverable-versus-fatal boundary | [P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md); [P1 task book](../../../p1/task-book-v0.1.md) | W02 (trap recognition), W06 (containment) | Plan text; P1 implementation/verification records absent | `Blocked Prerequisite` — W02/W06 cannot integrate dispatch without the exception path; stop and record |
| The synchronous-exception path classifies by exception class and can distinguish an HVC-class trap from other synchronous exceptions, with the guest register frame capturable at trap time | [P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md) (classification boundary); [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md) (exit categorization) | W02 (frame capture and decode) | Plan text; no HVC-specific handling is promised by P1/P4 — P5 adds HVC dispatch as a *new consumer* of the classified path | If the implemented P1/P4 path routes unknown synchronous exceptions to a fatal/guest-stop terminal without an extension seam, W02 integration is a `Contract Conflict` to record against P4-W06 — not a P5-local fork of the vector path |
| A Guest-EL1 vCPU can be entered by `ERET`, exit to a live diagnosable EL2, re-enter, and stop, with EL2 control retained | [P4-W04](../../../p4/plans/p4-w04-vcpu-entry-exit.md) | W02, W06, W07 (guest-side scenarios) | Plan text; P4 records absent | `Blocked Prerequisite` — no guest execution context means no caller exists; stop and record |
| Guest-caused faults are classified, diagnosed with Guest/vCPU, PC, IPA, access, and state context, and remain VM-facing rather than Hypervisor panics | [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md); [P4 task book](../../../p4/task-book-v0.1.md) §7 | W02 (guest-fault outcome class), W06 (containment) | Plan text; records absent | `Contract Conflict` if the implemented boundary converts controlled Guest faults into Hypervisor fatal events — record against P4-W06; P5 must not weaken its own containment to match |
| P4's exit/fault classification is a *factual* capability record, not a frozen `ExitReason` API | [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md) (out-of-scope: final `ExitReason` API); [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) | W02 (must design its own result classification, not inherit one) | Plan text | `Documentation Gap` if P4's implemented record freezes an exit API P5 would inherit — then the ABI-relation question is recorded, not absorbed |

### FD-2 — Guest-data and memory boundary (consumed by W03, W06)

| Fact | Source | Consumer | Evidence | Failure |
|---|---|---|---|---|
| A normalized Host physical-memory map exists in which every P2-protected range (Hypervisor image, DTB, firmware, boot artifacts, allocator metadata) is identified before dynamic allocation, with checked conflict handling | [P2-W03](../../../p2/plans/p2-w03-boot-memory-map-ownership.md) | W03 (forbidden-boundary treatment) | Plan text; records absent | `Blocked Prerequisite` — guest-data validation cannot bound Host exposure without the protected map |
| A physical-page allocation/release capability exists that never returns a protected range, with explicit OOM and ownership accounting; no allocator algorithm is handed over | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md); [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) | W03 (access backing assumptions), W04 (table backing option) | Plan text; records absent | `Blocked Prerequisite` for any P5 component that needs allocation; a static-backing fallback inside P5 would be a design change to record, not a silent substitute |
| Stage-2 capability boundary: one Guest's address space can be created/destroyed, mapped, unmapped, permission-protected, queried, installed, and made current-path consistent | [P4-W02](../../../p4/plans/p4-w02-stage2-address-space.md) | W03 (translate/query basis) | Plan text; records absent. P4-W02 explicitly does **not** deliver cross-pCPU shootdown | `Blocked Prerequisite` if translate/query is absent; `Contract Conflict` if a P5 design needs cross-CPU concurrent-mutation safety that P4 did not prove — W03 must scope v0 access against non-concurrent mutation instead of assuming shootdown |
| Stage-2 translation/permission faults are diagnosable with IPA and access context, and Hypervisor-owned ranges are blocked by Stage-2 | [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md); [P4 task book](../../../p4/task-book-v0.1.md) P4-V07 | W03 (fault-cause vocabulary for guest-data failures) | Plan text; records absent | `Documentation Gap` if the implemented diagnostic vocabulary cannot express the causes W03's contract needs; classify and request bounded clarification |
| Bounded Guest RAM originates only from allocatable pages; temporary Validation Guest IPA-layout facts are a P4 test contract, never a machine ABI | [P4-W03](../../../p4/plans/p4-w03-guest-memory-image.md); [P4 task book](../../../p4/task-book-v0.1.md) Reserved | W03 (range legality), W07 (scenario setup) | Plan text; records absent | `Contract Conflict` if any P5 artifact treats temporary IPA values as contractual — record against the offending P5 design |
| P2-ACR-01 is unresolved upstream and constrains memory-object planning | [P2-W03](../../../p2/plans/p2-w03-boot-memory-map-ownership.md) (visible, unresolved); [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) (preserved) | W03 (must not assume its resolution) | Recorded in P2 plan text | Kept visible per README decision 7; no P5 package may resolve or route around it |

### FD-3 — Object and lifecycle foundation (consumed by W04, W05, W06, W07)

| Fact | Source | Consumer | Evidence | Failure |
|---|---|---|---|---|
| P4 delivers a single-Guest Stage-2 and EL1 execution foundation with minimal VM/vCPU objects; formal HVC, capability, and management behavior remain P5 work | [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) | W04 (object-reference targets), W05 (authority subjects) | Plan text; records absent | `Blocked Prerequisite` if no addressable VM/vCPU objects exist for the object table to reference; P5 must not invent VM/vCPU lifecycle (P4 scope) — stop and record |
| P4 lifecycle facts include a defined create → run → stop → reinitialize → run-again path that does not rely on prior Guest RAM or CPU residue | [P4 task book](../../../p4/task-book-v0.1.md) P4-V10; [P4-W07](../../../p4/plans/p4-w07-repeatability-telemetry.md) | W04 (destroy/reuse semantics have a real upstream lifecycle to attach to) | Plan text; records absent | `Documentation Gap` if the implemented lifecycle lacks an observable stop/cleanup boundary W04's destroy cases can hook |
| The Rust `no_std` Validation Guest is a maintained test asset with an explicit scenario set (VG-001–VG-012), and its temporary conventions are not a Guest machine ABI | [P4-W05](../../../p4/plans/p4-w05-validation-guest.md); [P4 task book](../../../p4/task-book-v0.1.md) | W07 (extends the asset; two-context setup) | Plan text; records absent | `Blocked Prerequisite` for W07's guest-side evidence; W02–W06 are not blocked (they are host-designable boundaries) |
| P5 may rely only on *proven* P4 facts; P5 must not re-establish EL2→EL1 entry or basic Stage-2 fault capture | [P4 task book](../../../p4/task-book-v0.1.md) §7 | all P5 packages | Task-book rule | Any P5 work re-proving P4 behavior is scope violation; record as `Documentation Gap` in the offending design review |

### FD-4 — Authority and security boundary (consumed by W02–W06, W10)

| Fact | Source | Consumer | Evidence | Failure |
|---|---|---|---|---|
| Guest register values, hypercall buffers, MMIO, descriptors, and management input are untrusted and must be validated before use | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) ADR-007, §12, §19 | all | Accepted ADR (tracked) | Accepted architecture; deviations are `ADR Required` by definition |
| Authorization is capability/handle + rights + generation; roles, fixed VM IDs, first-VM status, and Control Domain identity are not authority; capability-check failure must not degrade to role/VM-ID pass-through | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) ADR-013, ADR-051, §19 | W04 (identity), W05 (authority), W06 (ordering) | Accepted ADR (tracked) | `ADR Required` for any contrary choice |
| No Host pointer reaches a Guest through a hypercall; guest-supplied addresses are checked before any memory access | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) §19; task book INV-P5-01/02 | W02 (result registers), W03 (access), W09 (log redaction) | Task book §6 invariant list; wording of the factual security deliverable is a post-implementation artifact (W10) | Accepted invariant; violations are `ADR Required` plus a security review |
| A writable physical page is not shared between mutually untrusting VMs without an explicit SharedRegion; all object handles use generation against stale references; checked arithmetic on lengths/offsets | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) §12, §19 | W03, W04 | Accepted ADR (tracked) | `ADR Required` for deviation |
| ADR-034 reserves Endpoint/Notification/SharedRegion as the native primitive set; ADR-036/ADR-040 keep the versioned native management ABI and machine ABI separate from each other and from this boundary | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) ADR-034/036/040; task book §1 Out of scope | W02 (must not freeze a management ABI), W10 (closeout wording) | Accepted ADR (tracked) | `ADR Required` if a P5 artifact claims management-ABI or machine-ABI status |
| ADR-056 (native management ABI encoding) is **pending**; candidates remain open | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) ADR-056; task book §8 | W02 (must not pre-empt it), W10 | Accepted ADR, decision pending | Kept visible; W02's HVC v0 encoding choices are scoped to the Guest service boundary and must record that they do not settle ADR-056 |
| The P5 permanent invariant set INV-P5-01 … INV-P5-10 exists by ID; its concrete wording belongs to the factual security deliverable after implementation | [P5 task book](../../task-book-v0.1.md) §6 | W02–W06 (preserve in design), W10 (publish wording) | Task book text | `Documentation Gap` if a P5 design cannot state which invariant IDs its contracts preserve |
| Bootstrap: a Hypervisor-created bootstrap context grants initial authority; P5's grant source is the Hypervisor/test bootstrap, not a Control Domain | ADR-038 (future Control Domain creation); [P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md) ("Hypervisor/test bootstrap"); task book P5-V07 | W05 (grant authority), W07 (test setup) | ADR + plan text | `Contract Conflict` if any P5 artifact substitutes a Control Domain, role, or VM ID for the explicit grant |

### FD-5 — Engineering and evidence boundary (consumed by all, especially W07–W10)

| Fact | Source | Consumer | Evidence | Failure |
|---|---|---|---|---|
| Pinned Rust toolchain baseline with declared components and update rules | [P0-W02 design](../../../p0/implementation/p0-w02-rust-toolchain-baseline/README.md) | all (build/test reproducibility) | Proposed design tracked; P0-W01 repository-baseline verification record tracked; toolchain implementation not evidenced | `Blocked Prerequisite` at implementation entry if the pin is not in effect |
| Host-side unit/integration test gates and a QEMU automation entry point exist as governed capabilities | [P0-W07](../../../p0/plans/p0-w07-development-quality-gates.md), [P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md), [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) | W02–W05 (host-testable seams), W08 (fuzz/stress), W09 (regression) | Plans; records absent | `Blocked Prerequisite` for host-side evidence; QEMU-side evidence for guest scenarios likewise |
| `unsafe` governance with SAFETY-comment discipline and an inventory; new `unsafe` is reported per change | [P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md) | W02–W06 (frame capture and guest access will need audited `unsafe`) | Plan | `Documentation Gap` if a design introduces `unsafe` without mapping it to the inventory process |
| Logging/diagnostic levels, trace-event namespace, and a panic policy distinguishing fatal Hypervisor invariants from Guest-caused faults | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md), [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md) | W02 (invariant-failure path), W06 (containment), W09 (telemetry) | Plans; records absent | `Blocked Prerequisite` for the invariant-failure classification W02 depends on |
| Synchronization semantics: mutual exclusion, IRQ-sensitive sections, atomic state transfer, lock-order baseline for allocator/registry/statistics consumers | [P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md) | W04 (table locking), W05 (grant/revoke races), W06 | Plan; records absent | `Blocked Prerequisite` for any shared-state P5 mechanism at implementation entry |
| Host SMP foundation: logical pCPU identity, CPU-local state, cross-CPU notification/completion, future TLB transport, CPU-attributed faults, audit/telemetry rules; no permanent caller↔pCPU binding may be assumed | [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md); [P5 task book](../../task-book-v0.1.md) §1 Reserved | W02 (caller identity), W04/W05 (concurrency scope), W08 (two-pCPU scenarios) | Plan; records absent | `Contract Conflict` if a P5 design binds authority or identity to a pCPU — record against the offending design |

## 3. Assumed-contract model for P5 designs

Every P5 package design (W02–W10) states its upstream reliance in this form:

```text
Assumed contract: <one-sentence fact P5 relies on>
Source:           <upstream plan/ADR path>
Failure boundary: <Blocked Prerequisite | Contract Conflict | ADR Required |
                   Documentation Gap> + the concrete stop/record action
```

Rules binding all P5 designs:

1. A design may *consume* an assumed contract; it may never *strengthen*,
   reinterpret, or quietly re-scope it. If the design needs more than the
   source grants, that need is a recorded gap or conflict, routed to the
   owning stage.
2. A design must not infer crates, module trees, APIs, targets, file layouts,
   or behavior from directory or plan names (repository `AGENTS.md`). The
   only valid citation is the source document path.
3. Sibling P5 dependencies (for example W05 → W02 error routing, W05 → W04
   identity, W06 → W03/W04/W05 composition) use the same model: cite the
   sibling design slug, state the failure boundary if the sibling design is
   revised incompatibly. Cross-sibling incompatibility is a `Contract
   Conflict` between the two designs, resolved at design review, not by
   either design silently adapting.
4. Each design's validation matrix distinguishes evidence it can produce at
   its own level (design review, host-side test) from evidence that requires
   an upstream prerequisite (QEMU guest scenario, hardware), and marks the
   latter blocked until the prerequisite is evidenced.

## 4. Artifact routing table

| Future artifact | Governed location | Producer | Publication precondition |
|---|---|---|---|
| Factual HVC ABI v0 document (register conventions, call numbers, error values, compatibility analysis) | `docs/abi/` per [`abi/README.md`](../../../../abi/README.md); exact file name chosen by the producer and indexed at closeout | W02 owns content decisions; W10 owns factual publication and index links | Approved detailed design + implemented boundary + compatibility analysis + linked evidence |
| P5 factual security deliverable (INV-P5-01…10 wording, threat notes, containment record) | `docs/security/` per [`security/README.md`](../../../../security/README.md) | W10, with inputs from W02–W06 and W07–W09 evidence | Implementation and validation evidence exists; no security claim from design text |
| Approved detailed designs (this set and W06–W10) | `docs/stages/p5/implementation/<slug>/` per [`../README.md`](../README.md) | each package | Design review acceptance |
| Implementation records (decisions taken, deviations, changed artifacts) | `docs/stages/p5/implementation/<slug>-record.md` (created when work starts) | each package | Work has started; no completion claim inside |
| Verification records (commands, environments, results, run/not-run) | `docs/stages/p5/verification/<slug>-verification.md` (created when evidence exists) | each package | Evidence actually produced |
| Entry-review record (this package's output) | `../p5-w01-entry-contract-reconciliation-record.md` | W01 | Review performed per [workflow](02-workflow-validation-handoff.md) |
| Stage index status rows | [`../README.md`](../README.md) (stage process) | stage integration process at record-acceptance time | Records exist and are linked from the entry-review and closeout records so the index update is mechanical |

Routing rule: no P5 design may place factual ABI or security *content* in
design text as if it were the governed artifact; designs propose values, the
governed document records them as implemented facts after the precondition is
met.

## 5. Gap classification and inherited open items

### 5.1 Labels and required handling

| Label | Meaning | Required handling |
|---|---|---|
| `Blocked Prerequisite` | The input is absent, or present only as plan text without the evidence the consumer needs at implementation entry | Stop the consuming package's affected step; record in the entry review and the consumer's record; resume only when the prerequisite is evidenced. No local workaround. |
| `Contract Conflict` | Two governing statements contradict | Record an `Architecture Change Request` issue naming both sources and the affected decision; stop that decision; owners of both sources resolve. P5 never picks a side silently. |
| `ADR Required` | Resolution would alter an accepted ADR decision or invariant | Stop the affected decision; open the ADR process per the baseline's change rules (superseding ADR, never silent edit). |
| `Documentation Gap` | The input exists but under-specifies what its P5 consumer needs | Request a bounded clarification from the owning stage's record; the consumer may proceed only with the explicit recorded assumption, repeated in its design's failure boundary. |

### 5.2 Inherited open items register (visible, unresolved by P5)

| Item | Source | Effect on P5 | Required P5 handling |
|---|---|---|---|
| P2-ACR-01 | [P2-W03](../../../p2/plans/p2-w03-boot-memory-map-ownership.md) (unresolved; blocks P4 memory-object definition), [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) (preserved) | FD-2 consumers (W03) depend on the memory-ownership foundation the item may affect | Keep visible in W03's assumed contracts; W03 must state its failure boundary without assuming resolution; resolution stays with P2 owners |
| ADR-056 pending (native management ABI encoding) | [ADR-000](../../../../adr/adr-000-architecture-baseline-v0.1.md) ADR-056; [P5 task book](../../task-book-v0.1.md) §8 | W02's HVC v0 encoding decisions touch the same topic space | W02 scopes its choices to the Guest service boundary v0 and records that they do not settle ADR-056; convergence of HVC v0 with a future management ABI is Reserved |
| Caller↔pCPU placement evolution | [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md); [P5 task book](../../task-book-v0.1.md) §1 Reserved | Caller identity must not embed pCPU placement | Fixed by README decision 5 (caller = VM execution context); W02/W04/W05/W08 must not key identity or authority to a pCPU |
| P4 architecture-deviation or ACR records | Unknown until P4 records exist | FD-1/FD-2/FD-3 evidence rows cannot be finalized | Evidence-level reconciliation re-runs at implementation entry; any P4 deviation that weakens a P5 input becomes `Contract Conflict` or `ADR Required` at that point |
| Scheduler policy (P7) and interrupt objects (P6) | [P5 task book](../../task-book-v0.1.md) §1 Reserved, §7 | Later stages may change vCPU execution context placement and add object classes | W04 designs the object class mechanism as extensible; W05 designs rights as operation-level; neither may pre-design P6/P7 semantics |

## 6. Consumer assignment summary

| P5 package | Primary fact domains | Additional entry inputs |
|---|---|---|
| W02 | FD-1, FD-4 | FD-5 (host-test, panic-classification, unsafe, documentation governance); routing §4 |
| W03 | FD-2 | FD-4 (no-host-pointer, checked arithmetic); FD-5 (checked-address conventions from P0-W15 vocabulary) |
| W04 | FD-3, FD-4 | FD-2 (allocation backing option); FD-5 (P3-W06 locking discipline) |
| W05 | FD-4 | W02 error routing and W04 identity as sibling assumed contracts; FD-3 subjects |
| W06 | FD-1–FD-4 (composition) | P4 exception boundary; sibling contracts W02/W03/W04/W05 |
| W07 | FD-3 (Validation Guest asset), FD-4 | FD-5 (QEMU governance); sibling expected-outcome contracts |
| W08 | FD-5 | P0 host-test gates, P3 SMP inputs; sibling seams (W03–W06) |
| W09 | FD-5 | FD-4 (redaction); W07 markers, W08 limits; P0 diagnostics/QEMU governance |
| W10 | all (closeout) | Routing §4 and open-items §5.2 registers; all package records |
