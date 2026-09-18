# P1-W06 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

W06 can start only after the accepted W02/W09 contracts are available as
designs and the W05 vectors seam is settled, because the phase position,
replay trigger, and exception-context premise are consumed, not invented.
Before changing any file, the implementer verifies the mandatory reading
(parent README), inspects the current tree (`git ls-files`; confirm the P0
workspace/target state and the accepted sibling designs), and records the
assumed-contract states from
[01-architecture-and-state.md](01-architecture-and-state.md) §7.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the reference platform's PL011 behavior cannot be confirmed for the
  minimal init sequence (register read-back mismatch in early exploration)
  — record the blocker; do not widen the init into a driver;
- a producer seam (W03 emit, W07 renderer) cannot accept the §2 C4
  verbatim-transport rule — raise the W03/W06 or W07/W06 coordination
  issue; do not re-render content here;
- closure appears to require probing, discovery, a level engine, a
  framework, or a second transport — Out of Scope (parent README); stop;
- the P0 logging baseline (P0-W12) has landed with semantics that
  contradict a role boundary of this channel — record the upstream
  conflict; the channel does not invent substitute semantics.

## 2. Ordered implementation steps

### Step 1 — confirm P0 contract usage and prerequisite seams

Target: implementation record
(`../p1-w06-early-console-logging-record.md`, created in this step).

Work: verify the P0-W12 role boundaries and identity association against
the channel's category model; verify W09's `console_step` shape, replay
trigger, and marker vocabulary; verify W02's writer/fallback and
`BuildIdentity` contracts; record the reference-console fact (address,
size, minimal init source) and any deferred seam.

Suggested observation: read the sibling designs and P0 plans; no
repository change.

**Acceptance:** the record states each seam satisfied, or names the gap
and its owner.  
**Failure/blocker:** a seam gap stops the affected step (fail-closed); no
local adaptation.

### Step 2 — implement the transport and availability signal

Target: the console module (physical placement per the P0 workspace
baseline; recorded).

Work: implement `console_init`, `console_write_bytes`, `console_write_line`,
`channel_available`, the start line, and the audited MMIO boundary per
[02-code-contracts-channel.md](02-code-contracts-channel.md) §1–§4.

Suggested observation: review against the contract; where the tree links,
a boot through the phase body is the transport's natural first observation
(deferred to integrated evidence per §3).

**Acceptance:** init is minimal and read-back-verified; writes are polled,
bounded, and drop-free; the signal is set exactly once.  
**Failure/blocker:** an init read-back mismatch routes as designed; a
transport redesign request stops the step.

### Step 3 — implement marker formatting and content transport

Target: the console module.

Work: implement `format_marker`, `transport_line`, and the producer
seams per
[02-code-contracts-channel.md](02-code-contracts-channel.md) §5 and
[03-code-contracts-integration.md](03-code-contracts-integration.md) §2,
including the truncation rule and the fixed emission points.

**Acceptance:** marker lines render W09's vocabulary exactly; producer
lines move verbatim; no fourth output path exists.  
**Failure/blocker:** a vocabulary or framing conflict is raised to the
owning package; silent rewording is prohibited.

### Step 4 — wire the integration seams and the mapping requirement

Target: the console module plus the recorded contract points in the
consumer designs' wiring (owned by those packages).

Work: confirm the W09 replay trigger reads `channel_available`; confirm
the W03 render wiring point; confirm W07's transport preference order
(channel first, W02 writer fallback); publish `CONSOLE_REGION` /
`CONSOLE_REQUIRED_ATTRIBUTES` for W08; verify the exception-context
callability premises against
[03-code-contracts-integration.md](03-code-contracts-integration.md) §1.

**Acceptance:** every seam is wired at its contracted point; the mapping
requirement is stated; no channel-owned code changes another package's
mechanism.  
**Failure/blocker:** a missing consumer design records the deferred link
with its owner (the W02 decision-6 precedent); no stub wiring.

### Step 5 — reference-assumption and output-path review

Target: implementation record; this design's review tables.

Work: run the prohibited-content walk
([integration](03-code-contracts-integration.md) §5), the output-path
inventory (architecture §3 rule), the temporal single-consumer walk
(architecture §3), and the replacement-boundary check (consumers reference
the API, not the constant).

**Acceptance:** every rule passes with a pointer to code or record; any
failure is a recorded P1-V10/P1-V15 finding.  
**Failure/blocker:** an unremovable violation stops the package.

### Step 6 — acceptance evidence and closure

