# P0-W22 Handoff-Map Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W22 detailed design](README.md).

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Handoff-map document | `docs/stages/p0/p0-handoff-map.md` | this design, the [plan index](../../plans/README.md), the task books, the stage implementation index, the delivered records and designs | the sole normative home of the deliverable register, the consumption mapping, the completion-report content contract, and the update rules; it does not restate any contract, author the completion report, or plan P1+ internals |
| Documentation routing | one row in `docs/README.md` routing table; stage index row in `docs/stages/p0/implementation/README.md` | map location / design status | discoverability and truthful status; no restatement |
| Implementation record | `../p0-w22-stage-dependency-map-record.md` (created when work starts) | decisions taken, status-audit results, deviations | changed artifacts and decisions; no command logs |
| Verification record | `../../verification/p0-w22-stage-dependency-map-verification.md` (created when evidence exists) | drill observations, review output | run/blocked/not-run evidence per the validation matrix; not part of the design |

The record and verification paths are future locations; this design does not
create them.

## 2. Map document structure (normative content of the map)

`docs/stages/p0/p0-handoff-map.md` must carry the status header required by
`docs/README.md` (status, scope, version `v0.1`, owner/change context,
supersedes: none) and exactly these parts:

1. **Purpose and reading rule** — who the map is for (P0 completion review,
   P1 planners, later stages) and the rule that every row is a pointer: the
   pointed-to document is the authority, the row is the view.
2. **Status vocabulary** — the four states of §3 with their evidence rules.
3. **The deliverable register** — §3's table, one row per P0-W01–W21.
4. **The consumption mapping** — §4's tables.
5. **The completion-report contract** — §5.
6. **The update rules** — §6.

## 3. Deliverable register

The register covers W01–W21. At implementation time the status and evidence
columns are filled truthfully from the stage records; this design fixes the
rows' shape and content sources. The "authoritative location" column names
the deliverable's contract or configuration artifact; where a sibling design
has not delivered, the row names the owning package and the stage
implementation index as the location source — it never invents a path. The
concrete paths shown above for design-proposed packages are the locations
those proposed designs declare; the authoring change re-verifies each against
the delivered artifact and follows the delivered form if it differs, so the
map never pins a path its owning package did not deliver.

| ID | Deliverable contract (one-line purpose) | Authoritative location | Owner | Stage validation | Consumers (plan index) |
|---|---|---|---|---|---|
| W01 | Repository conventions, root navigation, clone-safe baseline | root `README.md`, `AGENTS.md`, `docs/README.md`, `.gitignore`, `.editorconfig`, `LICENSE` | W01 | P0-V01, P0-V09 | W02, W05, W19, W20 |
| W02 | Pinned, recoverable, developer/CI-shared Rust toolchain | root `rust-toolchain.toml`; `docs/development/toolchain-baseline.md` | W02 | P0-V02 | W03, W07, W19, W20 |
| W03 | AArch64 bare-metal build path and target-class baseline | `docs/development/build-target-baseline.md`; workspace manifests; toolchain target entry | W03 | P0-V05 | W07, W09, W19, W20, P1 |
| W04 | Build capability/profile/feature governance | `docs/development/build-profile-governance.md` | W04 | P0-V09, P0-V15 | W03, W07, W16, P1+ |
| W05 | Documentation taxonomy, metadata, stage separation | `docs/development/documentation-baseline.md` | W05 | P0-V09 | W06, W10–W22, P1+ |
| W06 | ADR lifecycle and architecture-change handling | `docs/adr/README.md`; `docs/templates/adr-template.md` | W06 | P0-V10 | every later plan/stage |
| W07 | Quality-gate register, standards, failure semantics | `docs/development/quality-gates.md` | W07 | P0-V06–V08 | W20, P1+ |
| W08 | Host-test organization and CI-callable execution entry | `docs/testing/host-test-baseline.md`; the baseline host test | W08 | P0-V03–V04 | W07, W19, W20, P1+ |
| W09 | Single QEMU runner entry (interface-only placeholder in P0) | `docs/testing/qemu-runner-entry.md` | W09 | P0-V13 | W19, W20, P1+ |
| W10 | unsafe justification, inventory, review, boundary rules | `docs/security/unsafe-rust-policy.md`; `docs/security/unsafe-inventory.md` | W10 | P0-V11 | P1+ low-level work |
| W11 | Core/Arch/SoC/Board separation and capability-driven platform rules | named by W11's delivered design (see stage implementation index) | W11 | P0-V12 | P1+ platform work |
| W12 | Logging, tracing, metrics, panic/release, version-metadata semantics | named by W12's delivered design (see stage implementation index) | W12 | P0-V09, P0-V14 | W13, W16, P1+ |
| W13 | Trace-event namespace and compatibility governance | named by W13's delivered design (see stage implementation index) | W13 | P0-V09 | P1+ telemetry |
| W14 | Distinct invariant/guest/resource/unsupported/platform failure classes | named by W14's delivered design (see stage implementation index) | W14 | P0-V09 | W10, W12, P1+ |
| W15 | Semantic address/identifier type distinctions | named by W15's delivered design (see stage implementation index) | W15 | P0-V09 | P1+ designs |
| W16 | Build identity and compatibility metadata baseline | `docs/development/version-build-metadata.md` | W16 | P0-V14 | W17, W19, P1+ |
| W17 | Machine-processable artifact naming | `docs/development/artifact-naming.md` | W17 | P0-V14 | W19, W20, P1+ |
| W18 | Dependency evaluation governance (TCB, no_std, license, unsafe, platform) | `docs/development/dependency-governance.md`; `docs/development/dependency-register.md` | W18 | P0-V09 | P1+ dependency decisions |
| W19 | Clone-to-verified and branch-to-PR contributor workflow | `docs/development/contributor-workflow.md` | W19 | P0-V01–V05, P0-V13 | W20, P1 onboarding |
| W20 | CI checks, required-check enforcement, `main` protection | `docs/development/ci-baseline.md`; `.github/workflows/`; `main` protection settings | W20 | P0-V08 | P0 completion, P1+ |
| W21 | Layer responsibility flow, admission, traceability rules | `docs/development/stage-workflow.md` | W21 | P0-V09, P0-V15 | every later stage |

