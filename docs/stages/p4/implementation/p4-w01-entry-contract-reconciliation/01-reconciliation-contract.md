# P4-W01 Reconciliation Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W01 detailed design](README.md).

## 1. Logical artifact groups and ownership

W01 is a review package, so its logical modules are authoritative review
artifact groups, not Rust modules. The named location is the sole
authoritative home for that content; other documents may link to it but must
not duplicate or contradict it.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Prerequisite inspection list | W01 implementation record (created when the review runs; future path `../p4-w01-entry-contract-reconciliation-record.md`) | ADR, P4 task book, plan index, P0–P3 task books/plans, all existing implementation and verification records | one row per inspected source with revision and date; it does not restate upstream contract content |
| Reconciliation matrix | W01 implementation record, structured per §2 of this file | the inspection list | per-prerequisite evidence/consumer/block status; it does not redesign upstream packages |
| P4 entry-assumption ledger | W01 implementation record, structured per §3 of this file | ADR invariants and the reconciliation matrix | the constraints W01 binds for P4; it does not create new architecture |
| Gap and conflict list | W01 implementation record, structured per §4 of this file | matrix rows and ADR/task-book comparison | labeled gaps (`Blocked prerequisite`, `Architecture Change Request`, `ADR Required`); it never resolves a labeled item |
| Entry-review verification evidence | `../../verification/p4-w01-entry-contract-reconciliation-verification.md` (created when evidence exists) | actual review execution | P4-V01 run/not-run status per the validation matrix; not part of the design |

## 2. Reconciliation matrix

This is the required structure of the matrix the review executes. Each row is
one prerequisite **surface** that a named P4 package consumes. The reviewer
fills the four status columns from the tree as it exists on the review date:
`contract state` (tracked document that states the behavior), `evidence
state` (verification record with real evidence), `assumed status`
(`delivered` / `assumed: planned` / `absent`), and `gap label` (§4) if any.

