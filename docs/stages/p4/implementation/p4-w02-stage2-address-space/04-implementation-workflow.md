# P4-W02 Implementation Workflow

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.

## 1. Preconditions and failure boundary

Read the current source, governing task book, selected plan, Coding Guidelines
and W12/W11 designs. W04 is unimplemented; neither allocation nor Host access is currently delivered. Formal ADR integration and producer evidence gate production
integration; Host fakes may validate pure models only. Do not create raw HPA
imports, public completion booleans or an unscoped Stage2Released workaround.

## 2. Ordered implementation steps

1. Implement checked vocabulary/storage and pure descriptor/layout validation.
   Verify overflow, unsupported profiles and bounded capacity without side effects.
2. Bind actual W04 handles to W12 objects; bind W11 views/pins and Guest region
   leases. Acceptance: unique backing, exact usable prefix and no free while any
   region is retained. Failure before publication returns owned inputs explicitly.
3. Implement the state machine and transaction journal. Inject every partial
   publication/write failure; acceptance is completed rollback or explicit retained
   resources, never blanket unchanged-on-error or automatic Drop cleanup.
4. Integrate architecture publication, code visibility and retirement. Audit each
   unsafe operation against ownership, scope, alignment, aliases and CPU coverage.
   No guard survives callbacks or transport waits. Verify target profile and final
   table/journal/image capacity before hardware exposure.
5. Integrate W03 load-before-Guest-map and W04 execution leases/detachment. Base
   console has separate Device authority; no RAM adoption/free. Re-init retires
   old Guest views before obtaining a new Host writer.
6. Execute the [validation matrix](05-validation-and-handoff.md), separating
   pure Host results from reference-QEMU behavior. Feed W07/W08 and W09 factual
   evidence. W10 has a separate multiple-space/cross-CPU acceptance matrix.
7. Review changed files, new unsafe and ABI/dependency changes; retain every open
   gate and failed resource identity. Never mark a stage complete from design alone.

## 3. Evidence destinations

Implementation facts: `../p4-w02-stage2-address-space-record.md`.
Validation: `../../verification/p4-w02-stage2-address-space-verification.md`.
Each row records command/review input, exact revision, configuration, observed
result, not-run scope and retention/error classification. These future paths do
not imply existing evidence.
