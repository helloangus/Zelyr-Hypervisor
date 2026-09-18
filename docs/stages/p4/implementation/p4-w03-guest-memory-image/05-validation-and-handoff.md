# P4-W03 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W03 detailed design](README.md).

## 1. Validation matrix

Host-side rows validate logic without upstream runtime; QEMU rows are
executed through the integrated path (W02 mapping, W04 entry, W05 scenarios)
and recorded by [P4-W08](../p4-w08-qemu-integration-regression/README.md).
The matrix defines what W03's evidence must show.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W03-DV01 | P4-B01 RAM origin | allocation-path review + unit tests | trace every page of the region to an allocator call; protected-range exclusion inherited from the allocator contract (M2) | every byte of Guest RAM is covered by allocator-derived, Guest-marked pages; zero direct physical-address sources | the origin discipline of the code; not the allocator's own correctness (P2's evidence) |
| W03-DV02 | P4-B02 layout as test contract | record review | inspect `GuestLayout` record, version tag, and consumer citations (W02/W04/W05) | one authoritative record; all consumers cite it; values documented as P4 facts, not ABI | layout consistency; not fitness for any future machine |
| W03-DV03 | P4-B04 negative image inputs | negative unit suite | empty, oversize, misaligned, overlapping-destination cases; property-style classification of arbitrary sizes | every case rejected with the named error before any write; zero partial writes | loader rejection behavior; not Guest execution |
| W03-DV04 | P4-B04 prohibited overwrite | bounds-proof review | verify `validate_image_plan` + write-view bound checks jointly cover every store path | no store path exists without a prior in-bounds proof | overwrite impossibility in W03 paths; not Stage-2 enforcement (W02's) |
| W03-DV05 | P4-B05 deterministic initialization | determinism tests | two full constructions compared byte-wise; re-init equivalence; boot-info field scan for varying content | byte-identical results across constructions; no timestamps/randomness in Guest-visible memory | construction determinism; not whole-boot determinism (W07's declared scenarios) |
| W03-DV06 → P4-V03 | integrated construction | on-target construction via W08 automation | build → embed → boot → construct → Guest runs (W05 marker) | Guest executes from the loaded image; construction repeats cleanly in the same session | end-to-end load works on target; not hardware generality |
| W03-DV07 | release sequencing | lifecycle test | destroy-space-then-release ordering (D8) incl. attempted wrong-order call | wrong order rejected by API shape; correct order restores accounting exactly | ownership hand-off soundness; not allocator internals |
| W03-DV08 | unsafe and layering review | static review | audit the write view and module dependencies | single `unsafe` writer with three-point SAFETY notes; no board/QEMU names; no Arch registers in W03 modules | controlled-unsafe and layering compliance; not functional correctness |
| W03-DV09 | guest-untrusted readiness | design review | confirm no Guest-reachable input exists in W03 paths and Guest-side validation duty is assigned (W05) | documented; boot-info validated Guest-side | the package does not create Guest-input surfaces; not P5's copy framework |

Record each as **passed / failed / blocked / not run** with command or review
input, environment, date, and reason. QEMU rows prove the stated reference
environment only. Rows are plans until the verification record exists.

## 2. Error model

`GuestMemoryError` classes ([03 §6](03-code-contracts-guest-memory.md)) are
VM-facing recoverable values for setup consumers: allocation failures and
image rejections produce structured failures that stop Guest construction
without disturbing the Host. Two classes escalate to the fatal
failure-classification path because they indicate broken host-authored
contracts, not Guest behavior: `LayoutMismatch` and any write-path fault
after a proven-bounds plan. This split keeps Guest-facing and invariant
failures disjoint (W01 A2) even in a package the Guest never talks to.

## 3. Security model

- Threat posture: the Guest is untrusted (ADR-007) but has not executed
  during W03's operations; W03's security duties are therefore (a) that
  Guest RAM can only contain pages the allocator guarantees (P4-V03 origin
  requirement), (b) that no loader path can write outside the proven plan
  (forbidden-overwrite requirement), and (c) that no Guest-input surface is
  created ahead of P5's guest-safe-copy framework.
- The Guest-side boot-info validation duty (defensive parsing of a
  semi-trusted block) is assigned to [P4-W05](../p4-w05-validation-guest/README.md)
  and exists to keep the test asset itself honest, not because the block is
  Guest-controlled in P4.
- Standing conflict kept visible: **P2-ACR-01 (`ADR Required`)** — `GuestRam`
  is a stage-local handle, not the ADR `MemoryObject`; the Reserved
  re-entry point records where the ADR-conformant model supersedes it.
- `unsafe` surface: one bounded write view; growth beyond it is a review
  failure.

## 4. Observability model

Events (`gm.ram.allocate`, `gm.ram.release`, `gm.load.plan`, `gm.load.copy`,
`gm.bootinfo.write`) route through the P0 logging/trace baseline (W01 A8)
carrying region identity, sizes, and outcomes; layout version and image
identity flow into diagnostics for fault correlation (W06) and repeat
evidence (W07). No event carries Guest data content.

## 5. Handoff checklist

Before handing W03 work to a reviewer:

- exact changed-file list and implementation-record path
  (`../p4-w03-guest-memory-image-record.md`);
- DV01–DV09 statuses with explicit not-run/blocked entries and the upstream
  rows (W01 R07–R12, R04) each blocked item waits on;
- new `unsafe` list (write view) with SAFETY note locations and inventory
  delta;
- confirmation that layout values are recorded as test contracts and that no
  consumer hard-codes them;
- factual notes for [P4-W09](../p4-w09-closeout-p5-handoff/README.md):
  selected route, layout version, accounting degradation status (if M3
  gap-degraded), console-page convention;
- handoff to consumers: entry/stack/boot-info inputs to
  [P4-W04](../p4-w04-vcpu-entry-exit/README.md), image route and boot-info
  format to [P4-W05](../p4-w05-validation-guest/README.md), mapping grants
  and sequencing duty to [P4-W02](../p4-w02-stage2-address-space/README.md),
  determinism/layout facts to
  [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md),
  [P4-W07](../p4-w07-repeatability-telemetry/README.md), and
  [P4-W08](../p4-w08-qemu-integration-regression/README.md);
- open items: P2-ACR-01 unchanged; any Specification Investigation or
  upstream-mismatch records cited.
