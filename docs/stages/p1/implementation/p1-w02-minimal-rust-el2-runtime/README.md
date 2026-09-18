# P1-W02 Minimal Rust EL2 Runtime — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The minimal Rust `no_std` EL2 runtime required by
[P1-W02](../../plans/p1-w02-minimal-rust-el2-runtime.md): boot entry and stack
establishment, the Rust entry environment, static-data readiness, boot-context
retention, the early panic route, build identity, the initialization-sequencer
seam, and the controlled stable idle.  
**Owner/change context:** P1-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W02. W02 is the first
code-bearing P1 package: it turns the W01 entry contract into an executing
runtime and fixes the seams that every later P1 package consumes. It
deliberately does **not** design capabilities (W03), the EL2 control baseline
(W04), vectors (W05), the console (W06), crash diagnostics (W07), the Host
Stage-1 transition (W08), the initialization sequencer itself (W09), or any
Guest/SMP/GIC/discovery/allocator mechanism.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the logical
  module map, ownership of every piece of boot state, the establishment order,
  the concurrency model, and the assumed-contract table with failure
  boundaries. Load this first for any step.
- [02-code-contracts-entry-assembly.md](02-code-contracts-entry-assembly.md) —
  contracts for the image entry point, the assembly establishment sequence
  (stack, SPSel, DAIF, BSS), the boot stack, and the Rust transfer.
- [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) —
  contracts for the Rust entry, the boot context, runtime establishment, the
  W09 sequencer seam, and the controlled idle.
- [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md) —
  contracts for the early panic route, the build identity, and the early
  diagnostic writer, including the W07/W06 extension seams.
- [05-implementation-and-review.md](05-implementation-and-review.md) — ordered
  workflow, validation matrix, error/security/observability model, and handoff
  checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W02
plan](../../plans/p1-w02-minimal-rust-el2-runtime.md). This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W02 plan → this
design → Coding Guidelines. In particular:

- The task book requires "minimal Rust `no_std` EL2 runtime and stable idle
  state" with P1-V03/P1-V04: stack, Rust data state, boot context, panic
  route and identity established **in order**, and repeated boots reaching
  stable EL2 without hidden initial-register assumptions.
- The ADR fixes Rust-first with necessary assembly (ADR-006), `no_std`
  semantics, and controlled `unsafe` (P0-W10 governance). The entry assembly
  and the two audited state boundaries named in this design are the only
  `unsafe` this package authorizes.
- The [W01](../p1-w01-reference-boot-contract/README.md) entry contract and
  the [W09](../p1-w09-initialization-sequencing/README.md) lifecycle design
  are **accepted sibling contracts** consumed directly: W02 implements W01's
  pre-transfer tier inside its entry module verbatim, and W02-owned code
  writes the W09 `entry`/`runtime` records and invokes `run_init_sequence()`
  at the seam W09's decision 6 expects. The remaining supplying designs
  (P0 target/panic/metadata baselines, W06/W07) are assumed contracts with
  recorded failure boundaries.
