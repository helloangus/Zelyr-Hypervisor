# P0-W19 Workflow Stage Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W19 detailed design](README.md).

## 1. Logical artifact groups and ownership

W19 is documentation work, so its logical modules are authoritative artifact
groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Contributor workflow document | `docs/development/contributor-workflow.md` | the prerequisite packages' contracts, the integration-workflow policy, this design | the single end-to-end clean-environment path and branch-to-PR route, by reference; it does not restate commands or policies, and it does not implement or configure anything |
| Documentation routing | one row in `docs/README.md` routing table | workflow document location | discoverability of the path; it does not restate stages |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w19-reproducible-development-workflow-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations, prerequisite availability audit; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w19-reproducible-development-workflow-verification.md` (created when evidence exists) | walkthrough observations, commands, output | run/blocked/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. In particular, the integration-workflow policy remains
the sole authority for branch/PR/merge rules: the workflow document links it
and must not restate, summarize into divergence, or amend it.

## 2. Document structure contract

`docs/development/contributor-workflow.md` must carry the status header
required by `docs/README.md` (status, scope, version `v0.1`, owner/change
context, supersedes: none) and exactly these parts:

1. **Audience and promise** — who this is for (new contributors and agents)
   and what completing the path yields (a verified local environment and a
   merged-through-PR change), with the explicit statement that the document
   assembles other contracts and owns no command or policy itself.
2. **The stage chain** — the seven stages of §3, in order, each with the
   per-stage content of §4.
3. **Boundary sections** (§5 below): host vs target; the QEMU entry's P0
   placeholder boundary; artifact identification; enforcement status.
4. **Integration path** — the branch→PR→merge route per
   [the integration path design](02-integration-path-and-walkthrough.md) §2,
   referencing the policy.
5. **Blocked-stage legend** — what a contract-pending or blocked mark means
   and how it is cleared.

Additional informative detail is allowed but must not restate a contract's
content or contradict its authority.

## 3. Stage chain

| Stage | Outcome | Owning contracts (link targets) | Next step |
|---|---|---|---|
| S0 — clone and entry reading | a fresh clone with the mandatory entry documents read | W01 repository baseline (root `README.md`, `AGENTS.md`, `docs/README.md`) | S1 |
| S1 — toolchain restoration | the exact pinned toolchain and components provisioned from the repository declaration | W02 toolchain contract (`docs/development/toolchain-baseline.md` once delivered) | S2 |
| S2 — host build and tests | host-side build and host test entry executed successfully under the pinned toolchain | W08 host-testing baseline entry; W07 gate definitions | S3 |
| S3 — AArch64 target build | the bare-metal target artifact built through the documented entry | W03 target baseline entry | S4 |
| S4 — QEMU runner entry | the single documented QEMU runner entry invoked at its P0 placeholder level | W09 QEMU automation entry | S5 |
| S5 — artifact identification | the produced artifact identified: name parsed per grammar, metadata associated per identity schema | W16 metadata contract; W17 naming contract | S6 |
| S6 — integration path | the change developed on a new branch and merged through a GitHub PR per policy | [integration workflow](../../../../development/integration-workflow.md); W20 enforcement caveat | done |

Each stage's link target is the contract document's actual in-tree path at
implementation time; if the owning package has not delivered its document,
the stage is written with its planned citation and marked contract-pending
(§4 rules).

## 4. Per-stage content contract

Every stage must carry exactly these five content elements, and nothing that
belongs to its owning contract:

1. **Purpose** — one sentence on what the stage achieves in the chain.
2. **Authoritative references** — links to the owning contract document(s)
   and, inside them, the section that holds the commands or procedure. The
   workflow document states *that* the reader must follow the reference, never
   *what* the reference says operationally.
3. **Declared inputs** — what the reader must already have (prior stages,
   machine prerequisites as declared by the owning contract, nothing
   machine-local and undocumented).
4. **Expected evidence** — what a successful stage looks like in terms the
   owning contract defines (for example "the restoration verification from
   the toolchain contract passes"), so the reader can self-check without new
   criteria.
5. **Failure attribution and next step** — when the stage fails, which owning
   package's contract governs the failure, where to record it, and what the
   next stage is. A stage must never leave a failure without an owner.

Stage rules:

- **Contract-pending marking:** when the owning contract document is not
  yet in-tree, the stage states its planned citation, the expected document
  path, and is marked contract-pending. It must not inline commands,
  paraphrase the missing contract, or imply the stage is executable.
- **No duplication:** if a stage's text and its contract ever disagree, the
  contract wins and the workflow document is fixed; two statements of one
  procedure is a review failure.
- **No execution substitution:** expected evidence is described so the reader
  can recognize success; the document never reports a stage as performed.

## 5. Boundary sections the document must contain

### 5.1 Host versus target

Host-side builds and tests (S2) run on the development machine under the
pinned toolchain and prove host-side logic only. The AArch64 target build (S3)
cross-compiles the bare-metal artifact and proves the compilation chain, not
execution. Success in S2 is evidence for P0-V03/V04 only; success in S3 is
evidence for P0-V05 only; neither substitutes for the other and neither is
runtime behavior evidence. The section must make the non-interchangeability
explicit.

### 5.2 QEMU entry placeholder boundary

S4 reaches the single documented QEMU runner entry (P0-V13). In P0 it is a
runner/placeholder boundary, not an EL2 test: no guest boot, no hypervisor
log, no runtime claim may be inferred from reaching it. The section must
state this boundary verbatim so no reader treats a placeholder invocation as
virtualization evidence.

### 5.3 Artifact identification

S5 identifies what S3 produced: the artifact name parsed against the naming
grammar and its identity fields associated per the metadata schema. The
section explains that this is how "the thing I just built" is tied to "the
source and declared build/compatibility information it came from" — the
P0-V14 association the task book demands — without restating either
contract's rules.

### 5.4 Enforcement status

The section must state, truthfully and permanently keyed to the current
state: the branch→PR policy is adopted and binding as human process; GitHub
required checks and branch protection are **not yet configured** and are
W20's deliverable (P0-V08); until then, "required online checks" names the
checks W20 will configure, and no reader should expect to observe them. The
statement must be written so that W20's delivery (not W19's text) retires it.

## 6. Explicitly excluded content

No command spelling, tool invocation, flag, environment-variable contract,
target triple, GitHub workflow, check name, branch-protection setting, or
policy paraphrase may appear in the workflow document. Where a stage seems to
need one, the stage cites its owning contract instead; if no contract supplies
it, the stage is contract-pending, not improvised.
