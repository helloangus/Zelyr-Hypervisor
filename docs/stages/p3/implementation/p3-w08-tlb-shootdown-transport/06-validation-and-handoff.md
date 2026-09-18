# P3-W08 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

## 1. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W08-DV01 | Request encoding, slot state machine, mask type | encoding/layout review + table tests | encode/decode round-trips; Empty→Pending→Completed transitions on a slot-fake; illegal encodings fatal; mask algebra total | state machine exact; out-of-range ids refused by construction; no illegal encoding accepted | contract correctness; not cross-CPU behavior |
| W08-DV02 | Single/mask/broadcast selection and exclusion | selection tests | single-target; multi-bit mask; broadcast-minus-initiator; each with online/offline/unknown mixes | effective/excluded sets exactly per SR-1–SR-5; exclusions reported in every result | selection and exclusion correctness; not hotplug-era staleness (recorded) |
| W08-DV03 | Acknowledgement/completion correctness | host protocol tests | publish→consume→complete on threads-as-CPU; ordering assertions (descriptor stable before Pending; ack release before initiator's acquire observation) | exact completion per request; single consumption; no lost ack | transport protocol under the host memory model; not AArch64 SMP behavior and **not TLB invalidation semantics** |
| W08-DV04 | Invalid/offline targeting | gate-matrix tests | NotReady / UnknownTarget / EmptyMask rows; requested-but-offline exclusions | every refusal side-effect-free; exclusions diagnosable in results | fail-closed targeting; not lifecycle changes (none exist at P3) |
| W08-DV05 | Concurrent-request behavior | concurrency tests | concurrent initiators at declared thread counts; initiator-also-target (reactive wait) | serialized initiations; no deadlock (BW-4 interleaving exercised); exact accounting | single-flight semantics; not pipelined throughput (Reserved, P4) |
| W08-DV06 | Timeout and failure diagnostics | failure-path tests | induced non-consuming target; COLLECT_BOUND exhaustion; supersession after timeout; accounting invariants ([04 §6](04-code-contracts-transport-initiator.md)) | TimedOut with exact unacked set; transport reusable; invariants hold | diagnosable timeout; not wall-clock timeout semantics (P6) |
| W08-DV07 | Transport boundary and P4 handoff | closure review + boundary review | confirm descriptor never decoded, no TLBI/barrier/cache code, no Stage-2 types; read the P4-facing contract against W14's plan goal | boundary clean; both P4 obligations stated; gap explicit | contract readiness for P4; not that invalidation works — P3 asserts no Stage-2 TLBI semantics |

Task-book trace: P3-V08 passes when target/mask handling, acknowledgement,
completion, timeout, invalid/offline exclusion, and concurrent-request
behavior are diagnosable, and asserts no Stage-2 TLBI semantics. W08
delivers the transport and host-side evidence; QEMU-scale repeated
execution is [P3-W12](../p3-w12-smp-stress-failure-tests/README.md) and
the [P3-W13](../p3-w13-qemu-smp-regression/README.md) matrix. Evidence
states (passed / failed / blocked / not run) are recorded with command,
environment, and timestamp in
`../../verification/p3-w08-tlb-shootdown-transport-verification.md`;
implementation decisions go to
`../p3-w08-tlb-shootdown-transport-record.md`. Neither file is created by
this design.

## 2. Error, security, and observability model

- **Errors:** `TransportError` (validation refusals, side-effect-free)
  and `TimedOut` (diagnosable, recoverable by supersession) are the only
  failure modes; slot/control corruption is fatal-class per the P0-W14
  classification with CPU attribution. There is no partial-failure state:
  a request is Published-then-Completed or Published-then-Pending, always
  attributable.
- **Security:** host-only surface; the authorization boundary is the
  selection gate (W05 phase + W03 registry + SR-3 self-exclusion). The
  descriptor is hypervisor-internal; no guest path exists at P3, and the
  trust boundary note in
  [03 §5](03-code-contracts-transport-request.md) records where P4 must
  place validation when it defines descriptor content.
- **Observability:** per-target slot states, the result masks
  (completed/unacked/excluded), and the target-private completion
  counters are the activity surface; W11 owns the event catalog and may
  aggregate. Timeout diagnostics name CPUs, which is the property the
  task book's "diagnosable failure" requires.

## 3. Handoff checklist

Before handing W08 to a reviewer, provide:

- the changed-file list and the `unsafe` inventory entries (none is
  expected beyond W07-owned `sev`/`wfe` reuse — confirm);
- DV01–DV07 evidence paths with run status, including explicit not-run
  entries (QEMU SMP transport runs, real invalidation, pipelining,
  timer-based timeouts);
- the recorded `COLLECT_BOUND` value and rationale, and the no-timer
  limitation statement;
- the single-flight decision and its P4 revisit trigger, plus the BW-4
  reactive-wait conformance of the implemented wait loops;
- the boundary-review verdict (DV07): descriptor opaque, no invalidation
  semantics asserted, both P4 obligations (descriptor meaning; bound
  operation with barriers) stated for
  [P3-W14](../p3-w14-p4-smp-handoff/README.md);
- open items for W11 (activity catalog), W12/W13 (stress and matrix
  rows), W14/P4 (transport contract consumption), W15 (semantic-gap
  statement in stage docs) — without resolving their contracts here;
- confirmation that no Stage-2 type, VMID/IPA field, TLBI instruction,
  barrier, cache-maintenance op, GIC access, timer, or second wake
  mechanism was introduced.