- The plan's "stable idle" outcome is the terminal state of the integrated
  boot path. At W02's own point in the chain the sequencer does not exist
  yet; the wiring consequence is stated in
  [Resolved decision 6](#resolved-design-decisions-and-their-authority) and is
  not a silent placeholder.

Classification: the entry assembly and establishment sequence, the Rust entry
environment, boot-context retention, the early panic route, build identity,
the sequencer seam, and the controlled idle are **Required**. The physical
module-tree placement of the boot-path items (owned by the P0 workspace
baseline and later P1 designs), the panic-report body once W07's design lands,
the early-channel supersession by W06, and any growth of the boot stack or
context for later packages are **Reserved** with recorded triggers. General
heap/allocator, VM/vCPU, SMP, Guest execution, scheduler, platform frameworks,
console subsystem, crash-report content (W07), and real-hardware behavior are
**Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Confirm W01's entry assumptions are sufficient (work seq 1) | [Architecture](01-architecture-and-state.md) §6; [workflow](05-implementation-and-review.md) step 1 | W02 closure review (W02-DV01) |
| Establish runtime initialization conditions and ownership (work seq 2) | [Architecture](01-architecture-and-state.md) §1–§4; [entry contracts](02-code-contracts-entry-assembly.md); [runtime contracts](03-code-contracts-rust-runtime.md) | P1-V03 (W02-DV02, DV03) |
| Define the transition to stable idle and bounded failure (work seq 3) | [Runtime contracts](03-code-contracts-rust-runtime.md) §4–§5 | P1-V03 (W02-DV03) |
| Integrate build identity and the P0 panic/diagnostic baseline (work seq 4) | [Panic/identity contracts](04-code-contracts-panic-identity.md) | P1-V03 (W02-DV04) |
| Review for hidden firmware-state and future-stage dependencies (work seq 5) | [Architecture](01-architecture-and-state.md) §5; [workflow](05-implementation-and-review.md) step 5 | P1-V04 (W02-DV05) |
| Define runtime acceptance evidence and hand off the stable execution context (work seq 6) | [Workflow](05-implementation-and-review.md) §3 and §5 | P1-V04 (W02-DV06, DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): no Cargo workspace, no Rust sources, no linker script, no target
definition, and no boot-path code exist; `hypervisor/src/` and `crates/`
contain only `.gitkeep`. The P0 target baseline (P0-W03), panic
classification (P0-W14), and build metadata (P0-W16) are planned, and P0 has
implementation designs only for W01/W02. W01's design exists in this branch
and fixes the entry boundary W02 implements; W09's design fixes the tracker
and sequencer W02 calls. Everything else this runtime consumes is a planned
contract. The ledger states the foundations the outcome requires and who owns
them.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Rust `no_std` execution at EL2 is possible | No target, no workspace, no crate | The P0 target/build baseline (P0-W03) delivering an AArch64 bare-metal target, `no_std` semantics, panic/link extension points, and controlled-ASM co-build | Rust cannot execute bare-metal without the target baseline; P0 owns it | P0-W03 | W02-DV01 prerequisite review |
| Entry to that runtime is validated | W01 design fixes the tier; no entry code | W01's pre-transfer tier and rejection reporter implemented verbatim in W02's entry module | The runtime must not start in an environment W01's contract rejects | W01 (contracts); W02 (implementation) | W02-DV02; firing evidence via W11 NC1 |
| Runtime state established in order (P1-V03) | No code | The establishment sequence of [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) §3 bound to the W09 tracker events | "In order" is only reviewable when the order is explicit and recorded at each step | W02 (sequence); W09 (tracker) | W02-DV03 |
| Boot context retained | Nothing retained | `BootContext` retention per [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) §2 | W09/P2 need the DTB pointer and reserved registers after W02 returns control onward | W02 | W02-DV03 |
| Panic route exists | No panic handler anywhere | The early panic route of [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md) | `no_std` requires a panic handler to link; W09's routing matrix relies on the route from phase `runtime` onward | W02 (route); P0-W14 (classification semantics) | W02-DV04 |
| Build identity available | No metadata mechanism | `BuildIdentity` per [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md), sourced from P0-W16 | P1-V03 names identity; the panic route must self-identify | W02 (accessor); P0-W16 (metadata contract) | W02-DV04 |
| Stable idle reached on repeated boots (P1-V04) | No sequencer, no image, no runner | The sequencer seam and idle of [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) §4–§5; execution evidence via W10 | Idle is the terminal state of the integrated path; W02 alone cannot reach it without W03–W08 | W02 (seam + idle); W09 (sequencer); W10 (execution) | W02-DV06; execution deferred to W10 |
| No hidden firmware-state assumptions (P1-V04) | Nothing to audit | The self-establishment rule of [Architecture](01-architecture-and-state.md) §5 | The absence must be demonstrated by review, not asserted | W02 review procedure | W02-DV05 |

No row requires designing a later package's mechanism; the P0 dependencies are
failure boundaries, not work W02 may absorb.

## Resolved design decisions and their authority

1. **The boot entry module is one W02-owned unit that also implements W01's
   pre-transfer tier verbatim.** The entry assembly, the W01 tier and
   rejection reporter, the boot stack, and the shared UART constant live in
   one place so the single-source rule of
   [W01](../p1-w01-reference-boot-contract/02-entry-validation-contracts.md)
   §6 is structural rather than aspirational. Authority: W01 contract §1
   artifact table; ADR-006 (necessary assembly).
2. **W02 adds no lifecycle state owner.** Boot position is exclusively W09's
   tracker; W02-owned code records the `entry` and `runtime` events and reads
   nothing else. W02's own establishment is a fixed straight-line sequence,
   not a state machine. Authority: W09 decision 5 (single owner) and
   decision 6 (delegated records).
3. **Establishment discipline: the entry establishes every machine state the
   runtime needs and relies on none it cannot establish.** Concretely: stack
   selection and top, SPSel=1, DAIF all-masked, BSS zeroed, and (via W01's
   tier) the entry privilege. Firmware residue in control registers is not
   relied upon — W04 owns removing it, W05 owns vectors, W08 owns the MMU.
   Authority: plan goal ("without depending on accidental firmware register
   contents"); task book P1-V04.
4. **The early panic route is W02-owned in P1 and bounded by construction.**
   The `#[panic_handler]` is minimal (identity, message if present, bounded
   stop), emits through the early diagnostic writer, never allocates, never
   re-enters, and has a single-entry guard. W07's accepted design supersedes
   the report body through the recorded extension seam; ownership of the
   handler itself moves only through that recorded design change. Authority:
   W09 routing matrix (`runtime` row and all later panic-route rows); P0-W14
   classification semantics.
5. **`PhysAddr` is a minimal W02-owned newtype.** The runtime must retain the
   DTB pointer, and the Coding Guidelines forbid naked integers for
   addresses; P0-W15 fixes the semantic red lines but explicitly defers the
   type API. W02 defines the one type its contract needs, in the boot-path
   module, flagged for later generalization. Authority: Coding Guidelines
   newtype rule; P0-W15 handoff; stage-local design freedom.
6. **Sequencer seam: after recording `runtime` completion, the runtime calls
   W09's `run_init_sequence()`; on normal return, W02-owned glue records
   `stable` and enters the controlled idle.** This is exactly the seam W09's
   decision 6 expects. Consequence for W02's own evidence: until W09's item
   exists the call site does not compile, so W02's boot-reachability evidence
   (P1-V04) is executed on the integrated path via W10, while W02's own
   validation is the ordered-establishment reviews (P1-V03) plus host-side
   evidence where the P0 baseline permits. No stub sequencer, no temporary
   direct-to-idle wiring, and no placeholder phase is authorized — a silently
   shortened lifecycle would fabricate stable-state success. Authority: W09
   decision 6 and §8; P1-W02 plan acceptance.
7. **The controlled idle is `wfi`-loop with interrupts architecturally
   masked, entered only after `stable` is recorded.** WFI may complete on
   masked pending interrupts, so the loop re-enters rather than races;
   nothing wakes the runtime into new work because P1 owns no wake-up
   consumer. The stable-marker emission point is W09's/W10's, not W02's.
   Authority: W09 state machine T5; W10 marker protocol.
8. **Static boot stack, fixed size, single constant.** 64 KiB for the boot
   CPU's entire P1 lifetime (no growth path, no secondary stacks), 16-byte
   aligned, defined once in the boot-path module. Resizing is a reviewed
   change with recorded arithmetic. Authority: stage-local design freedom;
   plan scope ("execution stack").

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for the
   module map, state ownership, establishment order, concurrency model, and
   the assumed-contract table. Every implementation step depends on it.
2. Load [02-code-contracts-entry-assembly.md](02-code-contracts-entry-assembly.md)
   for the entry work, [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md)
   for the Rust runtime and seams, and
   [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md)
   for the panic route and identity.
3. Execute the steps in the order given in
   [05-implementation-and-review.md](05-implementation-and-review.md):
   prerequisite confirmation, entry assembly, Rust entry/context,
   panic/identity, sequencer-seam wiring, then the reviews.
4. Record implementation decisions and deviations in
   `../p1-w02-minimal-rust-el2-runtime-record.md` when implementation begins,
   and validation commands, environments, and outcomes in
   `../../verification/p1-w02-minimal-rust-el2-runtime-verification.md` when
   evidence exists. Neither file may exist yet, and neither this design nor a
   record may claim W02 complete.

## Explicitly excluded interfaces

No capability report, EL2 control write, vector table, console abstraction,
crash-report content, page-table or MMU mechanism, allocator, or heap is
designed or authorized; each belongs to its owning package. No public ABI,
wire format, or persistent layout is introduced — all W02-owned symbols are
internal boot-scope items. No scheduler, SMP, PSCI, GIC, or timer-virtualization
mechanism is touched. No second lifecycle owner, global manager object, or
convenience singleton is authorized; W09's tracker is the only boot-position
state and the W01 rejection reporter is the only pre-transfer output path.

## Downstream handoff

- **[P1-W03](../p1-w03-aarch64-capability-inventory/README.md)** receives a
  live Rust execution context (established stack, static data, panic route,
  identity), the `BootContext` with the retained DTB pointer, and the EL
  entry guarantee as its inventory precondition.
- **[P1-W04](../p1-w04-el2-architectural-state-baseline/README.md)** receives
  the execution-state facts it asserts (SPSel=1, DAIF masked) as
  single-owner-established state it must not rewrite.
- **[P1-W05](../p1-w05-el2-exception-entry-baseline/README.md)** receives a
  runtime whose panics are already routed and whose DAIF discipline is fixed;
  vector installation is free to proceed at W04's baseline.
- **[P1-W06](../p1-w06-early-console-logging/README.md)** receives the early
  diagnostic writer as the pre-channel output primitive its channel
  supersedes for markers (extension seam in
  [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md)
  §5), and the rule that the panic path keeps a channel-independent route.
- **[P1-W07](../p1-w07-fatal-crash-diagnostics/README.md)** receives the
  panic-handler ownership seam: the report body is replaceable per its
  design; the handler registration and the single-entry guard discipline
  transfer only through its accepted design.
- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** receives the
  complete inventory of runtime regions it must map (boot stack, BSS, static
  data, boot context, early writer MMIO) as the input named by its plan work
  seq 1.
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  seam it already assumes (decision 6): the delegated `entry`/`runtime`
  records, the `run_init_sequence()` call point, and the `stable` glue.
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** and
  **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receive the
  panic-marker class source (the early panic route's output) and the NC4
  panic invocation target.
- **P2** receives the stable execution context and the retained boot
  parameters; discovery and allocation remain P2 scope.

A coding agent completing W02 must leave the handoff checklist in
[05-implementation-and-review.md](05-implementation-and-review.md) answerable
without inspecting W02 source code.
