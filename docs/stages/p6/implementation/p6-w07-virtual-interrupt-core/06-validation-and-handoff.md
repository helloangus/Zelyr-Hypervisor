# P6-W07 Validation, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md). This file closes the workflow in
[05-implementation-workflow.md](05-implementation-workflow.md).

## 1. Validation matrix

W07 is controller-independent, so most of its validation is host-side;
runtime rows exist where the lifecycle interacts with producers and
consumers. QEMU rows **prove the declared QEMU environment only** — GIC
emulation success does not prove real-hardware GIC behavior, and W07 rows
prove nothing about W08's List-Register mechanics or W09's maintenance
processing.

| ID | Requirement (plan/task book) | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W07-DV01 | Typed identities, window intake | host-side unit tests | range/width edge tests; declaration conflict fixtures; corruption of the declaration input | constructors reject out-of-range values; intake failure disables delivery with diagnosis | validation logic; not platform capability truth |
| W07-DV02 | Lifecycle semantic coverage: pending → presented → completed (P6-V11 W07 share) | host-side state-machine tests + review against [02] §4 | transition-matrix tests; QEMU end-to-end single-vIRQ case with W06/W08/W09 once available | every legal transition reachable and only legal transitions occur; end-to-end event presented and completed exactly once | the W07 state model; not LR mechanics (W08) or maintenance (W09) |
| W07-DV03 | Deferred and unavailable-vCPU outcomes (P6-V12 W07 share) | host-side tests + QEMU deferral case | inject to absent vCPU → pending persists → selection after "entry"; inject to draining/destroyed vCPU → typed outcomes; drain determinism | pending preserved across absence; dead/draining targets produce named denials; drain drops are counted and never silent | deferral semantics; not wakeup latency (P7) |
| W07-DV04 | Repeated arrival policy (P6-V14 basis) | host-side tests | repeat inject in each state (Pending/Presented/Active); saturating counter wrap; completion-then-representation | coalescing per D5; no state corruption; pending+active never presented before completion (D6) | the documented repetition policy; not storm robustness (W12) |
| W07-DV05 | Request authorization (P6-V11 basis; P6-V21 input) | host-side denial-matrix tests + QEMU Guest-request path (consumes P5) | all §4 denial classes exercised, incl. stale handle, wrong rights, cross-VM target, SGI-only rule, priority rule, malformed payload | every denial structured, counted, state-preserving; no identity/role shortcut; Guest fault never panics the Host | the W07 validation spine; not P5 machinery itself |
| W07-DV06 | Target isolation (P6-V18 basis) | structural review + QEMU multi-vCPU case **only with** the evidenced prerequisite (else recorded stage block) | inject targeting vCPU A while observing vCPU B's bank; two-bank concurrent injection tests | B's state unchanged in every case; per-bank locks never nested | structural isolation; multi-vCPU evidence only when the prerequisite exists |
| W07-DV07 | Diagnostics and telemetry | review + counters observed in DV02–DV06 | counters/trace per [02] §7 present and correlated; protocol-misuse and impossible-state recoveries counted | all named counters populated by exercised paths; violations forced-consistent | observability basis; not collection/reporting (W13) |
| W07-DV08 | Controller independence + handoff readiness | design-conformance review | inspect module graph and sources for register/GIC concepts; handoff checklist §3 | zero register access; consumers (W08/W09/W10/W11/W12/W13, P7/P8) can act on the contracts without inventing semantics | scope conformance; not consumer correctness |

Planned, run, blocked, and failed are distinct evidence states recorded in
the verification record with command, input, environment, timestamp, and
reason. No W07 validation proves P6-V16/P6-V17 (W08/W09), Guest-visible
masking/priority (W10), real-hardware behavior, or scheduler semantics, and
none may be reported as doing so.

## 2. Error, security, and observability model

**Error model.** The taxonomy of
[04](04-code-contracts-authorization-and-errors.md) §4 is exhaustive:
Guest-caused outcomes are the seven structured denials (no state change,
no panic); Host-attributed anomalies are `ProtocolViolation` and
`InternalInvariant`, which force the bank to the nearest consistent state
and count the kind — an anomaly is surfaced, never absorbed. There are no
timeouts, retries, or drop-silently paths anywhere in W07.

**Security model.** Authorization is capability+rights+generation through
the P5 boundary; the same-VM and SGI-only class rules are enforced in the
one reviewed spine; Guest-controlled integers never reach state untyped.
Claims are unforgeable typed tokens invisible to Guests. No W07 state is
reachable through Guest memory, and no denial can be distinguished by a
Guest as anything but a structured class. The absence of `unsafe` in W07 is
a design property enforced at review (W07-DV08).

**Observability model.** Per-bank counters and the three trace events of
[02](02-architecture-and-state.md) §7 are the designed surface; P6-W13
owns collection, correlation (P6-V24), and any latency work. Occurrence
counters record suppressed duplicates, which makes later coalescing-policy
discussions (W10 and beyond) evidence-based rather than speculative.

## 3. Handoff checklist

Before handing W07 to a reviewer, provide:

- the exact changed-file list and the crate/module placement chosen for the
  logical modules;
- confirmation that W07 added **no** `unsafe`, no crate dependency, and no
  architecture-register access;
- W07-DV01–DV08 evidence paths and run status, including explicit not-run
  or blocked entries (multi-vCPU row without the prerequisite; Guest-path
  rows blocked by a P5 gap; end-to-end rows blocked by W08/W09 absence);
- the declared `VirqWindow` and priority width as actually intake-validated;
- confirmation that no LR/maintenance/masking/wakeup semantics, vGIC MMIO
  model, machine ABI, or persistent serialization format was introduced;
- open items for consumers, without resolving their contracts here:
  - **P6-W08** (`../p6-w08-gic-virtualization-interface/README.md`):
    selection/claim/return and the D6 exclusion are the contract W08
    consumes for entry-load, purge, and refill;
  - **P6-W09** (`../p6-w09-maintenance-interrupt/README.md`): the
    completion-report path and duplicate-completion protection are in
    place for maintenance processing;
  - **P6-W10** (`../p6-w10-interrupt-semantics/README.md`): the lifecycle
    substrate (pending/active truth, priority carriage, occurrence
    counts) is available for masking/priority/repeated-event semantics;
    the D6 exclusion and default-priority rule are W07-owned bounded
    behavior W10 may amend by design;
  - **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`):
    the Guest-request class (SGI-only, default priority) defines what
    VG-IRQ scenarios can request;
  - **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`): the
    denial taxonomy and forced-consistency rules are the isolation
    surface to stress;
  - **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`):
    counters and trace identities are available; collection is W13's;
  - **P7/P8**: pending-state observation and drain ordering are available;
    wakeup policy (P7) and the Linux-visible vGIC model (P8) remain
    theirs — no frozen API is implied.

No completion claim may be made anywhere in this design; completion
evidence belongs only in
`../../verification/p6-w07-virtual-interrupt-core-verification.md`.