Target: verification record
(`../../verification/p1-w06-early-console-logging-verification.md`).

Work: perform the executable reviews of §3; record the deferred executions
(P1-V10's entry-to-stable marker observation and post-MMU availability →
W10 R1/R6; fault-scenario output classes → W11) with their owning
packages; complete the handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W06-DV01 → W06 closure | P0-contract conformance review | map the category model and start line against P0-W12/P0-W16 semantics; map the fatal posture against P0-W14 | roles not substituted; identity associated or degraded recorded; classification posture consistent | semantic conformance; not that the P0 baselines are implemented |
| W06-DV02 → P1-V10 | Channel and format review | inspect §1–§5 of [channel contracts](02-code-contracts-channel.md) implementation | init verified; signal once; marker format exact; categories closed; truncation-safe; bounded formatter | the channel exists and frames correctly as designed; not integrated boot output |
| W06-DV03 → P1-V10 | Integration and transition review | inspect §1–§4 of [integration](03-code-contracts-integration.md) wiring and the mapping requirement | emission points contracted; producer obligations recorded; W08 requirement published; exception-context premises hold | the channel serves all producers and survives the transition by contract; not post-MMU liveness itself |
| W06-DV04 → P1-V10/P1-V15 | Reference-assumption and output-path review | §5 walk; search the tree for UART literals and print paths | assumption documented once; exactly the recorded four-path output set; consumers API-bound | no hidden platform dependency or path proliferation; not hardware behavior |
| W06-DV05 → P1-V10 | Entry-to-stable availability (deferred) | W10 R1/R6 on the integrated path: capture contains replay + live markers through `stable`, and post-transition markers | diagnostics identify every phase from entry to stable and continue post-MMU | the acceptance wording, executed; run by W10, not by W06 |
| W06-DV06 → W06 closure | Consumability review | read the outputs as W07 (transport + framing), W08 (requirement), W09 (signal + points), W10 (classes), W11 (fault-output classes), W12 (assumption + limitations) | each consumer can act without inventing W06 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The
boot-dependent proofs (W06-DV05, fault-output classes) are deferred to
W10/W11 by contracted wiring, not omitted; until they exist, P1-V10's
executed half is unproven and no W06 artifact may report otherwise. No
validation here proves P1-V11 through P1-V21.

## 4. Error, security, and observability model

**Errors.** W06 has exactly two failure postures: init read-back mismatch
(`ConsoleError` → the W09 matrix `console` row: best-effort panic-route
report, then bounded stop) and transmitter stall (no software bound; the
harness timeout is the bound — the recorded limitation). The channel adds
no retry, no fallback work, no silent drop. Output failure during a fatal
report is contained by the transports' contracts (early writer's
architectural poll semantics; the guard prevents re-entry).

**Security.** The channel adds no authorization decision and creates no new
input surface: it writes caller-supplied bounded content only, never reads
device state beyond the polled flags and init read-back, and never prints
content derived from untrusted sources (P1 has none; the rule is recorded
for future reuse). Layering posture: the reference constant stays in the
bring-up scope; generic consumers reference the API; no platform-name
branch exists anywhere in the path (ADR-043/044). `unsafe` is confined to
the audited MMIO accessor pair with `SAFETY` justifications in the P0
unsafe inventory.

**Observability.** The channel is the stage's primary observable: phase
markers (replay + live), capability content, the start line with build
identity, and (through W07) the fatal/panic report classes. It carries no
counters, metrics, or trace events — those remain P0-W13/P2 scope; the
boundary is recorded, not pre-built.

## 5. Handoff checklist

Before handing W06 to a reviewer, provide:

- the exact changed-file list and the module locations of every contracted
  item;
- the assumed-contract table as observed
  ([01-architecture-and-state.md](01-architecture-and-state.md) §7),
  including any recorded blocker or seam deviation;
- W06-DV01..DV06 evidence paths and run status, including the explicit
  deferred/not-run entries (integrated marker evidence → W10; fault-output
  classes → W11);
- the implementation-selected values: reference-console address/window,
  minimal init sequence and its source, prefix literals, channel maximum
  line length with sizing arithmetic;
- confirmation that all new `unsafe` is confined to the audited MMIO
  accessors with `SAFETY` justifications filed in the P0 unsafe inventory
  process;
- confirmation that no probing, framework, level engine, second
  transport, interrupt enable, or public ABI was introduced;
- open items: W07 transport-preference consumption, W08 mapping-class
  confirmation, W03 wiring point confirmation — recorded, not resolved
  here.
