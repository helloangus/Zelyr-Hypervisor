# P0-W17 Artifact Naming Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W17 detailed design](README.md).

## 1. Logical artifact groups and ownership

W17 is documentation/policy work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Artifact naming contract | `docs/development/artifact-naming.md` | this design, W16 identity schema, plan category list | the sole normative home of grammar, character set, vocabularies, per-class applicability, stability rules, identity mapping, category inventory, and evolution rules; it does not document file formats, generation processes, output directory layouts, or CI |
| Documentation routing | one row in `docs/README.md` routing table | contract document location | discoverability; it does not restate rules |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w17-artifact-naming-baseline-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations, prerequisite status assumptions; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w17-artifact-naming-baseline-verification.md` (created when evidence exists) | actual review commands and output | run/blocked/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it. The vocabulary tables in §5 are the only token authorities in
the repository; any other document that needs a class, platform, or profile
token links to them.

## 2. Governed scope

The grammar governs **artifact names for artifacts that leave the build tree**:
artifacts that are published, archived as verification or CI evidence, or
consumed by another tool. It does not govern a build tool's internal layout
(for example a future Cargo `target/` tree), source files, documentation paths
(already governed by the stage layout and documentation baseline), or tracked
records under `docs/stages/<stage>/` whose locations are fixed by the stage
governance. The contract must state this boundary verbatim so no later rule
silently expands the grammar's reach.

Consistency with the existing ignore policy: `.gitignore` already ignores the
common generated-output directories and binary extensions, so governed
artifacts are ordinarily untracked. Naming attaches identity to generated
files; it does not bring them under version control, and no governed name may
be used to circumvent the ignore policy.

## 3. Dimensions

A governed name expresses exactly these dimensions, in this order:

| Order | Dimension | Answers (W16 question) | Vocabulary authority |
|---|---|---|---|
| 1 | `name` | which project (Q1) | §5.1 (ADR-054-pending) |
| 2 | `class` | what kind of artifact (Q7) | §5.2 / the category inventory |
| 3 | `arch` | target architecture family (Q4) | §5.3 (W03 target boundary) |
| 4 | `platform` | platform designator (Q4) | §5.4 (W03 boundary; ADR-003 grounding) |
| 5 | `profile` | build profile (Q3) | §5.5 (W04 governance) |
| 6 | `version` | project release (Q1) | W16 `project_version` |
| 7 | `revision` | exact source state (Q2) | W16 `source_revision` |
| mod | `+dirty` | dirty-tree indication (Q2) | W16 `dirty` policy |

A class may omit a dimension only when the per-class applicability table
([the inventory](02-artifact-category-inventory.md)) declares the omission;
omitted dimensions are left out entirely — never replaced by empty or
placeholder fields. A dimension not in this table must not appear in any name.

## 4. Grammar and machine processability

### 4.1 Grammar

```text
name        := class-part extension
class-part  := field ( "-" field )* ( "+" dirty )?
field       := name | class | arch | platform | profile | version | revision
             (subset and order per the per-class applicability table)
