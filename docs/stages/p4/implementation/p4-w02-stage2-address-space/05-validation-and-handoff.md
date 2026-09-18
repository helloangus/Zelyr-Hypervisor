# P4-W02 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W02 detailed design](README.md).

## 1. Validation matrix

QEMU rows apply because Stage-2 behavior is a hardware-enforced property;
pure descriptor/ledger logic is validated host-side first. Guest-driven rows
are executed jointly with [P4-W05](../p4-w05-validation-guest/README.md)
scenarios and recorded by [P4-W08](../p4-w08-qemu-integration-regression/README.md)
automation; W02's matrix states what W02's evidence must show, not who runs
it.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 | P4-A02–A04 encoding correctness | descriptor/unit review | exhaustive `S2Flags`→field tests vs pinned spec revision | all variants correct; reserved bits never set | the encoding is right; not that hardware honors it |
| W02-DV02 | P4-A01 create/destroy | lifecycle unit tests | create→destroy→create with accounting stubs | clean allocation/accounting restoration; exhaustion handled; no partial state on errors | independent lifecycle logic; not on-target behavior |
| W02-DV03 | P4-A02/A03/A05 mutation + query | mutation unit suite | map/unmap/protect/query sequences incl. error paths | ledger and descriptors agree after success and after each error; all-or-nothing holds | mapper logic; not translation enforcement |
| W02-DV04 | P4-A06 activation | on-target review/test | activate then Guest executes through the space (with W04/W05) | Guest fetch/data accesses translate only through this space's mappings | active-context installation works; not multi-context operation |
| W02-DV05 | P4-A07 current-path consistency | on-target negative sequence | map→access→unmap→access-fault and protect→access-immediate sequences | post-mutation Guest accesses observe the new state with no stale translation window demonstrated | current-path invalidation works; not cross-pCPU shootdown (explicitly out of P4) |
| W02-DV06 → P4-V07 | isolation: unmapped + bounds | Guest negative scenarios (W05 VG-004 class) driven via W08 | Guest accesses an unmapped IPA; access at Guest-RAM boundary | Stage-2 fault taken; EL2 diagnostically live; fault context matches the unmapped IPA | Guest memory bounds enforced by Stage-2; not DMA/IOMMU isolation |
| W02-DV07 → P4-V07 | isolation: Hypervisor-owned ranges | Guest negative scenario via W08 | Guest attempts access into a Host-owned range left unmapped in Stage-2 | blocked by Stage-2 with diagnosable fault; no Host state disturbance | Host protection via unmapped-by-default; not complete TCB isolation |
| W02-DV08 → P4-V08 | permission enforcement + no stale reliance | Guest permission scenarios via W08 (W05 VG-005/VG-006 class) | write to read-only page; execute-permission comparison; protect-then-access without remap | faults distinguishable per W06 classification; immediate effect proves no stale-permission reliance | R/W/X enforcement and invalidation correctness; not all permission corner cases |
| W02-DV09 | repeated lifecycle (task-book "apply each mutation before later Guest execution") | repeat test | create→map→activate→destroy→create cycle | identical observable behavior across cycles | no hidden state; not long-run soak performance |
| W02-DV10 | multi-pCPU non-preclusion | design/record review | inspect the invalidation seam and limitation records | current-path-only scope recorded; seam extension point named | the P4 result does not preclude later shootdown; not that shootdown works |
| W02-DV11 | unsafe and layering review | static review | audit `unsafe` sites and module dependencies | exactly the two `unsafe` primitives in `s2-table` plus sysreg writes, each with SAFETY notes; Core sees no descriptor bits; no board/QEMU names | controlled-unsafe and layering compliance; not functional correctness |

Record each as **passed / failed / blocked / not run** with command or review
input, environment, date, and reason. QEMU-derived rows (DV04–DV08) prove
behavior in the stated QEMU reference environment only; they do not prove
real-hardware semantics (W01 A7). Rows are not claims: until the verification
record exists, all are not run.

## 2. Error model

- Every W02 entry point returns a named `Stage2Error`; none panics on
  Guest-influenced conditions (P4 Guest cannot call these APIs at all, but
  the discipline holds for fault-context consumers).
- Hardware Stage-2 faults are *inputs* to W06's classification, not errors of
  W02; W02 provides query context ([03 §3.5](03-code-contracts-stage2-core.md))
  for that diagnosis.
- Fatal escalation is reserved for hypervisor invariant violations (second
  activator, use-after-destroy, allocator misbehavior) and follows the P0
  failure-classification contract; the escalation path is named, never an
  ad-hoc panic inside Arch code.

## 3. Security model

- Untrusted-boundary posture: Stage-2 is the mechanism that makes
  Guest-untrusted memory access enforceable (ADR-007); the Guest has no API
  into the mapper. Security-relevant invariants: zeroed root at create
  (translates nothing by default); VMID uniqueness; unmapped-by-default for
  all Host-owned memory; BBM+invalidation ordering so permission revocation
  is effective immediately for the current path.
- Standing conflict kept visible: **P2-ACR-01 (`ADR Required`)** — the
  ADR-level `MemoryObject`/`MemoryRegion` model remains unresolved; W02's
  `MappingGrant` is stage-local and asserts no API stability, so a future
  ADR-conformant object model can supersede it without a compatibility
  claim to defend.
- `unsafe` surface: the two table-memory primitives plus sysreg write
  sequences, each requiring a four-point SAFETY justification at the call
  site and an inventory entry; growth beyond this list is a review failure.

## 4. Observability model

Events (`s2.space.create/destroy`, `s2.map`, `s2.unmap`, `s2.protect`,
`s2.activate`, `s2.invalidate`) carry space identity, IPA range, and outcome;
they route through the P0 logging/trace baseline (W01 A8) and are the
correlation keys W06/W07 use for fault context and repeatability counts.
No event carries Guest data content. High-rate events (page-granular
invalidation loops) are batched into one range event per operation, matching
the trace-cost guidance in the Coding Guidelines.

## 5. Handoff checklist

Before handing W02 work to a reviewer:

- exact changed-file list and the implementation-record path
  (`../p4-w02-stage2-address-space-record.md`);
- DV01–DV11 statuses with explicit not-run/blocked entries and the upstream
  rows (W01 R08/R09/R02/R03) each blocked item waits on;
- new `unsafe` list with SAFETY note locations and inventory delta;
- confirmation that Core-visible vocabulary leaked no descriptor bits, VMIDs,
  or sysreg names;
- factual capability/limitation notes for [P4-W09](../p4-w09-closeout-p5-handoff/README.md)
  (current-path-only invalidation; capability-gated range invalidation
  choice; temporary identity placement consumed from W03);
- handoff to consumers: activation boundary to
  [P4-W04](../p4-w04-vcpu-entry-exit/README.md), query/negative basis to
  [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md), mapping-input
  contract to [P4-W03](../p4-w03-guest-memory-image/README.md), events to
  [P4-W07](../p4-w07-repeatability-telemetry/README.md);
- open items: P2-ACR-01 unchanged; any Specification Investigation entries
  recorded with citations.
