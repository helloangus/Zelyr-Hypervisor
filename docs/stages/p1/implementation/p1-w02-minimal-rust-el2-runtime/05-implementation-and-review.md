# P1-W02 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

W02 can start only after the P0 target/build baseline exists in
implementable form, because every step compiles against it. Before changing
any file, the implementer verifies the mandatory reading (parent README),
inspects the current tree (`git ls-files`; confirm the P0 workspace/target
state and the accepted W01/W09 designs), and records the assumed-contract
states from [01-architecture-and-state.md](01-architecture-and-state.md) §6.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P0 target/build baseline is absent or cannot host `no_std` +
  controlled-ASM co-build — record the upstream defect; do not invent a
  target, linker script, or build hack;
- the W01 tier contracts or the W09 tracker/sequencer contracts are absent
  or contradict this design's seams — raise the conflict to both owners
  (W09 §1 rule); do not adapt silently;
- closure appears to require a heap, a console, capability or baseline
  mechanisms, vectors, or MMU work — those are Out of Scope (parent README);
  stop;
- a contract clause cannot be implemented as written (toolchain limitation,
  P0 gap) — record the design conflict; local variants are prohibited.

## 2. Ordered implementation steps

### Step 1 — confirm W01's entry assumptions are sufficient

Target: implementation record (`../p1-w02-minimal-rust-el2-runtime-record.md`,
created in this step).