dirty       := "dirty"
extension   := "." ext (informative; declared per category)
```

Full canonical form, for illustration of field order only:

```text
<name>-<class>-<arch>-<platform>-<profile>-<version>-<revision>[+dirty]<ext>
```

### 4.2 Character set and parseability rules

- Field values contain only lowercase letters and digits `[a-z0-9]`; the
  `version` field additionally allows `.` (three-part version per W16).
- `-` is the field separator and is forbidden inside values; `+` appears only
  in the `+dirty` modifier; spaces, uppercase letters, and underscores are
  forbidden everywhere in the class-part.
- Because the separator cannot occur inside values and the per-class
  applicability table fixes the field subset and order for each class, a name
  is parsed by splitting on `-` and checking the fields against that class's
  declared subset — no human memory and no content inspection required.
- Values that would need a hyphen (board and platform names such as
  `qemu-virt`, `orangepi-3b`) are concatenated, and the vocabulary tables map
  the token to its repository display name, so the mapping lives in a
  declared table rather than in a reader's memory.
- The `ext` is not part of identity. Two artifacts differing only in format
  version or compression are distinguished by their owning design's format
  rules, never by smuggling identity into the extension.

### 4.3 Revision and dirty modifier

- `revision` is the W16 `source_revision` designator (version-control commit
  abbreviation, or a release tag designator once a release scheme exists). The
  contract must require enough characters to be unambiguous within the
  repository's history at generation time and must state that the producing
  design records the chosen minimum.
- `+dirty` is appended exactly when the W16 `dirty` policy marks the build
  dirty. A governed name without `+dirty` asserts a clean build; asserting
  clean for a dirty build is a review failure in the producing package.

## 5. Vocabularies and their governance

### 5.1 Name

The value is the lowercase repository working token `zelyr`. The contract must
state that the official project name is pending under ADR-054 and that
resolution is a reviewed migration: every governed class's vocabulary note
gains the new token, and previously produced, still-circulating artifacts keep
their historical names as historical evidence. W17 does not decide the official
name.

### 5.2 Class

Class tokens are declared by the [category inventory](02-artifact-category-inventory.md).
Initial tokens: `hypervisor` (P0-required), and the reserved tokens
`validationguest`, `linuxguest`, `bootpackage`, `controldomain`, `dtb`,
`firmware`, `snapshot`, `migration`, `symbols`, `report`. Adding a token is an
ordinary reviewed change to the inventory.

### 5.3 Architecture

Initial entry: `aarch64` (ADR-002 grounds the family). The concrete bare-metal
target definition, including any more specific naming it requires, is W03's
design; if W03 introduces a finer designator, it is added here by reviewed
change referencing W03's delivered contract. Future architectures (ADR-002
defers x86_64) enter the same way.

### 5.4 Platform

Initial entries: `qemuvirt` (QEMU `virt`, the ADR-003 reference platform) and
`orangepi3b` (Orange Pi 3B, the first real-hardware target), mapped to their
repository display names. A `host` entry is **not** pre-declared: host-side
build/test outputs are governed only when a consuming design first needs to
name one, and that design adds the token with its meaning. Platform rules and
support tiers remain with the platform-guardrails owner (P0-W11 for rules) and
later platform designs; the vocabulary here records designators only.

### 5.5 Profile

The profile vocabulary starts **empty** and is populated from W04's delivered
profile governance (ADR-047 names the long-term profile set as Reserved). A
class whose applicability table includes `profile` cannot produce a governed
name until the vocabulary has the token — a deliberate fail-closed so that no
one invents a profile value to satisfy the grammar.

### 5.6 Adding a vocabulary entry

An ordinary reviewed change states: the token, the display name it maps to,
the owning package/design that requires it, and the date. A token without an
owning consumer is not added.

## 6. Stability and determinism

- Governed names are pure functions of their identity inputs: the same
  identity inputs must produce the same name, on any machine, at any time.
- Wall-clock timestamps are forbidden in governed names (W16's timestamp
  policy is the authority).
- Repeated builds of the same identity may produce the same name; collision
  handling between successive builds is an output-layout concern owned by the
  producing build design (expected W03 for the baseline artifact), never a
  reason to mutate names with non-identity information.
- Renames of vocabulary tokens follow §5.6; renaming a class token that has
  already produced artifacts requires migration guidance in the same change.

## 7. Identity linkage (W16 mapping)

The naming contract must contain this mapping table and the rule that W16's
schema is the sole authority for field meaning.

| Name field | W16 identity field | Notes |
|---|---|---|
| `name` | (project identity) | working token per §5.1; not a W16 field, tracked pending ADR-054 |
| `class` | — (category, not identity) | answers "what is this artifact", complements identity |
| `arch` | `target_architecture` | family designator; W03 owns the concrete target |
| `platform` | `platform` | designator; W03 boundary names it |
| `profile` | `build_profile` | token from W04's governance |
| `version` | `project_version` | three-part per W16 |
| `revision` | `source_revision` | commit designator per W16 |
| `+dirty` | `dirty` | appended only when dirty per W16 |

Rules: a name is a **view** of identity, never a second authority; no field
outside the W16 schema (except `class` and the pending `name` token) may
appear; `capability_summary` and the reserved compatibility positions
(`schema_version`, `machine_version`, `management_abi_version`) are
deliberately **not** name-encodable — they stay in the metadata record so
names remain bounded and parseable.

## 8. Format non-commitment (plan out-of-scope guard)

- Per-category expected extensions in the inventory are **informative**: they
  record the currently plausible suffix so reviews can discuss artifacts, and
  they commit to nothing. The owning design declares the real format and
  extension when the category activates.
- Nothing in the naming contract may prescribe a generation process, output
  directory, packaging step, or tool invocation.

## 9. Evolution rules

- **Ordinary reviewed change:** adding a class token or vocabulary entry per
  §5.6; correcting informative text; adding an informative parsing example.
- **Recorded policy decision** (issue and owner decision, grammar version
  bump with migration guidance): adding, removing, or reordering a dimension;
  changing the character set or separators; changing the dirty modifier;
  populating or re-scoping a vocabulary's authority.
- **ADR Required:** treating governed names as an external ABI or compatibility
  contract; any change that would make a name load-bearing for machine-model
  or management-ABI compatibility.

## 10. Explicitly excluded code interfaces

There are no Rust types, functions, crates, Cargo manifests, build scripts,
target triples, output directory layouts, file formats, packaging processes, CI
workflows, or public APIs in this design. The only machine-facing surface is
the grammar as policy; the only human-facing procedure is the naming and
cross-review obligation. Adding any excluded item is a scope conflict requiring
the applicable detailed design (at minimum W03 for the first artifact instance,
W04 for profile values, W16 for identity, W20 for CI) and must be stopped at
review.
