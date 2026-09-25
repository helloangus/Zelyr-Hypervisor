# P1-W06 Early Console and Bring-up Logging — Detailed Implementation Design

Read the [current-state reconciliation](05-implementation-reconciliation.md)
first for delivered P0 diagnostic semantics and the init-time write boundary.

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The early console required by
[P1-W06](../../plans/p1-w06-early-console-logging.md): a polled, fixed
reference-console output channel available from its bring-up through the
stable EL2 state and across the Host Stage-1 transition; the bounded message
categories it carries (phase markers, capability content, exception/panic
report lines); the channel-availability signal; the marker line format; and
the documented reference-console assumption with its replacement boundary.  
**Owner/change context:** P1-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W06. W06 gives the stage one
reliable, bounded, allocation-free output channel and fixes who may print
what through it. It deliberately does **not** design the future console
subsystem, probe or discover hardware, implement a full UART driver, add a
logging framework or log-level engine, introduce structured tracing, or make
any QEMU-specific constant a Core contract.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the logical
  module map, the message-category model and prefix vocabulary, ownership of
  every piece of channel state (including the temporal single-consumer rule
  on the UART), the channel lifecycle across the MMU transition, the
  concurrency model, and the assumed-contract table. Load this first for any
  step.
- [02-code-contracts-channel.md](02-code-contracts-channel.md) — contracts
  for the channel phase body (`console` mechanism), the transport
  (init/write/write-str), the availability signal, the marker line format,
  the bounded formatter, and the audited MMIO boundary.
- [03-code-contracts-integration.md](03-code-contracts-integration.md) —
  integration contracts with W05 (exception-context callability), W07
  (report-line transport), W03 (capability-content transport), W08 (MMIO
  mapping class and transition continuity), W09 (replay and marker
  emission), W10 (token classes), plus the replacement boundary and
  prohibited content.
- [04-implementation-and-review.md](04-implementation-and-review.md) —
  ordered workflow, validation matrix, error/security/observability model,
  and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W06
plan](../../plans/p1-w06-early-console-logging.md). This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W06 plan → this
design → Coding Guidelines. In particular:

- The task book requires "early console and structured bring-up markers"
  with P1-V10: diagnostics identify each required stage from entry to stable
  state and remain available across the Host Stage-1 transition.
- The plan scopes startup markers, capability output, exception/panic
  output, MMU-transition markers, reference-console assumptions, and
  pass/stable markers — and explicitly excludes the generic console
  framework, platform probing, device drivers, the runtime tracing
  architecture, and any permanent QEMU-specific Core contract. A channel
  that registers devices, enumerates UARTs, or grows a trait framework
  would violate that exclusion; none is designed here.