Register rules:

- A row is complete only when all six fields are filled and the status and
  evidence columns (below) are truthful. A row with an invented location or
  an unevidenced `delivered` status is a register defect.
- Status column vocabulary (normative): `delivered` — the contract exists
  and its stage validation has recorded evidence (pointer to the
  verification record required); `design-proposed` — an implementation
  design exists (pointer required), nothing delivered; `plan-only` — only
  the plan exists (pointer required); `blocked` — with cause and owner.
  No other status word may appear.
- Evidence column: for `delivered`, the verification-record path; for
  `design-proposed`, the design path; otherwise empty. Evidence lives only
  in the stage verification area.

## 4. Consumption mapping

The map must contain these tables. Reuse conditions reference the status
vocabulary; blocking implications cite the [P1 task
book](../../../p1/task-book-v0.1.md) §1 rule as the standing gap policy:
missing P0 inputs are upstream defects — recorded, with the P0 package
named — never permission to redesign P0 inside the consuming stage.

### 4.1 P1 (EL2 minimum bring-up)

The P1 task book §1 supply list, resolved row by row:

| P1 expectation (cited supply list item) | P0 deliverable(s) | Reuse condition | Blocking implication if missing or unverified |
|---|---|---|---|
| Documented workspace/toolchain | W02, W03 (workspace), W04 (governance) | `delivered` | P1-W01/W02 designs cannot fix their build environment; upstream defect recorded against W02/W03; P1 must not pin its own toolchain |
| AArch64 target | W03 | `delivered`; the target triple and its policy are read from W03's contract, never re-chosen | P1 boot design has no buildable artifact path; upstream defect against W03 |
| Build entry point | W03, gated by W07 (`QG-BUILD-TARGET`), executed by W20 | `delivered` (entry), `delivered` (gate enforcement) | P1 cannot prove target buildability per change; upstream defect against W03/W07/W20 |
| QEMU entry point | W09 (entry contract), first implementation P1-W10 | `delivered` (contract). The P0 entry is an interface-only placeholder; P1-W10 implements it under the contract | P1-W10 cannot design the boot regression without re-deriving the runner interface — prohibited; upstream defect against W09 |
| CI gates | W07 (register), W20 (enforcement), W08 (host tests) | `delivered` | P1 changes merge without the required checks; upstream defect against W07/W20; P1 must not weaken the register |
| Logging/panic baseline | W12, W13, W14 | `delivered` | P1-W06/W07 would re-derive diagnostics semantics; upstream defect against W12–W14 |
| Version/build metadata | W16, W17 | `delivered` | P1 artifacts cannot be identified per the metadata/naming contracts; upstream defect against W16/W17 |
| unsafe governance | W10 | `delivered` | P1's first `unsafe` has no review/inventory path; upstream defect against W10 |
| Address/error conventions | W15 (types), W14 (failure classes) | `delivered` | P1 designs would re-derive newtype and failure-class rules; upstream defect against W14/W15 |
| Contributor/workflow governance | W01 (entries), W19 (path), W21 (layer flow), W05/W06 (documents and escalation), W18 (dependency intake) | `delivered` | P1 agents re-derive reading orders and escalation paths; upstream defect against the named packages |

