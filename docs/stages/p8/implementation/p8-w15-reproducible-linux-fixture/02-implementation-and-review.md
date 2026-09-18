# P8-W15 Implementation and Review Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W15 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies it has loaded the documents named in
the parent README and inspects the actual state of the prerequisites: the
approved W03 boot-contract facts, the W04 DTB-contract facts, the approved
machine configuration per W14/W02, and the P0-W19 workflow declarations. Per
the plans index, only evidenced/approved facts may be cited.

Stop and obtain direction instead of improvising when any of the following
occurs:

- the boot/DTB contracts or machine configuration are unapproved — the
  manifest groups dependent on them stay `blocked`; the §4 procedure may be
  exercised against the contract only; do not select bootargs, DTB facts, or a
  machine configuration locally;
- acquisition or re-derivation fails (network, integrity reference, digest
  mismatch) — record the blocker; do not substitute a mirror, a nearby
  version, or a locally modified configuration;
- pinning appears to require committing a binary, hosting an artifact, or
  adopting a distribution — stop; that is the §6 policy/ADR route, never a
  local choice;
- a consumer requests tooling outside the §5 table — route it through the
  intake rule (new need → manifest revision) or the Reserved lane (W17);
  do not add convenience content;
- the exact artifact storage location at run time is undetermined — that is a
  W16 harness decision; the manifest records identities, not storage policy;
- any work seems to require Host-side loading mechanics, kernel development,
  or harness internals — out of scope; record the need for the owning package.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite facts and record the source baseline

Target: implementation record
(`../p8-w15-reproducible-linux-fixture-record.md`, created in this step).

Work: record the approval status of W03/W04 facts and the machine
configuration; record which manifest groups are therefore blocked. Record the
P0-W19 workflow integration point the procedure will slot into.

Suggested observation: read the W03/W04/W14 designs and any approved records;
no network or build action.

**Acceptance:** the baseline names each group's status with its authority;
blocked groups are explicit.
**Failure/blocker:** a permanently missing authority (e.g., no machine
decision route) is recorded as a blocker for the dependent groups.

### Step 2 — select identities and author the manifest

Target: the fixture manifest per
[01 §1–§2](01-fixture-manifest-contract.md) (home recorded in the
implementation record).

Work: apply the §3 selection rules to choose the exact kernel-source and
initramfs-component identities; record the configuration identity and
rationale; write the manifest with the schema of
[01 §2](01-fixture-manifest-contract.md), the §5 content table, the §4
procedure sections, and license metadata. Leave artifact digests unrecorded
and dependent groups blocked as applicable.

**Acceptance:** W15-DV02 schema review passes: all required groups present;
exact identities (no moving references); blocked groups explicit; no digests
pre-filled; no tooling mandated beyond what the route records.
**Failure/blocker:** a schema or selection-rule violation fails review; fix
the manifest, not the rules.

### Step 3 — exercise acquisition and pinning

Target: verification record
(`../../verification/p8-w15-reproducible-linux-fixture-verification.md`).

Work: execute [01 §4](01-fixture-manifest-contract.md) steps 1–2 (acquire,
regenerate-or-acquire, digest) as far as the blocked state allows. Record every
command, output, environment, and timestamp. Record what was not run and why.

**Acceptance:** sources acquired and verified by their integrity references;
route executed or its blocked extent recorded; digests computed for any
artifact actually produced.
**Failure/blocker:** an acquisition failure is recorded failed/blocked with
diagnosis; no substitution.

### Step 4 — verify conformance and reproducibility evidence

Target: verification record.

Work: execute §4 steps 3–5 as far as allowed: bootargs/DTB/initramfs/machine-
binding conformance checks against the approved facts, and the independent
re-derivation for every artifact the route claims reproducible. Record digest
comparisons and the re-derivation result.

**Acceptance:** the W15-DV04 substance: a pinned identity set with digest
evidence, plus re-derivation evidence for the route's reproducibility claims —
or an explicit blocked record naming the unmet approvals. This satisfies the
definition requirement of P8-V20; it is not a built-image claim.
**Failure/blocker:** a conformance mismatch routes to the owning contract
(W03/W04/W14), not to a local fix; a failed re-derivation downgrades the
route's determinism claim in the manifest with a recorded investigation.

### Step 5 — policy, licensing, and closure review