| # | Prerequisite surface | Upstream owner and contract location | Evidence expected | P4 consumer | Compatibility requirement for P4 |
|---|---|---|---|---|---|
| R01 | Stable Non-secure EL2 Rust runtime and boot contract | P1-W01/P1-W02; documented by [P1-W12](../../../p1/plans/p1-w12-p1-documentation-handoff.md) (boot contract, EL2 init contract) | P1-V01–V04 evidence in `docs/stages/p1/verification/` | W04 | P4 enters the Guest from the documented stable EL2 environment; P4 does not re-derive boot entry |
| R02 | EL2 exception vectors, syndrome capture, fatal boundary | P1-W05/P1-W07; P1-V08, P1-V09, P1-V11, P1-V12 | exception-diagnostic contract + evidence | W04, W06 | Guest exits land in an EL2 path that captures ESR/ELR/FAR-class context; P4 extends, not replaces, the vector baseline |
| R03 | EL2 architectural-state baseline (routing, traps, timer access, EL1/EL0 preparation) | P1-W04; P1-V07 | EL2 baseline contract + evidence | W04 | Guest-entry control-register programming starts from the documented EL2 baseline |
| R04 | Host Stage-1 address space and post-MMU environment | P1-W08; P1-V13, P1-V14 | host address-space description | W02 | Host mapping attributes are known; Stage-2 work must not invalidate the Host Stage-1 contract |
| R05 | Ordered initialization lifecycle and stable idle | P1-W09; P1-V15 | lifecycle contract | W04 | Guest entry is reached through the declared stable runtime state |
| R06 | Reference QEMU environment and boot regression automation | P1-W10; P1-V16, P1-V17; P0 QEMU runner plan (P0-W09) | automation contract + evidence | W05, W08 | P4 automation consumes the existing runner boundary; QEMU stays the reference environment, not an architecture definition |
| R07 | Platform discovery → normalized platform facts (CPU topology, capabilities, console, timer, PSCI facts) | P2-W02; P2-V03, P2-V04 | PlatformInfo contract + evidence | W02, W03 | P4 reads normalized facts/capabilities, never reparses DTB and never branches on board/QEMU names |
| R08 | Normalized boot physical-memory map with protected ranges | P2-W03; P2-V05, P2-V10 | boot-map contract + evidence | W02, W03 | Guest RAM and Stage-2 tables derive only from the authoritative allocatable/protected classification |
| R09 | Safe physical-page allocation/free with accounting | P2-W04; P2-V06, P2-V10 | allocation contract + evidence | W02, W03 | Stage-2 pages and Guest RAM come only from this allocator; zero protected-page returns is inherited as a hard invariant |
| R10 | Dynamic small-object allocation | P2-W05; P2-V07 | allocation contract + evidence | W02, W04 | Guest/vCPU control structures may use dynamic allocation with explicit failure handling |
| R11 | Platform/memory inspection facts | P2-W06; P2-V08 | inspection contract | W06, W07 | Diagnostics may reference inspection outputs without treating inspection as a control API |
| R12 | P2 consumer contract, limitations, and P2-ACR-01 visibility | [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md); P2-V12, P2-V13 | P3/P4 handoff record | all P4 packages | P4 consumes the stated inputs/limitations; **P2-ACR-01 stays visible and unresolved** (see §4) |
| R13 | Logical pCPU identity, physical-CPU lifecycle, online set | P3-W01–W03; consolidated by [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md); P3-V01–V03 | SMP handoff record | W02, W04 | P4 binds a Guest to an online pCPU's identity without equating pCPU with vCPU and without a permanent boot-CPU assumption |
| R14 | Per-CPU local state foundation (stack, identity, exception-local state, current-vCPU reservation) | P3-W04; P3-V04, P3-V09 | SMP handoff record | W04 | The world-switch discovers per-CPU context through the P3-declared mechanism; no implicit global current-CPU state |
| R15 | Shared-state synchronization semantics and lock ordering | P3-W06; P3-V06, P3-V10 | SMP handoff record | W02, W04 | Stage-2 mutation and vCPU state obey the documented atomic/lock/IRQ rules |
| R16 | Cross-CPU notification primitive | P3-W07; P3-V07 | SMP handoff record | W04 (reserved use) | P4 must not preclude multi-pCPU handling; P4 itself uses the primitive only within its designed scope |
| R17 | TLB shootdown transport (request/ack/completion, no TLBI semantics) | P3-W08; P3-V08 | SMP handoff record + explicit semantic gap | W02 | W02 consumes transport **only** and designs the Stage-2 invalidation operation itself; P4 proves current-path consistency, not cross-pCPU shootdown |
| R18 | Toolchain, AArch64 target, build and quality gates | P0-W02/W03/W07 and related P0 packages; P0-V02+ | P0 verification records (`docs/stages/p0/verification/`) | W05, W08 | Guest and Hypervisor build under the pinned toolchain/target baseline; no unstable-feature adoption |
| R19 | Address/identifier newtypes and error/panic classification | P0-W15, P0-W14; P0-V IDs of those packages | P0 records | W02, W03, W04 | GPA/IPA/GVA/HPA/HVA stay semantic newtypes; guest-caused faults stay recoverable VM-facing errors while hypervisor invariants are fatal |
| R20 | Logging/trace-event and unsafe governance | P0-W10/W12/W13 | P0 records | all P4 packages | P4 diagnostics route through the established logging/trace namespace; every new `unsafe` is inventoried with SAFETY justifications |
| R21 | P4-side scenario/telemetry/automation inputs from sibling designs | W05 (scenario markers), W06 (exit classification), W07 (repeat/event expectations) | the sibling detailed designs `../p4-w05-validation-guest/README.md`, `../p4-w06-fault-isolation-diagnostics/README.md`, `../p4-w07-repeatability-telemetry/README.md` | W08, W09 | Alignment rows only: W01 records that these inputs are planned and names their design locations; their content is not reconciled here |

Rules for completing the matrix:

- One row per surface; a P4 package may appear in many rows, a row names at
  least one consumer.
- `contract state` cites a tracked document path, not a summary from memory.
- `evidence state` cites a verification record path or records `absent`.
- A row whose contract exists only as an approved plan records
  `assumed: planned`; consuming packages then inherit the §4 failure
  boundary. A row with neither contract nor plan records `absent`.
- R01–R20 are the minimum row set; the reviewer adds rows when a P4 detailed
  design names a dependency this table missed (that addition is a W01 record
  update, not a redesign).

## 3. P4 entry-assumption ledger (what W01 re-fixes for P4)

These items are not owned by any P0–P3 contract. W01 re-fixes them as P4
entry constraints so that W02–W09 designs share one interpretation. Each item
cites its authority; if a later P4 design must deviate, it re-owns the item
explicitly in its own resolved-decisions list and the deviation is reviewed
against the same authority. W01 does not design mechanisms here; it fixes
interpretation and discipline only.