### 4.2 P2 (platform discovery and memory)

| P2 need | P0 deliverable(s) | Reuse condition | Blocking implication |
|---|---|---|---|
| Documentation/platform-class conventions for new contracts | W05, W11 | `delivered` | P2 contracts placed or classified ad hoc; upstream defect |
| ADR escalation for platform decisions | W06 | `delivered` | Platform decisions taken without the ADR path; upstream defect |
| Address/memory type conventions | W15, W14 | `delivered` | P2 allocator/ownership designs re-derive type rules; upstream defect |
| Regression runner entry (P2-W09) | W09 | `delivered` contract; P1's implementation is the precedent | P2 integration regression would embed its own QEMU path; upstream defect |
| Artifact identity for new artifacts | W16, W17 | `delivered` | P2 artifacts unnamed per grammar; upstream defect |

### 4.3 P3 (SMP) and standing later-stage rule

| Need | P0 deliverable(s) | Reuse condition | Blocking implication |
|---|---|---|---|
| Gates/CI for concurrency work | W07, W20, W08 | `delivered` | SMP changes merge unverified; upstream defect |
| Telemetry/trace governance | W12, W13 | `delivered` | IPI/lock trace events without a namespace; upstream defect |
| New dependency intake (e.g. test harnesses) | W18 | `delivered` | Ad-hoc dependency additions; upstream defect |

Standing rule (normative map content): every later stage consumes W01, W05,
W06, and W21 as governance ground; a stage that finds one of them missing or
unverified records an upstream defect against the owning package and proceeds
only through the [W21 admission table](../p0-w21-stage-plan-implementation-workflow/02-admission-and-traceability.md),
never by local redesign.

## 5. Completion-report content contract

The map must fix the following as the P0 completion report's required
content. The report is authored by the stage completion review at P0
completion ([W21](../p0-w21-stage-plan-implementation-workflow/README.md) L7;
the map applies that layer contract); W22 writes only this contract.

- **Future location:** `docs/stages/p0/verification/p0-completion-report.md`
  (created at completion; this design and the map do not create it).
- **Required links:** every package's implementation record and verification
  record (W01–W22), the map itself, the task book, and every governing
  contract the register rows name.
- **Per-validation-ID status:** a table over P0-V01–P0-V15 with, per ID:
  `verified` (evidence pointer), `unverified` (no evidence exists; state
  why), or `blocked` (cause and owner). No other word. The exit-criteria
  checklist of task book §7 is answered from this table.
- **Open issues:** every pending ADR-register item the stage touched
  (e.g. ADR-054 naming), every unresolved design conflict, every recorded
  blocker, each with its owner and tracking location.
- **Scope honesty:** the reserved and out-of-scope boundaries restated by
  pointer (task book §1), and the explicit statement of what P0 evidence
  does not prove (no EL2, guest, QEMU-execution, or hardware claims —
  citing the W08/W09/W20 proof boundaries).
- **Handoff assembly:** the handoff package enumerated per task book §7,
  assembled by linking the register rows' artifacts — never by copying
  content.

## 6. Update rules

- **Same-change rule:** a package's row moves to `delivered` (or to
  `blocked`) in the same change that records the package's evidence; a map
  lagging the records is a defect, not a TODO.
- **Pointer-only rule:** row updates change status, evidence pointers, and
  consumer links; they never edit contract content.
- **Mutation thresholds:** adding/removing a register row or consumption row,
  or changing the status vocabulary or completion-report contract, is a
  design-level change recorded against the map; wording corrections are
  ordinary review; any change that would let a stage treat an unverified
  input as delivered is `ADR Required` per the W21 escalation path.
- **Extension rule:** later stages create their own stage-to-stage maps
  following this map's shape; they extend the pattern, never rewrite the P0
  map except through the update rules above.

## 7. Explicitly excluded content

No contract prose, no P1+ module or sequencing plan, no completion claim, no
evidence copy, and no new document class. If a row seems to need contract
detail to be useful, the row deepens its pointer instead (for example to the
owning contract's section). If the drill or a consumer appears to require
re-deciding a P0 choice, that is the stop boundary: record the upstream
defect and raise it through the [W21 escalation
path](../p0-w21-stage-plan-implementation-workflow/02-admission-and-traceability.md).