- The [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime and
  [W09](../p1-w09-initialization-sequencing/README.md) lifecycle are
  accepted sibling contracts consumed directly: the channel becomes
  available inside W09's `console` phase, its availability signal triggers
  W09's marker replay (`materialize_replay`), and its failure routes via
  the matrix's `console` row. The
  [W05](../p1-w05-el2-exception-entry-baseline/README.md) vectors and
  [W08](../p1-w08-host-stage1-address-space/README.md) mapping are parallel
  designs consumed through recorded seams with failure boundaries.
- The P0 logging and panic contracts (work seq 1) are **assumed
  contracts**: [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)
  (diagnostic-category semantics: human log, structured trace, metrics,
  crash dump are distinct channels never substituted for one another; every
  diagnostic associates build/version identity; minimal crash-information
  principle), [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md)
  (failure-class semantics: P1 has no Guest, so every P1 failure is a
  hypervisor-invariant/fatal class — the guest-caused class is untouched),
  and [P0-W16](../../../p0/plans/p0-w16-version-build-metadata-baseline.md)
  (identity content, already embedded by W02's `BuildIdentity`). W06 cites
  their semantics; it does not restate or re-own them. A missing or
  contradicting P0 baseline is an upstream defect to record, never
  permission to redesign P0 inside P1 (task book §1).

Classification: the polled reference-console transport (init, byte/line
write), the availability signal, the message-category model with its prefix
vocabulary, the marker line format, the bounded formatter integration, the
MMIO mapping requirement for W08, and the exception-context callability
guarantee are **Required**. Log-level filtering of the early channel, the
structured-trace boundary registration (P0-W13 namespace), replacement of
the fixed console by P2 discovery, any second console instance, and
bandwidth/throughput tuning are **Reserved** with recorded triggers. A
console framework or trait hierarchy, platform probing, a full PL011 driver,
a telemetry transport or ring buffer, a second output path outside the
W01/W02/W06/W07-owned set, and any permanent QEMU-specific Core contract
are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Identify P0 logging and panic contracts usable before discovery (work seq 1) | Authority section above; [architecture](01-architecture-and-state.md) §7; [workflow](04-implementation-and-review.md) step 1 | W06 closure review (W06-DV01) |
| Define the early channel's supported message categories and phase markers (work seq 2) | [Architecture](01-architecture-and-state.md) §2–§3; [channel contracts](02-code-contracts-channel.md) §3–§5 | P1-V10 (W06-DV02) |
| Integrate output requirements with W05 exception and W08 MMU transitions (work seq 3) | [Integration](03-code-contracts-integration.md) §1, §3; [channel contracts](02-code-contracts-channel.md) §6 | P1-V10 (W06-DV03) |
| Review fixed reference assumptions and the future replacement boundary (work seq 4) | [Architecture](01-architecture-and-state.md) §5; [integration](03-code-contracts-integration.md) §5; [workflow](04-implementation-and-review.md) step 4 | P1-V10 (W06-DV04) |
| Define evidence that output remains available through stable state (work seq 5) | [Workflow](04-implementation-and-review.md) §3 | P1-V10 (W06-DV05; execution via W10) |
| Hand off marker and diagnostic-channel expectations to automation and docs (work seq 6) | Downstream handoff below; [workflow](04-implementation-and-review.md) §5 | W06 closure review (W06-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no console or UART code exists anywhere in the tree; no
logging crate, no output abstraction, no marker vocabulary. The only output
mechanisms designed are W01's rejection reporter (pre-transfer) and W02's
early diagnostic writer (panic-route era) — both deliberately narrow. The
W09 lifecycle and W02 runtime exist as accepted designs, not code; W05/W07
are parallel designs. Every executing prerequisite is a contract, not a
present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| One reliable early output channel | No console code; W02's writer is panic-route-only by contract | The transport and availability signal of [channel contracts](02-code-contracts-channel.md) §1–§2 | "Reliable, diagnosable output from entry through stable state" needs one owned transport, not an accumulation of ad-hoc prints | W06 (this design) | W06-DV02 |
| Diagnostics identify each required stage from entry (P1-V10) | Phases 1–5 complete before any channel can exist (W09 decision 1) | The availability signal consumed by W09's replay (`materialize_replay`), plus the marker line format | Early stages are identifiable through the replay, not by inventing pre-console printers | W06 (signal + format); W09 (replay) | W06-DV02; W09-DV02 reads the same point |
| Markers, capability output, exception/panic output carried (plan scope) | W03 owns capability line content; W07 owns report content; neither owns a transport | The category model and emit-callback compatibility of [architecture](01-architecture-and-state.md) §2 and [integration](03-code-contracts-integration.md) §2 | Without a fixed carrier, each producer would invent its own output path — the exact proliferation the W02 output table forbids | W06 (transport); W03/W07 (content) | W06-DV02/DV03 |
| Output remains available across the MMU transition (P1-V10) | No mapping exists (W08 parallel) | The MMIO mapping requirement and transition-continuity contract of [integration](03-code-contracts-integration.md) §3 | A channel that dies at `SCTLR_EL2.M=1` fails the stage's own acceptance wording | W06 (requirement + markers); W08 (mapping class) | W06-DV03; NC5/R1 execution via W11/W10 |
| Reference-console assumption is explicit and bounded (work seq 4) | No documented console assumption exists | The fixed-region assumption and replacement boundary of [architecture](01-architecture-and-state.md) §5 | An undocumented fixed address is a hidden dependency (P1-V15's forbidden pattern) | W06 | W06-DV04 |
| Pass/stable marker expectations handed to automation (plan handoff) | W10 owns the token; W09 owns the emission point; no carrier fixed | W06's marker transport + token-class vocabulary for W10 matching | W10's verdict needs markers that were formatted by one owner | W06 (transport); W09 (points); W10 (token) | W06-DV06 |

No row requires designing the console subsystem or platform discovery; the
replacement boundary is recorded, not built.

## Resolved design decisions and their authority

1. **The early channel is the human bring-up log and the marker carrier —
   and nothing else.** P0-W12 separates human log, structured trace,
   metrics, and crash-dump channels; P1's early channel is the first two
   roles' minimal carrier (human-readable lines + phase markers), with
   trace-event registration (P0-W13 namespace) and metrics explicitly
   Reserved for the telemetry architecture. Authority: P0-W12 semantics
   (assumed contract); plan out-of-scope list.
2. **The reference console is a fixed, documented reference-platform fact —
   QEMU `virt`'s PL011 at its canonical address — owned by W06 as its own
   constant, never by consuming W01's boot-entry constant.** W01 §6 fixes
   this separation; two documented constants for one UART is the recorded
   arrangement (different owners, different lifecycles), and P2 discovery
   supersedes both through its own design. The constant lives in the
   boot/platform scope of the reference bring-up, never in generic Core
   (ADR-041/043/044: reference-platform facts are documented assumptions,
   not board-name branches). Authority: W01 §6 single-source rule;
   ADR-041/043/044; plan scope ("reference-console assumptions").
3. **The channel is level-less in P1: no log-level engine exists at this
   stage.** The early channel emits bounded, fixed-vocabulary lines only;
   every line is a bring-up diagnostic by construction. Level filtering and
   release trimming are P0-W12 semantics applied by the future logging
   subsystem (Reserved trigger). Rationale: filtering infrastructure with
   four line classes would be unused policy. Authority: P0-W12 (levels
   belong to the logging baseline's consumers); stage-local design freedom
   with recorded rationale.
4. **Availability is a W06-owned once-flag set by the `console` phase body —
   the availability signal W09's design already consumes.** No probing, no
   timeout, no "should work by now" heuristics: the signal means the
   transport was initialized and its start line was emitted. W09's
   `materialize_replay` waits for exactly this signal (W09 §5). Authority:
   W09 §8 seam table ("Channel write; availability signal; marker format —
   W06").
5. **W06 owns the marker line format; producers own line content.** Marker
   lines are W06-formatted from W09's label/event vocabulary; capability
   lines arrive as complete strings through W03's emit callback and are
   transported verbatim; report lines arrive through W07's renderer the
   same way. W06 never re-renders, truncates mid-vocabulary, or interprets
   content (W09 M4). Authority: W09 marker rules; W03 render contract §7
   ("W06 owns the channel and the marker format").
6. **A stuck transmitter is bounded by the harness, not by software
   policy: the transmit poll has no software timeout.** A polling timeout
   would be a silent-drop policy on the diagnostics path — the one path
   that must not drop. The wall-clock bound belongs to W10's runner
   (`FAIL-TIMEOUT`). Recorded limitation for W12: a hung UART hangs the
   boot, visibly, inside the harness bound. Authority: W02 R4 posture
   precedent (no fallback work on the fatal path); W10 timeout contract.
7. **Exception-context callability is a Required guarantee:** the channel
   has no locks and no DAIF manipulation, is callable with masks set from
   the guarded exception path (W07's post-arm report), and relies on the
   guard — not on a channel-internal lock — for single-consumer discipline.
   Authority: W05 handoff; [W02 architecture](../p1-w02-minimal-rust-el2-runtime/01-architecture-and-state.md)
   §4 concurrency model.
8. **Channel-start identity line: the first line the channel emits carries
   the build identity** (via W02's `BuildIdentity` accessor), satisfying
   P0-W12's identity-association requirement for the human log without
   per-marker identity overhead (markers identify phases; W07 reports and
   the start line carry identity). Authority: P0-W12 (assumed contract);
   W02 identity contract §2.

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for the
   module map, category model, state ownership, lifecycle, and assumed
   contracts. Every implementation step depends on it.
2. Load [02-code-contracts-channel.md](02-code-contracts-channel.md) for the
   transport, signal, format, and boundary contracts.
3. Load [03-code-contracts-integration.md](03-code-contracts-integration.md)
   when wiring any producer or consumer seam (W03/W05/W07/W08/W09/W10) or
   when touching the replacement boundary.
4. Execute the steps in the order given in
   [04-implementation-and-review.md](04-implementation-and-review.md):
   prerequisite confirmation, transport and signal, format and categories,
   integration seams, reference-assumption and replacement reviews, evidence
   and handoff.
5. Record implementation decisions and deviations in
   `../p1-w06-early-console-logging-record.md` when implementation begins,
   and validation commands, environments, and outcomes in
   `../../verification/p1-w06-early-console-logging-verification.md` when
   evidence exists. Neither file may exist yet, and neither this design nor
   a record may claim W06 complete.

## Explicitly excluded interfaces

No console trait, driver-registration framework, device probing, or
enumeration; no log-level engine, filter syntax, or runtime verbosity
control; no structured-trace events, metric counters, or ring buffers; no
DTB parsing or platform discovery; no second UART or multi-instance
support; no public ABI, wire format, or persistent layout — the prefix
vocabulary is a P1-internal boot diagnostic consumed by W10/W11 matching
rules, not an ABI. No new output path outside the recorded set (W01
rejection reporter; W02 early writer; W06 channel; W07 report renderer
through W06's channel or W02's writer): a fourth print path is a scope
failure (W06-DV04 walks the output-path inventory).

## Downstream handoff

- **[P1-W07](../p1-w07-fatal-crash-diagnostics/README.md)** receives the
  channel as its post-arm report transport (with the early writer as the
  recorded fallback), the exception-context callability guarantee, and the
  line-termination/framing rules its renderer must follow.
- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** receives the
  console MMIO region identity and its required mapping class (Device,
  execute-never) as a named input of its mapping-class table; W06 consumes
  W08's continuity guarantee and emits nothing itself during the transition
  (transition markers are W09's).
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `console` phase body (the mechanism entry its `console_step` adapter
  calls), the availability signal contract its replay depends on, and the
  marker line format its `record_marker` emissions render into.
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** receives the
  bounded marker vocabulary as the matchable evidence classes for
  boot-start markers; the stable token remains W10-owned (W09 M3), and its
  text is recorded before the first verdict-bearing run.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  guarantee that fault scenarios' diagnostic output (pre-arm summaries,
  panic-class reports) reaches the captured serial stream through one of
  the two recorded transports, and the marker-class vocabulary for
  scenario matching.
- **[P1-W03](../p1-w03-aarch64-capability-inventory/README.md)** has its
  render/emit seam confirmed unchanged: content and line order stay W03's,
  the transport becomes W06's channel at the `console` phase wiring.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  reference-console assumption, the channel categories and vocabulary, and
  the recorded limitations (level-less; fixed address; stuck-transmitter
  posture) for the diagnostics and limitations contracts.

A coding agent completing W06 must leave the handoff checklist in
[04-implementation-and-review.md](04-implementation-and-review.md)
answerable without inspecting W06 source code.