Work: run the W15-DV05 policy review (versioning thresholds, licensing
posture, evidence-record completeness) and the W15-DV06 consumability review;
then run the validation matrix (§3) and confirm the handoff checklist (§5) —
in particular that consumer-facing identities in the manifest match what this
design promised W09–W19. Verify the package against the plan's acceptance
wording and the task-book P8-V20 row. Completion is claimed only in the
verification record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W15-DV01 → P8-V20 | source-fact review | inspect W03/W04/W14/P0-W19 status against 01 §1 | every group's authority named; blocked groups explicit | the contract sits on real authorities; not that artifacts exist |
| W15-DV02 → P8-V20 | manifest schema review | review the manifest against 01 §2–§3 | all groups present; exact identities; blocked cells explicit; digests unrecorded until verified; single-source respected | the definition is reviewable and pinnable; not that it acquires |
| W15-DV03 → P8-V20 | content-by-need review | review the §5 table against W09–W12 requirement sources | every entry names its need and source; no unneeded content; Reserved content absent | the tool set serves declared needs; not that tools work |
| W15-DV04 → P8-V20 | acquisition/pinning/re-derivation exercise | execute 01 §4 with recorded output | sources verified; digests recorded for produced artifacts; re-derivation evidence for reproducibility claims — or explicit blocked record with the unmet approvals | the definition is pinnable and (for claimed artifacts) reproducibly generable; not that Linux boots, that the fixture is optimal, or that a built image exists |
| W15-DV05 → P8-V20 | versioning/licensing review | review 01 §6–§7 against the plan and ADR boundaries | thresholds explicit; licensing posture recorded; no distribution commitment | governance coherence; not future compliance |
| W15-DV06 → closure | consumer consumability review | read the manifest as W09/W10 (boot inputs), W11/W12 (programs/markers), W16 (identities/steps), W17–W19 (expectations) | each consumer can act without inventing identities or content | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. A manifest without
the W15-DV04 exercise does not satisfy P8-V20's "pinned or reproducibly
generated definition" requirement, and no evidence may claim a fixture binary
exists without its digest record. Nothing here contributes to P8-V12–V19 or
P8-V21–V26.

## 4. Error, security, and observability model

**Errors and failure guarantee.** W15 adds no runtime error path. Its failure
model is evidential and fail-closed: an unresolvable identity, integrity
failure, digest mismatch, or failed re-derivation is recorded as failed/blocked
— never papered over with substitutions, schema loosening, or silent identity
changes. A consumer reading a manifest identity is guaranteed that the identity
was verified at pinning time or is explicitly marked otherwise.

**Security.** The fixture is supply-chain-sensitive: it determines the code
that runs as the untrusted Linux Guest in every P8 regression. The contract's
position: exact upstream identities with the upstream's own integrity
references, digest pinning of artifacts, and recorded routes make every
fixture change reviewable; mirrors, vendoring, and private rebuilds are route
changes requiring the §6 policy path. The definitions-only posture also keeps
the repository free of unauditable blobs. Guest-side content remains Guest
input: markers and outputs are observations, trusted for nothing (matching the
W11/W12/W13 rules).

**Observability.** The evidence surface is the verification record (procedure
commands, outputs, digests, environments, re-derivation results, not-run
entries) plus the implementation record (selected identities, manifest home,
deviations). The manifest itself is the durable observable: consumers and
reviewers can always see what is pinned, what is blocked, and what the route
claims.

## 5. Handoff checklist

Before handing W15 to a reviewer, provide:

- the exact changed-file list (expected: the fixture manifest; the
  implementation record; verification entries; no binaries, no code, no
  builder scripts mandated by the design);
- the source baseline with per-group authority and blocked status (W15-DV01);
- the selected identities with selection dates, resolution sources, and
  configuration rationale (W15-DV02);
- the acquisition/pinning/re-derivation evidence, including digest records and
  explicit blocked/not-run entries (W15-DV04);
- the content-by-need table traceability to W09–W12 requirement sources
  (W15-DV03);
- the versioning/licensing review outcome (W15-DV05);
- confirmation that no distribution choice, binary commit, Host-loading
  mechanism, kernel patch, machine value, bootargs value, or DTB fact was
  selected or created by W15;
- open items: unmet approvals blocking machine binding/bootargs/DTB groups,
  consumer requests in the Reserved lane (W17), and any route-determinism
  investigation — without resolving them here.