Work: verify the [W01 entry-state table](../p1-w01-reference-boot-contract/01-boot-contract.md)
§3 covers everything stages 1–9 need; verify the tier, reporter, and
transfer contracts of
[W01's validation file](../p1-w01-reference-boot-contract/02-entry-validation-contracts.md)
are implementable in the entry module; record any insufficiency as the
coordination issue it is.

Suggested observation: read the W01 design; no repository change.

**Acceptance:** the record states "sufficient" with a check per establishment
stage, or names the gap.  
**Failure/blocker:** an insufficiency stops this step (the runtime would
otherwise encode a hidden firmware assumption — exactly what P1-V04 forbids).

### Step 2 — implement the entry assembly

Target: the boot entry module (physical placement per the P0 workspace
baseline; recorded).

Work: implement `p1_el2_entry`, the W01 tier verbatim, the rejection
reporter, the boot stack, BSS clearing, and the transfer exactly per
[02-code-contracts-entry-assembly.md](02-code-contracts-entry-assembly.md),
including §4's prohibited-content list.

Suggested observation: review of the assembly source against the contract;
link-level inspection of the entry symbol and stack placement only where
the tree links (i.e., once step 5 completes with W09's items) and the P0
artifact baseline provides tooling.

**Acceptance:** stages 1–4 of the establishment order are present, ordered,
and free of prohibited content; the W01 single-source constant rule holds.  
**Failure/blocker:** a linker/extension-point gap is the recorded P0 blocker
(§1); no private layout fork.

### Step 3 — implement the Rust entry, context, and establishment

Target: the boot-path runtime module.

Work: implement `el2_rust_entry`, `PhysAddr`, `BootContext` with its audited
once-publication boundary, and the readiness steps per
[03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) §1–§3,
including the W09 tracker records at their contracted points.

Suggested observation: host-side unit evidence of `BootContext` mapping and
the publication guard where the P0 host-test baseline permits; otherwise
contract inspection.

**Acceptance:** stages 5–9 are implemented in the contracted order with no
additional lifecycle state; every `unsafe` block carries its `SAFETY`
justification for the P0 unsafe inventory.  
**Failure/blocker:** a tracker-seam mismatch stops the step (fail-closed, per
W09's adapter discipline); no stub records.

### Step 4 — implement the panic route and build identity

Target: the panic/identity module.

Work: implement `p1_panic`, the guard, `BuildIdentity`, and
`early_write_bytes` per
[04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md);
wire the identity source from the P0-W16 mechanism as it exists.

**Acceptance:** the route is bounded by construction (capacity recorded,
single-entry guard, terminal stop); identity resolves or degrades to the
recorded unavailable literal; no second output path exists.  
**Failure/blocker:** a P0-W16 gap degrades identity and records the blocker;
it does not stop the panic route.

### Step 5 — wire the sequencer seam

Target: the seam glue in the runtime module.

Work: add the `run_init_sequence()` call, the `stable` glue, and the idle
entry per
[03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md) §4–§5.
If W09's items do not exist yet, this step records the deferred link and
leaves the call site uncompiled — that is the contracted state, not a
failure; the deferred link and its owner are named in the implementation
record.

**Acceptance:** the seam matches W09 decision 6 point-for-point; no stub, no
temporary wiring, no placeholder phase.  
**Failure/blocker:** a seam conflict is raised to W09 per §1; silent
adaptation is prohibited.

### Step 6 — hidden-dependency and order review

Target: implementation record; this design's review tables.

Work: run [01-architecture-and-state.md](01-architecture-and-state.md) §5
(H1–H4) and §3 (ownership register) over the implemented tree; walk §2's
prohibited-content list; confirm the establishment order of §2 line-by-line
against the code.

**Acceptance:** every rule passes with a pointer to code or record; any
failure is a recorded P1-V04 finding.  
**Failure/blocker:** an unremovable hidden dependency stops the package.

### Step 7 — acceptance evidence and closure

Target: verification record
(`../../verification/p1-w02-minimal-rust-el2-runtime-verification.md`).

Work: perform the executable reviews of §3; record the deferred executions
(P1-V04's repeated-boot proof) with their W10 ownership, exactly as W09
handles its deferred boot evidence; complete the handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W02-DV01 → W02 closure | Prerequisite sufficiency review | step 1's checks against W01/P0/W09 sources | every establishment stage's prerequisites declared and satisfied or owned elsewhere | readiness to build; not that prerequisites are implemented |
| W02-DV02 → P1-V03 | Entry and establishment review | inspect entry module against [02-code-contracts-entry-assembly.md](02-code-contracts-entry-assembly.md) (incl. §4 list) | tier verbatim; stack/SPSel/DAIF/BSS ordered; single stack constant; no prohibited content | the entry establishes what it must; not firmware behavior on real hardware |
| W02-DV03 → P1-V03 | Ordered-establishment review | map code to [01-architecture-and-state.md](01-architecture-and-state.md) §2 stages 1–12; check tracker records at stages 5–6, 9–11 | order matches; records at contracted points; single lifecycle owner; context published once | in-order establishment as designed; not boot execution |
| W02-DV04 → P1-V03 | Panic/identity review | inspect [04-code-contracts-panic-identity.md](04-code-contracts-panic-identity.md) implementation | handler bounded by construction; guard; terminal stop; identity resolves or degrades recorded; no second output path | the route exists and is bounded by design; not its behavior under a real panic (NC4, W11) |
| W02-DV05 → P1-V04 | Hidden-dependency review | H1–H4 walk; every register read/memory access checked against W01 §3 | no reliance on unestablished firmware state; no control-register reads; no future-stage reach | absence of hidden assumptions as designed; not runtime behavior |
| W02-DV06 → P1-V04 | Repeat-boot evidence (deferred) | W10 regression on the integrated path after W09 lands | repeated boots reach `stable` with identical phase sequence and token set | the runtime reaches stable EL2 repeatably; executed by W10, not by W02 |
| W02-DV07 → W02 closure | Consumability review | read the outputs as W03 (live context? identity? panic route?), W04 (asserted facts single-owned?), W07 (panic seam clear?), W08 (region inventory complete?), W09 (seam exact?) | each consumer can act without inventing W02 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The two
QEMU-dependent proofs (W02-DV06, and any boot-observed panic evidence) are
deferred to W10/W11 by contracted wiring, not omitted; until they exist,
P1-V04's executed half is unproven and no W02 artifact may report otherwise.
No validation here proves P1-V05 through P1-V21.

## 4. Error, security, and observability model

**Errors.** The runtime has exactly three failure postures: W01's
pre-transfer rejections (owned by W01), invariant violations during
establishment (routed via the panic route, phase-attributed by the tracker),
and sequencer failures (owned by W09's routes). There is no retry, no
degraded mode, and no partial-success state. The panic route itself is
termination-only and single-entry.

**Security.** The entry validation is the stage's trust boundary; W02 adds
no authorization decision. Security-relevant postures preserved: no
execution before the tier accepts; no FP/SIMD anywhere in the boot path
(binding W04's baseline); `unsafe` confined to three audited boundaries
(W09's tracker CAS — consumed; the once-publication of `BootContext`; the
panic-entry guard), each with a `SAFETY` justification in the P0 unsafe
inventory; no echo of untrusted content in any output.

**Observability.** The observable surface is: the tracker events (rendered
by W06's channel when available), the panic route's bounded report through
the early writer, and the `stable` emission point owned by W09/W10. W02 adds
no debug prints, counters, or telemetry; structured observability remains
P0/P2 scope.

## 5. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list and the module locations of every contracted
  item;
- the assumed-contract table as observed
  ([01-architecture-and-state.md](01-architecture-and-state.md) §6), including
  any recorded blocker or seam deviation;
- W02-DV01..DV07 evidence paths and run status, including the explicit
  deferred/not-run entries (integrated-path boots → W10; panic-class
  execution → W11 NC4);
- the implementation-selected values: stack sizing arithmetic, report/token
  literals, identity degradation literal, entry-point/linker placements;
- confirmation that all new `unsafe` is confined to the named audited
  boundaries with `SAFETY` justifications filed in the P0 unsafe inventory
  process;
- confirmation that no allocator, console, capability/baseline mechanism,
  vector table, MMU work, second lifecycle owner, or public ABI was
  introduced;
- open items: the deferred sequencer link (W09), the panic-body extension
  (W07), marker-channel supersession (W06), region inventory consumption
  (W08) — recorded, not resolved here.
