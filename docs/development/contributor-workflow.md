# Zelyr Contributor Workflow

**Status:** Normative contributor path (assembly of the governing contracts;
it owns no command and no policy itself).  
**Scope:** The single end-to-end clean-environment path (clone → toolchain →
host build/tests → target build → QEMU entry → artifact identification) and
the branch-to-PR integration route, by reference to the owning contracts. It
does not restate, summarize, or amend any contract or policy; where this
document and a contract ever disagree, the contract wins and this document is
fixed.  
**Version:** v0.1  
**Owner/change context:** P0-W19 reproducible development workflow; stage
citations track their contracts' delivered paths.  
**Supersedes:** The absence of an assembled contributor path.

## 1. Audience and promise

This is for a new contributor or agent on a clean machine. Completing the
stage chain (S0–S5) yields a verified local environment; completing S6 yields
a change merged through a GitHub pull request. Every stage cites the
contract that owns its procedure and its success criteria — this document
only tells you which stage comes next and where its rules live.

## 2. Stage chain

### S0 — Clone and entry reading

- **Purpose:** a fresh clone with the mandatory entry documents read.
- **Authoritative references:** the repository root [`README.md`](../../README.md),
  [`AGENTS.md`](../../AGENTS.md), and the [documentation
  index](README.md) (routing table). Read them before any stage.
- **Declared inputs:** Git; network access to the configured remote.
- **Expected evidence:** a working tree checked out on a branch off current
  `main`; the entry documents present and read.
- **Failure attribution and next step:** clone/remote problems are
  environment issues to record in the contributor's own notes; continue to S1.

### S1 — Toolchain restoration

- **Purpose:** the exact pinned toolchain and components provisioned from the
  repository declaration alone.
- **Authoritative references:** [toolchain baseline](toolchain-baseline.md),
  §5 (restoration) — its declared inputs and verification commands are the
  procedure and the success criteria.
- **Declared inputs:** S0 done; a rustup-capable host; network access to the
  official Rust distribution.
- **Expected evidence:** the restoration verification commands from the
  toolchain contract pass (active toolchain equals the pin; rustfmt and
  clippy present; installed targets equal the manifest list).
- **Failure attribution and next step:** failures are governed by the
  toolchain contract's failure boundaries — record the blocker there per its
  rules; never substitute another toolchain. Next: S2.

### S2 — Host build and tests

- **Purpose:** the host-side build and the host-test entry executed
  successfully under the pinned toolchain.
- **Authoritative references:** [host-test baseline](../testing/host-test-baseline.md)
  (the single execution entry, §3–§4) and [quality gates](quality-gates.md)
  (§1 register: `QG-TEST-HOST` binding; §9 development set).
- **Declared inputs:** S1 done.
- **Expected evidence:** the host-test entry exits success with its reported
  test summary (run/passed/failed/skipped) and no unreported skips.
- **Failure attribution and next step:** a failing host test or build is
  governed by the host-test baseline and the gate register's failure
  semantics; fix before proceeding (gates block merge). Next: S3.

### S3 — AArch64 target build

- **Purpose:** the bare-metal target artifact built through the documented
  entry.
- **Authoritative references:** [build-target baseline](build-target-baseline.md),
  §6 (build invocation) — the entry, environment, and artifact boundary are
  the contract's.
- **Declared inputs:** S1 done (the target is provisioned through the same
  manifest).
- **Expected evidence:** the entry completes and the declared ELF artifact
  exists under the AArch64 target's profile directory, per the build-target
  contract's artifact boundary.
- **Failure attribution and next step:** build failures are governed by the
  build-target contract's failure boundary (no improvised linker scripts,
  target JSONs, or unstable features); record and diagnose there. Next: S4.

### S4 — QEMU runner entry (P0 placeholder)

- **Purpose:** reach the single documented QEMU runner entry at its P0
  placeholder level.
- **Authoritative references:** [QEMU runner entry contract](../testing/qemu-runner-entry.md)
  — §8 is this stage's entire content in P0: the entry is interface-only.
- **Declared inputs:** S3 done (the future boot subject exists).
- **Expected evidence:** knowledge of the entry, its grammar, and its
  placeholder status. **No execution, no runner, no QEMU run occurs in P0.**
- **Failure attribution and next step:** there is nothing to fail in P0; the
  first concrete implementation is P1-W10's. Next: S5.

