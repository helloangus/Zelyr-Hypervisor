# Zelyr Version & Build Metadata Baseline

**Status:** Normative identity/metadata governance.  
**Scope:** The mandatory artifact-identity questions, the identity field
schema, the timestamp and dirty-tree policies, the project-version single
declaration, the ADR-040 compatibility reservations, and the linkage rules
to diagnostics (P0-W12) and artifact naming (P0-W17). It does not document
build commands, artifact naming grammar, diagnostic channels, or CI; it
defines no binary/wire representation, environment-variable contract, or CI
workflow.  
**Version:** v0.1  
**Owner/change context:** P0-W16 version/build metadata baseline;
operationalizes ADR-040 (independent versioning) and the P0-V14 identity
wording.  
**Supersedes:** The absence of an identity/metadata policy.

## 1. Mandatory artifact identity questions

Every governed artifact must be able to answer these questions, each from its
declared source. A question without a source, or a source without a question,
fails review.

| # | Identity question | Declared source of the answer |
|---|---|---|
| Q1 | Which project release does this artifact correspond to? | the single `project_version` declaration (§3, §4.3) |
| Q2 | Exactly which source state produced it? | `source_revision` plus the `dirty` indication (§4.2) |
| Q3 | Under which build profile and compiled-in capability set? | `build_profile` per the [build-choice governance](build-profile-governance.md); `capability_summary` once W04 semantics land in-tree |
| Q4 | For which target architecture and platform? | `target_architecture` per the [build-target baseline](build-target-baseline.md); `platform` per the platform vocabulary that baseline's boundary names |
| Q5 | When was it produced, in a way that does not break reproducibility? | the revision-derived timestamp rule (§4.1); wall-clock time is auxiliary only |
| Q6 | Which compatibility contracts does it declare or reserve? | the reserved compatibility positions (§5); declared values come only from their own future designs |
| Q7 | How is a human or tool directed from the artifact to these answers? | the association mechanics of the producing package's design; the name-encoded subset per W17's grammar (§6.2) |

The questions are the stable contract; the sources may gain mechanisms (for
example when a build system first exists) without changing the questions.

## 2. Identity field schema

Representation below is deliberately informative (textual form for
readability); no binary or wire representation is authorized.

