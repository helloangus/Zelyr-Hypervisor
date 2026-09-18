# P0 Handoff Map

**Status:** Normative handoff map — the register below is filled truthfully
from the stage records; every row is a pointer: the pointed-to document is
the authority, the row is the view.  
**Version:** v0.1  
**Owner/change context:** P0-W22 stage dependency map; row updates follow
§5's rules and never edit contract content.  
**Supersedes:** The absence of an explicit P0 consumption map.

## 1. Purpose and reading rule

For the P0 completion review, P1 planners, and later stages: find what P0
delivered, where its authority lives, what evidence backs it, and who
consumes it. Every row is a pointer — follow it; do not quote it.

## 2. Status vocabulary

| Status | Meaning | Evidence column holds |
|---|---|---|
| `delivered` | the contract exists and its stage validation has recorded evidence | the verification-record path |
| `design-proposed` | an implementation design exists; nothing delivered | the design path |
| `plan-only` | only the plan exists | the plan path |
| `blocked` | with cause and owner | — |

No other status word may appear.

## 3. Deliverable register

| ID | Deliverable contract (one-line purpose) | Authoritative location | Owner | Status | Evidence | Consumers (plan index) |
|---|---|---|---|---|---|---|
| W01 | Repository conventions, root navigation, clone-safe baseline | root `README.md`, `AGENTS.md`, `docs/README.md`, `.gitignore`, `.editorconfig`, `LICENSE` | W01 | delivered | [verification](verification/p0-w01-repository-baseline-verification.md) | W02, W05, W19, W20 |
| W02 | Pinned, recoverable, developer/CI-shared Rust toolchain | root `rust-toolchain.toml`; [toolchain baseline](../../development/toolchain-baseline.md) | W02 | delivered | [verification](verification/p0-w02-rust-toolchain-baseline-verification.md) | W03, W07, W19, W20 |
| W03 | AArch64 bare-metal build path and target-class baseline | [build-target baseline](../../development/build-target-baseline.md); workspace manifests; toolchain target entry | W03 | delivered | [verification](verification/p0-w03-aarch64-build-target-baseline-verification.md) | W07, W09, W19, W20, P1 |
| W04 | Build capability/profile/feature governance | [build-profile governance](../../development/build-profile-governance.md) | W04 | delivered | [verification](verification/p0-w04-build-profile-feature-governance-verification.md) | W03, W07, W16, P1+ |
| W05 | Documentation taxonomy, metadata, stage separation | [documentation baseline](../../development/documentation-baseline.md) | W05 | delivered | [verification](verification/p0-w05-documentation-baseline-verification.md) | W06, W10–W22, P1+ |
| W06 | ADR lifecycle and architecture-change handling | [ADR process](../../adr/README.md); [ADR template](../../templates/adr-template.md) | W06 | delivered | [verification](verification/p0-w06-adr-governance-verification.md) | every later plan/stage |
| W07 | Quality-gate register, standards, failure semantics | [quality gates](../../development/quality-gates.md) | W07 | delivered | [verification](verification/p0-w07-development-quality-gates-verification.md) | W20, P1+ |
| W08 | Host-test organization and CI-callable execution entry | [host-test baseline](../../testing/host-test-baseline.md); `crates/host-test-baseline/` | W08 | delivered | [verification](verification/p0-w08-host-side-testing-baseline-verification.md) | W07, W19, W20, P1+ |
| W09 | Single QEMU runner entry (interface-only placeholder in P0) | [QEMU runner entry](../../testing/qemu-runner-entry.md) | W09 | delivered | [verification](verification/p0-w09-qemu-automation-entry-baseline-verification.md) | W19, W20, P1+ |
| W10 | unsafe justification, inventory, review, boundary rules | [unsafe policy](../../security/unsafe-rust-policy.md); [unsafe inventory](../../security/unsafe-inventory.md) | W10 | delivered | [verification](verification/p0-w10-unsafe-rust-governance-verification.md) | P1+ low-level work |
| W11 | Core/Arch/SoC/Board separation and capability-driven platform rules | [platform portability rules](../../development/platform-portability-rules.md) | W11 | delivered | [verification](verification/p0-w11-platform-portability-guardrails-verification.md) | P1+ platform work |
| W12 | Diagnostic channels, levels, visibility, fatal minimums | [diagnostics baseline](../../development/diagnostics-baseline.md) | W12 | delivered | [verification](verification/p0-w12-logging-diagnostic-baseline-verification.md) | W13, W16, P1+ |
| W13 | Trace-event namespace and compatibility governance | [trace event namespace](../../development/trace-event-namespace.md) | W13 | delivered | [verification](verification/p0-w13-trace-event-namespace-baseline-verification.md) | P1+ telemetry |
| W14 | Distinct invariant/guest/resource/unsupported/platform failure classes | [failure classification](../../security/failure-classification.md) | W14 | delivered | [verification](verification/p0-w14-panic-failure-classification-verification.md) | W10, W12, P1+ |
| W15 | Semantic address/identifier type distinctions | [address & identifier type-safety requirements](../../development/address-identifier-type-safety.md) | W15 | delivered | [verification](verification/p0-w15-address-identifier-type-safety-verification.md) | P1+ designs |
| W16 | Build identity and compatibility metadata baseline | [version & build metadata baseline](../../development/version-build-metadata.md) | W16 | delivered | [verification](verification/p0-w16-version-build-metadata-baseline-verification.md) | W17, W19, P1+ |
| W17 | Machine-processable artifact naming | [artifact naming baseline](../../development/artifact-naming.md) | W17 | delivered | [verification](verification/p0-w17-artifact-naming-baseline-verification.md) | W19, W20, P1+ |
| W18 | Dependency evaluation governance (TCB, no_std, license, unsafe, platform) | [dependency governance](../../development/dependency-governance.md); [dependency register](../../development/dependency-register.md) | W18 | delivered | [verification](verification/p0-w18-dependency-governance-verification.md) | P1+ dependency decisions |
| W19 | Clone-to-verified and branch-to-PR contributor workflow | [contributor workflow](../../development/contributor-workflow.md) | W19 | delivered | [verification](verification/p0-w19-reproducible-development-workflow-verification.md) | W20, P1 onboarding |
| W20 | CI checks, required-check enforcement, `main` protection | [CI baseline](../../development/ci-baseline.md); `.github/workflows/ci.yml`; `main` protection settings | W20 | delivered | [verification](verification/p0-w20-ci-baseline-verification.md) | P0 completion, P1+ |
| W21 | Layer responsibility flow, admission, traceability rules | [stage workflow](../../development/stage-workflow.md) | W21 | delivered | [verification](verification/p0-w21-stage-plan-implementation-workflow-verification.md) | every later stage |
| W22 | This handoff map and the completion-report contract | this document | W22 | delivered | [verification](verification/p0-w22-stage-dependency-map-verification.md) | P0 completion, P1 planning |