### S5 — Artifact identification

- **Purpose:** identify what S3 produced — tie "the thing I just built" to
  "the source and declared build/compatibility information it came from."
- **Authoritative references:** [version & build metadata](version-build-metadata.md)
  (§1 questions, §2 schema, §3 policies) and [artifact naming](artifact-naming.md)
  (§3 grammar, §5 identity mapping).
- **Declared inputs:** S3 done; the identity inputs of the build (revision,
  dirty state) known per the metadata contract's policies.
- **Expected evidence:** the artifact's name parses against the grammar per
  its class's applicability row, and its identity fields answer the metadata
  contract's questions Q1–Q7.
- **Failure attribution and next step:** unparseable names or unanswerable
  identity questions are review failures governed by the naming and metadata
  contracts. Next: S6.

### S6 — Integration path

- **Purpose:** the change developed on a new branch and merged through a
  GitHub PR per policy. See §4 below.

### Blocked-stage legend

A stage marked **contract-pending** states its planned citation and expected
document path but is not executable and may not be improvised. A stage is
cleared when its owning contract is delivered and the citation updated. No
stage in this version is contract-pending: every cited contract is in-tree.

## 3. Boundary sections

### 3.1 Host versus target

Host-side builds and tests (S2) run on the development machine under the
pinned toolchain and prove **host-side logic only**. The AArch64 target build
(S3) cross-compiles the bare-metal artifact and proves the **compilation
chain, not execution**. S2 success is evidence only where host evidence is
required; S3 success is evidence only of target buildability. Neither
substitutes for the other, and neither is runtime-behavior evidence.

### 3.2 QEMU entry placeholder boundary

S4 reaches the single documented QEMU runner entry. **In P0 it is an
interface placeholder, not an EL2 test: no guest boot, no hypervisor log,
and no runtime claim of any kind may be inferred from reaching it.** No QEMU
execution occurs under this workflow in P0.

### 3.3 Artifact identification

S5 identifies what S3 built: the artifact's name is parsed against the
naming grammar, and its identity fields are associated per the metadata
schema. This is how a produced artifact is tied to the source and declared
build/compatibility information it came from; the rules live in the two
cited contracts, not here.

### 3.4 Enforcement status

The branch→PR policy (see §4) is adopted and binding as **human process**.
GitHub required checks and branch protection are **not yet configured**;
they are the CI-baseline package's deliverable (P0-V08). Until that package
delivers, "required online checks" names the checks it will configure — do
not expect to observe them yet. This statement is retired by that package's
delivery, not by an edit motivated from here.

## 4. Integration path (stage S6)

This is the mandatory route for every post-policy change. The governing
policy is the [branch and pull-request integration
workflow](integration-workflow.md); this section adds only the
contributor-side entry into it and never amends it.

1. **Entry from the stage chain:** S6 begins only from an environment that
   has reached S2 (and S3 when the change touches target buildability) — a
   PR is opened for validated work, not for an unverified tree.
2. **Branch creation:** one branch per bounded change, based on current
   `main`, named per the policy's descriptive-namespace convention (see the
   policy).
3. **Local validation before pushing:** run the minimum pre-push validation
   set from the owning contracts — the quality gates' development set (§9 of
   the gate register), the host-test entry, and the target build when
   applicable to the change.
4. **Push and pull request:** push to the configured GitHub remote; open the
   PR with `main` as its base. The PR description carries what the policy
   requires: the work package or policy change, changed contracts, validation
   run and not run, and any unresolved conflict.
5. **Required online checks:** per the policy, every configured required
   check must pass; a missing, cancelled, skipped, or failed required check
   is not passing evidence. The configured set does not exist until the
   CI-baseline package delivers it (§3.4 above).
6. **Merge:** an authorized maintainer merges the PR after the required
   checks pass, per the policy's merge-method rule. Direct pushes to `main`
   are prohibited by the policy; nothing here softens that.

| Responsibility | Owner |
|---|---|
| Branch hygiene, local validation, PR description truthfulness | contributor |
| Policy text and its future amendments | the policy's owning change (not this document) |
| GitHub workflows, required-check classification, branch protection, enforcement evidence | the CI-baseline package (P0-W20, P0-V08) |
| Merge decision | authorized maintainer |