| ID | Assumption bound for P4 | Authority basis | Failure boundary if violated |
|---|---|---|---|
| A1 | **Guest address terminology:** Guest-visible physical addresses are GPA/IPA (one space for the P4 Guest); host physical addresses are HPA; Guest virtual addresses are GVA. P4 documents its temporary layout in these terms and never in QEMU-internal terms. | ADR §13 (newtype address vocabulary), ADR-018 | layout or mapping records that conflate address classes fail P4 review |
| A2 | **Guest-untrusted boundary:** all Guest-caused addresses, lengths, and faults are untrusted input; a Guest-caused fault is a recoverable, VM-facing classified event, never a Host panic; only hypervisor invariant violations are fatal, per the P0 failure-classification contract (R19). | ADR-007, ADR §19 invariants; P0-W14 | any design that panics the hypervisor on Guest input fails review |
| A3 | **Physical CPU / vCPU separation:** P4 may bind one vCPU to one online pCPU for execution, but pCPU and vCPU remain distinct objects; nothing may encode "the Guest owns the machine." | Plan Agent guardrails; P3-W03 handoff (R13) | any design merging pCPU and vCPU state fails review |
| A4 | **Temporary layout discipline:** every Guest-visible constant (RAM placement, entry point, console access) is a P4 test convention recorded as an implemented fact; none may be presented as the `rusthv-arm-virt-v1` machine ABI (owned by P8). | P4 task book §1 Reserved, §8 | any P4 record freezing layout as machine ABI fails the P4-V16 review |
| A5 | **No permanent single-pCPU / single-VM architecture:** one Guest and one vCPU are P4 test scope; data structures must not assume the Guest is the only possible one where a bounded generalization is free (for example per-object rather than global state), without implementing P5+ policy. | P4 task book §1 Reserved; ADR-015 | a design that hardcodes a global singleton Guest fails review |
| A6 | **Current-path consistency scope:** P4 proves Stage-2 consistency for the current execution path only; cross-pCPU shootdown is Reserved and consumes the P3 transport seam (R17) without implementing final policy. | P4 task book §1 Reserved, P4-A07 | a claim of cross-pCPU shootdown proof fails P4-V07/V08 review |
| A7 | **Test-environment scope:** QEMU `virt` is the reference validation environment; QEMU-observed behavior never defines Core semantics; divergences from AArch64 architectural expectations are recorded as Specification Investigation items, not silently encoded. | P4 task book §8; ADR-003, ADR-052 | any QEMU-name branch or QEMU-defined semantic in Core fails review |
| A8 | **Telemetry and evidence conventions:** P4 events route through the established logging/trace baseline (R20) with stage-local event names; verification claims live only in `docs/stages/p4/verification/`. | ADR-048; P4 task book §3 | free-form prints outside the baseline or evidence in design documents fail review |
| A9 | **Scenario-set authority:** the mandatory/planned Validation Guest scenario split (VG-001–VG-012) is taken from the P4 task book §6; the canonical per-scenario definition text from the superseded root source task book is **not tracked in this repository** (verified against the full git history). The W05 design owns the P4-local scenario definitions and must label them as this design's reconstruction pending stage-owner confirmation. | P4 task book §6; P4-W05 plan scope | a silent claim that untracked source definitions are authoritative fails review; recorded as an open question in the W01 gap list |

Item A9 is a documentation-provenance finding, not an architecture conflict:
the P4 task book is the governing source for the mandatory/planned split, and
the reconstruction rule keeps P4 honest about what was and was not inherited.

## 4. Gap classification rules and failure boundaries

Every matrix row that is not `delivered` receives exactly one classification:

- **`Blocked prerequisite`** — the upstream contract exists (as approved plan
  or delivered record) but the evidence P4 needs is absent, or the contract is
  silent on a behavior P4 needs. Handling: P4 consumers may design against the
  contract as an assumed contract, but any P4 acceptance whose evidence
  depends on the missing behavior stays not run until the upstream evidence
  exists. The gap names the upstream owner; P4 must not repair P0–P3 scope.
- **`Architecture Change Request`** — a delivered upstream contract
  contradicts a P4 requirement at the task-book/plan level. Handling: stop the
  affected P4 decision; record the conflict with both source citations; the
  conflict is resolved by the stage owners, never by a P4 local choice.
- **`ADR Required`** — the conflict or gap reaches the architecture baseline
  itself (for example a required object model the ADR leaves deferred).
  Handling: same as above, with an ADR proposal as the resolution path.

Standing item: **P2-ACR-01** (`ADR Required`, recorded in the P2 task book §3)
conflicts the ADR P2 roadmap bullet ("define minimal `MemoryObject` /
`MemoryRegion` structures") with the P2 task book's prohibition on designing
P4's object system. W02 and W03 touch this area. W01's duty is to keep
P2-ACR-01 visible in the gap list and in the W02/W03 consumer rows; neither
W01 nor the P4 designs may resolve it. The P4 designs therefore define
stage-local memory handles without claiming to establish the ADR-level
`MemoryObject`/`MemoryRegion` model.

Failure-boundary statement inherited by every consumer: if a row later
delivers differently than the assumed contract (different mechanism name,
different ownership rule, different invalidation seam), the consuming P4
design's assumptions become recorded conflicts — the consumer stops and
reconciles through a W01 matrix update; it does not adapt silently.

## 5. Explicitly excluded interfaces

There are no Rust structs, enums, traits, functions, modules, crates, Cargo
manifests, target triples, shell scripts, CI workflow files, QEMU
configuration, or public APIs in this design, and none are authorized. W01
adds no runtime state and mutates no upstream document. If executing this
review appears to require any of those, that requirement is itself a gap to
classify per §4, not a license to widen W01 scope.
