# P3-W07 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

## 1. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W07-DV01 | Slot layout, kind encoding, init | layout/encoding review + table tests | encode/decode round-trips; invalid-kind refusal; init postconditions on a slot-fake | every field round-trips; kind ≥ 4 refused; init validates all slots | contract correctness of the encoding; not cross-CPU behavior |
| W07-DV02 | Delivery and ordering | host protocol tests (threads-as-CPU) | send→poll pairing; wait wake pairing; payload-never-before-sequence assertions; spurious-wake tolerance | exact pairing across the declared thread counts; no fabricated event | protocol correctness under the host memory model; not AArch64 SMP behavior |
| W07-DV03 | Self-notification defined | targeted test | self `notify` → own accounting updates → next poll delivers | identical behavior to remote send minus wake; accounting exact | the defined self case; not scheduler-style self-wakeup semantics |
| W07-DV04 | Concurrent sender behavior | storm test | N senders → 1 target at declared limits; coalescing gaps counted | no lost sequence; arrivals + coalesced == sends; latest-kind delivery | defined coalescing; not per-event attribution (recorded limitation) |
| W07-DV05 | Invalid/offline targeting | gate-matrix tests | each error row under its gate state (pre-SmpReady; unknown id; offline-class id; invalid kind) | every refusal side-effect-free (slot-fake untouched); error names correct | fail-closed targeting; not hotplug-era staleness (recorded boundary) |
| W07-DV06 | Accounting explainable | invariant tests | storm + mixed traffic; assert arrivals/coalesced/per-kind invariants exactly | invariants hold at declared limits; no drift | accounting exactness of the protocol; not W11 catalog coverage |
| W07-DV07 | Integration, scope, and limit recording | closure review | read integration map vs W08/W11/W12/W13 plans; confirm no platform constants; confirm non-RPC statement | consumers' obligations citable; no board/SoC constant; limit statement present | handoff readiness; not that consumers are done |

Task-book trace: P3-V07 passes when targeted, self, concurrent, invalid,
and offline notification behaviors are defined and recoverable with
explainable arrival/type accounting; it does not prove a general
communication facility (that exclusion is this stage's non-RPC limit,
[04 §5](04-code-contracts-notification-api.md)). W07 delivers the
behaviors and host-side evidence; notification storms at QEMU SMP scale
are [P3-W12](../p3-w12-smp-stress-failure-tests/README.md) rows and the
[P3-W13](../p3-w13-qemu-smp-regression/README.md) matrix. Evidence states
(passed / failed / blocked / not run) are recorded with command,
environment, and timestamp in
`../../verification/p3-w07-cross-cpu-notification-verification.md`;
implementation decisions go to
`../p3-w07-cross-cpu-notification-record.md`. Neither file is created by
this design.

## 2. Error, security, and observability model

- **Errors:** `NotifyError` is total and side-effect-free; there is no
  runtime error state in the slots. Slot corruption (reserved bits,
  impossible sequences, invalid kinds) is fatal-class per the P0-W14
  classification with CPU attribution — the primitive never "fixes up" a
  slot.
- **Security:** the primitive is host-only; the authorization boundary is
  the targeting gate (W03 online set + W05 phase). There is no guest
  reach, no capability concept, and no payload trust: payload bytes are
  interpreted only by the kind's owning design, so a malicious sender
  within the hypervisor's own trust boundary can at worst send a
  well-formed event — the threat model treats sender code as hypervisor
  code (no guest path exists at P3).
- **Observability:** slot counters are the activity surface; W11 owns
  event ids/catalog and may mirror or aggregate. Send-side visibility
  comes from `notify` return values at call sites (W12 counts them in
  tests). Wake storms (SEV broadcast) are bounded by the ≤ 8 CPU
  inventory and are observable as spurious-wake counts in test harnesses.

## 3. Handoff checklist

Before handing W07 to a reviewer, provide:

- the changed-file list and the `unsafe` inventory entries (`sev`/`wfe`
  boundaries);
- DV01–DV07 evidence paths with run status, including explicit not-run
  entries (QEMU SMP storms, SGI-carried delivery, hotplug staleness);
- the kind-claim table status (0 Doorbell defined; 1 claimed-by-W08
  pointer; 2–3 unclaimed; ≥ 4 invalid);
- the declared host-side test limits and the coalescing limitation as
  recorded statements;
- open items for W08 (kind 1 payload semantics; poll-driven consumption
  per BW-4), W11 (counter catalog/aggregation), W12/W13 (storm and
  regression rows), W14/P4 (notify contract as foundation), and the P6
  extension point (SGI carrier) — without resolving their contracts here;
- confirmation that no queue, RPC, timer, GIC access, scheduler hook, or
  W02/W05 edit was introduced.