| Field | Meaning | Source / authority | Status for the first target artifact |
|---|---|---|---|
| `project_version` | project release identity | single declaration, §4.3 | required |
| `source_revision` | exact source state (version-control commit designator) | version-control metadata of the build tree | required |
| `dirty` | whether the build tree deviated from `source_revision` | §4.2 definitions | required |
| `build_profile` | which governed build profile produced the artifact | build-choice governance semantics | required once W04 semantics are in-tree; before that the field is required-but-unfilled and any produced artifact is blocked-by-prerequisite, not exempt |
| `target_architecture` | architecture family of the artifact | build-target baseline | required (value family AArch64 per ADR-002; the concrete target definition is W03's, delivered) |
| `platform` | platform designator the artifact was built for | platform vocabulary from the build-target boundary | required once W03's boundary names the vocabulary; before that required-but-unfilled as above |
| `build_time` | auxiliary wall-clock provenance of one build invocation | build environment | optional; never identity (§4.1) |
| `capability_summary` | declarative summary of compiled-in build capabilities | build-choice capability/profile classification | reserved slot; activates with W04 semantics in-tree |
| `schema_version` | reserved for the future configuration/schema versioning contract | ADR-040 | reserved position; no value or format |
| `machine_version` | reserved for the future machine-model contract | ADR-040 | reserved position; no value or format |
| `management_abi_version` | reserved for the future management ABI contract | ADR-040 | reserved position; no value or format |

Rules:

- The **required subset** for the first target artifact is `project_version`,
  `source_revision`, `dirty`, `target_architecture`, and `build_profile`;
  `platform` joins the required set when the target boundary names its
  vocabulary. A produced artifact that cannot be associated with every
  required field fails review; it is never silently shipped with fields
  missing.
- Adding an optional field is an ordinary reviewed change; changing the
  required subset, a field's meaning, or a policy section is a recorded
  policy decision; removing or redefining a reserved compatibility position
  is `ADR Required` (it touches ADR-040's consequence).

## 3. Policies

### 3.1 Timestamp policy (determinism-first)

- The only identity-sanctioned timestamp is one **derived from the source
  revision** (its commit timestamp). Two builds of the same revision must be
  able to produce identical identity metadata.
- Wall-clock `build_time` is auxiliary provenance: it may be recorded, must
  be labelled non-identity, and must never be compared for artifact equality
  or embedded in artifact names (W17 enforces the name side).
- No policy in the repository may make a wall-clock timestamp a required
  input for reproducing an artifact.

### 3.2 Dirty-tree policy

- **Clean tree:** a checkout of exactly `source_revision` with no uncommitted
  tracked changes and no untracked file that participates in the build as
  declared by the build baseline's inputs.
- **Dirty tree:** anything else. Dirty builds are legitimate for local
  development; their artifacts must carry the dirty indication and must not
  be used as release, gate, or verification evidence.
- Verification-evidence subject artifacts must come from clean trees. Once
  CI exists, W20's builds are from clean checkouts; until then this rule
  binds human practice, and this contract states that explicitly.
- The dirty indication is part of identity: stripping or ignoring it fails
  review.

### 3.3 Project version declaration and single source

- **The current project version is `0.1.0`, declared here.** Exactly one
  authoritative project-version declaration exists in the repository: this
  section. Member manifests' `version` keys (the hypervisor member's
  placeholder, per the build-target baseline's handoff note) are
  package-local values, not the project release declaration, and must not
  drift from this declaration without a same-change review of both.
- When the workspace introduces a shared version key ([workspace.package],
  anticipated when members need shared keys), the declaration migrates there
  and this document keeps the schema and a pointer; the migration is a
  reviewed same-change edit of both locations, never a period of two
  declarations.
- Pre-release and post-release identifiers are permitted by the declared
  scheme (`0.1.0` initial value) but are not defined further in P0; a fuller
  release scheme is a later reviewed change to this contract.

## 4. Compatibility reservations (ADR-040)

- `schema_version`, `machine_version`, and `management_abi_version` are
  independent of `project_version` and of each other.
- This baseline assigns them no values, no formats, no compatibility rules,
  and no production mechanics. Their first concrete definition belongs to
  the future ABI and machine-model designs that own those contracts.
- The positions exist so that no later design has to renegotiate the identity
  schema to introduce them, and so that no build/diagnostic/naming rule
  silently conflates project identity with compatibility identity.

## 5. Linkage rules (cross-review obligations)

### 5.1 Diagnostics (P0-W12, delivered)

- The **minimum diagnostic identity set** this baseline supplies is:
  `project_version`, `source_revision`, `dirty`, `target_architecture`,
  `build_profile`.
- The [diagnostics baseline](diagnostics-baseline.md) references this set
  instead of restating fields (its §5 association property). Cross-review
  outcome: **consistent** — W12's property ("associable with the identity
  minimum owned by P0-W16, by reference") matches the set above; no field
  divergence exists. The mutual cross-review is recorded in both
  verification records.

### 5.2 Artifact naming (P0-W17)

- Artifact names may encode only a declared subset of the identity fields.
  This baseline's authority is the field set and its semantics; W17's
  grammar decides which fields appear in names and in what form. **A name is
  a view of identity, never a second authority.**
- The mapping table lives in W17's naming contract; W16's cross-review
  checks that every name field traces to a §2 schema field and that no name
  field invents identity information the schema does not carry.

## 6. Mutation rules

- **Ordinary reviewed change:** adding an optional field; correcting
  informative representation text; adding an informative example.
- **Recorded policy decision** (issue and owner decision before the change):
  changing the required subset; changing a field's meaning or source;
  changing the timestamp or dirty policies; migrating the version declaration
  home; activating `capability_summary` per W04's delivered in-tree
  semantics.
- **ADR Required:** removing or redefining a reserved compatibility position;
  any change that would make project identity load-bearing for ABI or
  machine compatibility (the inverse conflation is also prohibited).
