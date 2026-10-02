# Documentation audit — current findings and follow-up

Chinese readers can use the [Chinese edition](README.zh-CN.md).

**Status:** Informative consolidated audit record; audit coverage is closed with
findings, while stage implementation/evidence gates remain open.
**Scope:** P0–P8 documentation audit and its recorded remediation, not stage completion.
**Version:** v0.2
**Owner/change context:** Project owner requested design approval and intermediate-document cleanup, 2026-10-02.
**Supersedes:** The live development-directory audit handoff, phase 1–7 reports
and append-only consolidated report. The owner explicitly requested deletion
of intermediate records without an archive.

## Current findings

This is the single current audit summary. Stage designs/records remain the
sources for their contracts and runtime evidence. The
[memory/Stage-2 design approval](design-approval.md#owner-approval)
records the owner's explicit 2026-10-02 design approval; it does not manufacture
implementation, hardware or stage-completion evidence.

| Finding / owner | Current disposition | Remaining work |
|---|---|---|
| AUD-001 — P7 wakeup | Owner selected W06 shared atomic phase/event handshake | Implement and prove W06-DV04/P7-V14/V25; separately close W08 idle check/wait and deadline-fold progress gaps |
| AUD-002 — P3 transport lifetime | W08-SYNC A selected; no Pending replacement, strong-CAS admission, no held data guard during collection, checked release and terminal retention specified | W06/W08 design admission outside the approved memory/Stage-2 set, implementation and DV03/DV05/DV06 evidence |
| AUD-003 — Host mapping / P2 | W12 v0.1 and W11 v0.2 approved; sole backing owner and retained mapping views specified | ADR-062 lifecycle, W04-MAP, final architecture/fit review, W12/W11 implementation and consumer-specific Host/SMP evidence |
| AUD-004 — Stage-2 / P4–P7 | W02/W03 v0.2, W10 v0.1 and P7-W02 abort companion v0.1 approved; ownership, installed/resident context, retirement and safe abort specified | S2-INSTALL/RETIRE production paths, P7 cleanup/abort implementation and V17–V22 evidence; no multi-VM runtime admission yet |
| AUD-005 — P6 SGI units | Mixed-unit comparison removed; call/write/target-attempt/ack/completion units and controlled/coalescing cases specified | Implementation and P6-V04/V05-related evidence; no inferred coalescing counts |
| AUD-006 — P8 authority homes | Proposed-design location conflict resolved: W02/W03 machine contracts under machine-types, W14 compatibility under abi, W20 links them | Actual approved public contracts, machine values and owning-stage evidence |
| AUD-007 / DOC-MECH-01 / DOC-META-01 | Navigation, stale anchors and current task-book version headers corrected | Maintain checks as documents change; no runtime claim |
| AUTH-01 — P0-W01 approval provenance | Owner confirmed no separate historical approval record; false approval claim removed | Preserve P0-W01 Proposed status and its separately scoped completion evidence; this turn does not approve it retroactively |
| P6 physical IRQ / Guest timer completion | Combined EOIR and deferred physical completion contracts remain a producer/consumer issue | Settle completion API/order, including orphan EOI, and validate the real interrupt path |
| P5 authorize-then-use lifetime | Revocation/destruction between check and use remains a recorded contract issue | Stable object/grant retention or atomic use/revocation protocol and validation |
| P1 NC6 / P6-V29 | Genuine asynchronous unexpected-vector execution remains unproven | P6 owns execution and retained evidence; P1 reference-QEMU completion stays bounded |
| Other stage-owned findings | P6 W1C/timer/LR; P7 lifecycle/serialization; P4 permission/re-entry; P8 timer/fault/fixture issues were recorded in the historical detailed review | Owning-stage source documents retain these issues; this cleanup does not resolve them |

Additional admissions remain explicit in their owning records: P2 W05/W07/W09,
P8-W02 machine values/P8-V19, ADR-057 RoundRobin v0, and prerequisite contracts.
P8 future P9–P21 mechanisms, DMA/IOMMU, migration, hardware and broad security/
performance guarantees are not made current-stage obligations by this audit.

## Decisions and implementation boundary

- On 2026-10-02 the owner explicitly excluded P2-W04 from this submission and
  directed that it be treated as not completed and redone later. Its local code,
  new package design amendments and delivery/verification records are excluded.
  W04 is an unavailable prerequisite; no prior local test result grants admission.
- ADR-062 option A assigns common backing/view foundations to P2-W12 and Guest
  adapters to P4; its formal ADR status remains recorded in
  [ADR-062](../../adr/adr-062-p2-minimal-memory-object-foundation.md).
- The approved [W11](../../stages/p2/implementation/p2-w11-host-allocated-frame-mapping/README.md),
  [W12](../../stages/p2/implementation/p2-w12-minimal-memory-objects/README.md),
  [P4-W02](../../stages/p4/implementation/p4-w02-stage2-address-space/README.md),
  [P4-W03](../../stages/p4/implementation/p4-w03-guest-memory-image/README.md),
  [P4-W10](../../stages/p4/implementation/p4-w10-multivm-stage2-handoff/README.md)
  and [P7 abort](../../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md)
  are implementation-design authority within their stated gates. Approval is
  recorded locally as instructed; no PR merge is claimed or required for this record.

Implementation proceeds through producer prerequisites, consumer integration and
actual validation. No unresolved admission is silently waived by approval.
Other open questions include online branch-protection state, off-host custody of
local runtime artifacts and exact future machine values; no fresh external or
runtime investigation was performed during this cleanup.

## Historical records

The original audit used snapshot `085dae0cf49fca69ce269afeecd6078582a6189b`:
967 full-text inputs and 125 historical plans. Later supplements brought plans
to 128. The prior handoff census recorded 217 cross-stage relations: 17 mirrored,
60 producer-only, 106 consumer-only and 34 assertion-only; 117 lacked a
status-qualified assertion. Those unresolved asymmetries remain an owner review
obligation, not delivered contracts. W10's seven P7 and W12's three P4 relations
were separate planning supplements.

On 2026-10-02 the owner explicitly instructed: “压缩包不要提交，这些记录不需要存档”.
The 36 former development audit files were removed as loose intermediate records;
the temporary archive and manifest were also deleted. There is no retained audit
archive or live delta ledger. Historical counts are context only, not a current
file census.
Current contracts, unresolved items above and the focused approval record
remain available. This summary replaces obsolete phase/handoff navigation.

## Maintenance and validation

Development now contains guidance/governance rather than successive audit rounds.
This testing subdirectory contains the current summary, focused approval record and
no historical working-file archive; future status changes update these live records
instead of appending another phase/handoff report. This is evidence organization
inside the existing Testing class, not a new policy/architecture class.

Cleanup validation covers removal of intermediate/archive files, live relative links and
anchors, CI documentation checks, translation metadata and whitespace. It does not
rerun Guest/QEMU, code tests or architecture review. No production code, unsafe,
ABI or dependency changed; existing worktree changes are preserved and no PR merged.