## 4. Consumption mapping

Missing P0 inputs are upstream defects — recorded, with the P0 package
named — never permission to redesign P0 inside the consuming stage (the P1
task book's standing gap policy).

### 4.1 P1 (EL2 minimum bring-up)

| P1 expectation | P0 deliverable(s) | Reuse condition | Blocking implication if missing/unverified |
|---|---|---|---|
| Documented workspace/toolchain | W02, W03 (workspace), W04 (governance) | `delivered` | P1-W01/W02 designs cannot fix their build environment; upstream defect against W02/W03; P1 must not pin its own toolchain |
| AArch64 target | W03 | `delivered`; the triple and its policy are read from W03's contract, never re-chosen | no buildable artifact path; upstream defect against W03 |
| Build entry point | W03, gated by W07 (`QG-BUILD-TARGET`), executed by W20 | `delivered` (entry and enforcement) | target buildability unprovable per change; upstream defect against W03/W07/W20 |
| QEMU entry point | W09 (contract); first implementation P1-W10 | `delivered` (contract); the P0 entry is an interface-only placeholder; P1-W10 implements under it | P1-W10 would re-derive the runner interface — prohibited; upstream defect against W09 |
| CI gates | W07 (register), W20 (enforcement), W08 (host tests) | `delivered` | changes merge without required checks; upstream defect against W07/W20; P1 must not weaken the register |
| Logging/panic baseline | W12, W13, W14 | `delivered` | P1-W06/W07 would re-derive diagnostics semantics; upstream defect against W12–W14 |
| Version/build metadata | W16, W17 | `delivered` | artifacts unidentifiable per the contracts; upstream defect against W16/W17 |
| unsafe governance | W10 | `delivered` | the first `unsafe` has no review/inventory path; upstream defect against W10 |
| Address/error conventions | W15 (types), W14 (failure classes) | `delivered` | P1 designs would re-derive newtype and failure-class rules; upstream defect against W14/W15 |
| Contributor/workflow governance | W01, W19, W21, W05/W06, W18 | `delivered` | agents re-derive reading orders and escalation paths; upstream defect against the named packages |

### 4.2 P2 (platform discovery and memory)

| P2 need | P0 deliverable(s) | Reuse condition | Blocking implication |
|---|---|---|---|
| Documentation/platform-class conventions for new contracts | W05, W11 | `delivered` | contracts placed or classified ad hoc; upstream defect |
| ADR escalation for platform decisions | W06 | `delivered` | platform decisions taken without the ADR path; upstream defect |
| Address/memory type conventions | W15, W14 | `delivered` | allocator/ownership designs re-derive type rules; upstream defect |
| Regression runner entry (P2-W09) | W09 | `delivered` contract; P1's implementation is the precedent | P2 regression would embed its own QEMU path; upstream defect |
| Artifact identity for new artifacts | W16, W17 | `delivered` | artifacts unnamed per grammar; upstream defect |

### 4.3 P3 (SMP) and standing later-stage rule

| Need | P0 deliverable(s) | Reuse condition | Blocking implication |
|---|---|---|---|
| Gates/CI for concurrency work | W07, W20, W08 | `delivered` | SMP changes merge unverified; upstream defect |
| Telemetry/trace governance | W12, W13 | `delivered` | IPI/lock trace events without a namespace; upstream defect |
| New dependency intake (e.g. test harnesses) | W18 | `delivered` | ad-hoc dependency additions; upstream defect |

**Standing rule:** every later stage consumes W01, W05, W06, and W21 as
governance ground; a stage that finds one of them missing or unverified
records an upstream defect against the owning package and proceeds only
through the [stage workflow's admission table](../../development/stage-workflow.md),
never by local redesign.

## 5. Completion-report contract

The P0 completion report (`verification/p0-completion-report.md`) is the L7
artifact authored by the stage completion review (stage workflow layer
contract). It must contain:

- **Required links:** every package's implementation record and verification
  record (W01–W22), this map, the task book, and every governing contract
  the register rows name.
- **Per-validation-ID status:** a table over P0-V01–P0-V15 with, per ID:
  `verified` (evidence pointer), `unverified` (no evidence exists; state
  why), or `blocked` (cause and owner). No other word. The task book §7
  exit-criteria checklist is answered from this table.
- **Open issues:** every pending ADR-register item the stage touched, every
  unresolved design conflict, every recorded blocker — each with owner and
  tracking location.
- **Scope honesty:** the reserved and out-of-scope boundaries restated by
  pointer (task book §1), and the explicit statement of what P0 evidence
  does not prove (no EL2, guest, QEMU-execution, or hardware claims —
  citing the host-test, runner-entry, and CI proof boundaries).
- **Handoff assembly:** the handoff package enumerated per task book §7,
  assembled by linking the register rows' artifacts — never by copying
  content.

## 6. Update rules

- **Same-change rule:** a package's row moves to `delivered` (or `blocked`)
  in the same change that records the package's evidence; a map lagging the
  records is a defect, not a TODO.
- **Pointer-only rule:** row updates change status, evidence pointers, and
  consumer links; they never edit contract content.
- **Mutation thresholds:** adding/removing a register or consumption row, or
  changing the status vocabulary or completion-report contract, is a
  design-level change recorded against this map; wording corrections are
  ordinary review; any change that would let a stage treat an unverified
  input as delivered is `ADR Required` per the stage workflow's escalation
  path.
- **Extension rule:** later stages create their own stage-to-stage maps
  following this map's shape; they extend the pattern, never rewrite the P0
  map except through these update rules.
