# P6-W04 SGI accounting-unit remediation

Chinese readers can use the [Chinese edition](07-accounting-units-remediation.zh-CN.md).

**Status:** Proposed design correction; no production implementation or runtime evidence.
**Scope:** AUD-005 / P6-V04–V05; counter units and evidence interpretation.
**Version:** v0.1
**Owner/change context:** Owner-directed full audit remediation, 2026-10-02.
**Supersedes:** Register-write versus receipt equality and inferred failed-target drift.
**Parent:** [W04 design](README.md); [W04 plan](../../plans/p6-w04-smp-interrupt-routing-sgi.md).

## 1. Basis and counterexample

One encoding can name several targets, so one register write can correspond to
multiple receiver acknowledgements. These counters have different units.
Additionally, [Arm DAI0492 version 3.0, §7.2, pp.36–37](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/GICv3_v4_overview.pdf?revision=65f91645-cd52-4795-952b-f01095ff5ef8)
explains that overlapping same-ID SGIs from two senders can produce one receipt
at their common target in GICv3. A later send after acknowledgement can instead
produce another receipt. Quiescence alone therefore does not make send-attempt
and receipt counts equal. This is source/contract review, not execution evidence.

## 2. Required units and authority

| Quantity | Unit and authority | Meaning / limit |
|---|---|---|
| accepted_calls | fully emitted logical calls; sender/W04 | not number of targets or acknowledged messages |
| register_writes | actual emitted encodings; sender/W04 | not target cardinality |
| target_attempts[pcpu] | intended signals to that target from emitted call; W04 | one per validated member per successful call; not delivered count |
| validation_rejections | rejected pre-emission calls; W04 | zero writes/attempts; never an expected-loss allowance |
| acked[pcpu] | receiver acknowledgements of this SGI; W03 | cannot identify a sender or logical call by SGI ID alone |
| completed[pcpu] | W03 completed interrupt lifecycles | not proof of a consumer's deferred work completion |

Emit target identity/membership and encoding counts separately. For IRM, record
its eligible-set snapshot and prove the declared hardware target-domain matches
before using exact-target assertions; otherwise evidence is unavailable for that
form. No implicit hardware filtering by a software OnlineSet is assumed.

W04 retains no second authoritative receipt mirror. Counters use W13's bounded
atomic update convention so IRQ preemption cannot lose sender updates. A sampled
view records run/epoch, units, approximate/quiesced mode and overflow status.
Counter overflow cannot silently wrap into a valid exact delta. No receipt-total
subtraction estimates how many sends coalesced or were lost.

## 3. Exact comparison belongs to a controlled validation scenario

For DV04/DV05 exact per-target assertions, the harness must establish:

1. Exclusive use of the validation SGI and a known drained initial state; no
   unrelated sender/consumer traffic in the comparison epoch.
2. At most one outstanding attempt per target/SGI. Start the next round only
   after each intended receiver has completed its W03 interrupt lifecycle and
   the harness has observed the round's completion through its test-owned
   synchronization. This is a fixture, not a new production SGI protocol.
3. Stable eligible target membership, completed sends, no counter overflow,
   and stable post-drain samples; all declared targets are checked individually,
   including absence of receipts on unintended monitored targets.
4. Bounded observation with an explicit failed/inconclusive result when the
   premises or drain cannot be established. A failed target is not subtracted
   as an invented allowance to make an equality pass.

Then compare per-target attempt, acknowledgement and W03 completion deltas.
Never compare register writes to their sum. A multi-target encoding can have
one write and N individual completions. Outside these premises the report is
ObservedOnly/ExactUnavailable; concurrent same-ID tests validate signal behavior
and bookkeeping, not one acknowledgement per send. No SGI-level payload,
retransmission, fairness or guaranteed completion is introduced.

## 4. Validation and handoff

DV02 checks units across single, same-affinity multi-target and multi-encoding
sets; rejection contributes zero attempts. DV05 adds overlapping same-ID senders,
concurrent snapshots, readiness changes, overflow and exact controlled epochs.
Host-fake results must not be labeled QEMU/GIC behavior. W11 consumes these limits
for its Host-trigger/Guest-tail scenarios; W13 carries units and sample premises
into telemetry and regression. Final counter schema/storage remain W13-owned.

The dimensionally invalid proposed comparison is removed from W04 and consumers.
AUD-005 design accounting is corrected; package approval, implementation and
P6-V04/V05 execution remain pending. No production code, unsafe, dependency or
public ABI is changed by this document.
