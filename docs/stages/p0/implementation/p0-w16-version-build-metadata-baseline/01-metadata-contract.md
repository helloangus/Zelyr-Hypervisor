# P0-W16 Version & Build Metadata Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W16 detailed design](README.md).

## 1. Logical artifact groups and ownership

W16 is documentation/policy work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Version/build metadata contract | `docs/development/version-build-metadata.md` | this design, ADR-040, task-book P0-V14 wording | the sole normative home of identity questions, field schema, timestamp/dirty policies, compatibility reservations, and linkage rules; it does not document build commands, artifact naming grammar (W17), diagnostic channels (W12), or CI |
| Documentation routing | one row in `docs/README.md` routing table | contract document location | discoverability of the contract; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w16-version-build-metadata-baseline-record.md` (created when work starts) | actual decisions taken | declared version, changed artifacts, deviations, prerequisite status assumptions; no command logs (those live in verification) |
| Verification record | `docs/stages/p0/verification/p0-w16-version-build-metadata-baseline-verification.md` (created when evidence exists) | actual review commands and output | run/blocked/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Mandatory artifact identity questions

The contract document must open with the questions every governed artifact must
be able to answer, each with its declared source. A question without a source,
or a source without a question, fails review.

| # | Identity question | Declared source of the answer |
|---|---|---|
| Q1 | Which project release does this artifact correspond to? | the single `project_version` declaration (§3, §4.3) |
| Q2 | Exactly which source state produced it? | `source_revision` plus the `dirty` indication (§4.2) |
| Q3 | Under which build profile and compiled-in capability set? | `build_profile` per W04's governance semantics; `capability_summary` once W04 semantics land |
| Q4 | For which target architecture and platform? | `target_architecture` per W03's target boundary; `platform` per the platform vocabulary W03's boundary names |
| Q5 | When was it produced, in a way that does not break reproducibility? | the revision-derived timestamp rule (§4.1); wall-clock time is auxiliary only |
| Q6 | Which compatibility contracts does it declare or reserve? | the reserved compatibility positions (§5); declared values come only from their own future designs |
| Q7 | How is a human or tool directed from the artifact to these answers? | the association mechanics of the producing package's design; the name-encoded subset per W17's grammar (§6) |

The questions are the stable contract; the sources may gain mechanisms (for
example when a build system first exists) without changing the questions.

## 3. Identity field schema

The contract document must contain this field table. Representation is
deliberately informative (textual form shown for readability); no binary or
wire representation is authorized.

| Field | Meaning | Source / authority | Status for the first target artifact |
|---|---|---|---|
| `project_version` | project release identity | single declaration, §4.3 | required |
| `source_revision` | exact source state (version-control commit designator) | version-control metadata of the build tree | required |
| `dirty` | whether the build tree deviated from `source_revision` | §4.2 definitions | required |
| `build_profile` | which governed build profile produced the artifact | W04's profile semantics (prerequisite) | required once W04 semantics are in-tree; before that the field is required-but-unfilled and any produced artifact is blocked-by-prerequisite, not exempt |
| `target_architecture` | architecture family of the artifact | W03's target boundary | required (value family AArch64 per ADR-002; the concrete target definition is W03's) |
| `platform` | platform designator the artifact was built for | platform vocabulary from W03's boundary | required once W03 names the boundary; before that required-but-unfilled as above |
| `build_time` | auxiliary wall-clock provenance of one build invocation | build environment | optional; never identity (§4.1) |
| `capability_summary` | declarative summary of compiled-in build capabilities | W04's capability/profile classification | reserved slot; activates with W04 semantics |
| `schema_version` | reserved for the future configuration/schema versioning contract | ADR-040 | reserved position; no value or format |
| `machine_version` | reserved for the future machine-model contract | ADR-040 | reserved position; no value or format |
| `management_abi_version` | reserved for the future management ABI contract | ADR-040 | reserved position; no value or format |

Rules the contract must state:

- The **required subset** for the first target artifact is `project_version`,
  `source_revision`, `dirty`, `target_architecture`, and `build_profile`;
  `platform` joins the required set when W03's boundary names it. A produced
  artifact that cannot be associated with every required field fails the
  package's review; it is never silently shipped with fields missing.
- Adding an optional field is an ordinary reviewed change to the contract;
  changing the required subset, a field's meaning, or a policy section is a
  recorded policy decision; removing or redefining a reserved compatibility
  position is `ADR Required` (it touches ADR-040's consequence).

## 4. Policies the contract must contain

### 4.1 Timestamp policy (determinism-first)

- The only identity-sanctioned timestamp is one **derived from the source
  revision** (its commit timestamp). Two builds of the same revision must be
  able to produce identical identity metadata.
- Wall-clock `build_time` is auxiliary provenance: it may be recorded, must be
  labelled non-identity, and must never be compared for artifact equality or
  embedded in artifact names (W17 enforces the name side).
- No policy in the repository may make a wall-clock timestamp a required input
  for reproducing an artifact.

### 4.2 Dirty-tree policy

- **Clean tree:** a checkout of exactly `source_revision` with no uncommitted
  tracked changes and no untracked file that participates in the build as
  declared by the build baseline's inputs.
- **Dirty tree:** anything else. Dirty builds are legitimate for local
  development; their artifacts must carry the dirty indication and must not be
  used as release, gate, or verification evidence.
- Verification evidence artifacts (stage verification records' subject
  artifacts) must come from clean trees. Once CI exists, W20's builds are from
  clean checkouts; until then the rule binds human practice, and the contract
  must say so explicitly.
- The dirty indication is part of identity: stripping or ignoring it fails
  review.

### 4.3 Project version declaration and single source

- The contract document declares the current project version (`0.1.0`
  initially) and the rule that exactly one authoritative declaration exists in
  the repository.
- When the owning build-baseline design (expected W03 per its plan scope)
  introduces a workspace manifest, the declaration migrates there and the
  contract document keeps the schema and a pointer; the migration is a reviewed
  same-change edit of both locations, never a period of two declarations.
- Pre-release and post-release identifiers are permitted by the declared scheme
  but are not defined in P0 beyond the initial value; defining a fuller release
  scheme is a later reviewed change to this contract.

## 5. Compatibility reservations (ADR-040)

The contract must state, and this design fixes:

- `schema_version`, `machine_version`, and `management_abi_version` are
  independent of `project_version` and of each other.
- W16 assigns them no values, no formats, no compatibility rules, and no
  production mechanics. Their first concrete definition belongs to the future
  ABI and machine-model designs that own those contracts.
- The positions exist so that no later design has to renegotiate the identity
  schema to introduce them, and so that no build/diagnostic/naming rule
  silently conflates project identity with compatibility identity.

## 6. Linkage rules (cross-review obligations)

### 6.1 Diagnostics (W12)

- W12's plan requires every diagnostic record to be associable with
  build/version identity. The **minimum diagnostic identity set** W16 supplies
  is: `project_version`, `source_revision`, `dirty`, `target_architecture`,
  `build_profile`.
- The contract must state that set explicitly so W12's contract can reference
  it instead of restating fields. If W12's delivered contract names a
  different set, the two contracts are reconciled in the same change that
  delivers the divergence; a semantic disagreement is a cross-package design
  conflict, and touching ADR-level constraints is `ADR Required`.

### 6.2 Artifact naming (W17)

- Artifact names may encode only a declared subset of the identity fields.
  W16's authority is the field set and its semantics; W17's grammar decides
  which fields appear in names and in what form. The contract must say that a
  name is a *view* of identity, never a second authority.
- The mapping table itself lives in W17's naming contract; W16's cross-review
  checks that every name field traces to a schema field and that no name field
  invents identity information the schema does not carry.

## 7. Mutation rules

- **Ordinary reviewed change:** adding an optional field; correcting
  informative representation text; adding an informative example.
- **Recorded policy decision** (issue and owner decision before the change):
  changing the required subset; changing a field's meaning or source; changing
  the timestamp or dirty policies; migrating the version declaration home;
  activating `capability_summary` per W04's delivered semantics.
- **ADR Required:** removing or redefining a reserved compatibility position;
  any change that would make project identity load-bearing for ABI or machine
  compatibility (the inverse conflation is also prohibited).

## 8. Explicitly excluded code interfaces

There are no Rust types, functions, crates, Cargo manifests, build scripts,
environment-variable contracts, target triples, wire formats, binary layouts,
CI workflows, or diagnostic implementations in this design. The only
machine-facing surface defined is the *field schema as policy*; the only
human-facing procedure is the cross-review obligation. Adding any excluded
item is a scope conflict requiring the applicable detailed design (at minimum
W03 for target and artifact production, W12 for diagnostics, W17 for names,
W20 for CI) and must be stopped at review.
