# Zelyr Artifact Naming Baseline

**Status:** Normative artifact-naming governance.  
**Scope:** The governed name grammar and character set, the dimension table
with vocabularies, per-class applicability (category inventory), stability
rules, the identity mapping to the [version/build metadata
baseline](version-build-metadata.md), and evolution rules. It does not
document file formats, generation processes, output directory layouts, or
CI; per-category extensions are informative only.  
**Version:** v0.1  
**Owner/change context:** P0-W17 artifact naming baseline; consumes the W16
identity schema as the sole field-meaning authority.  
**Supersedes:** The absence of an artifact-naming policy (the QEMU runner
entry's placeholder naming rule is superseded by this document, compatible
with it: machine-processable, per-run-unique, deterministic names — see the
[runner entry](../testing/qemu-runner-entry.md)).

## 1. Governed scope

The grammar governs **artifact names for artifacts that leave the build
tree**: artifacts that are published, archived as verification or CI
evidence, or consumed by another tool. It does **not** govern a build tool's
internal layout (for example Cargo's `target/` tree), source files,
documentation paths (governed by the stage layout and the documentation
baseline), or tracked records under `docs/stages/<stage>/`.

Consistency with the existing ignore policy: `.gitignore` already ignores
the common generated-output directories and binary extensions, so governed
artifacts are ordinarily untracked. Naming attaches identity to generated
files; it does not bring them under version control, and no governed name
may be used to circumvent the ignore policy.

## 2. Dimensions

A governed name expresses exactly these dimensions, in this order:

| Order | Dimension | Answers (W16 question) | Vocabulary authority |
|---|---|---|---|
| 1 | `name` | which project (Q1) | §4.1 (ADR-054-pending) |
| 2 | `class` | what kind of artifact (Q7) | §4.2 / the category inventory (§6) |
| 3 | `arch` | target architecture family (Q4) | §4.3 (build-target baseline) |
| 4 | `platform` | platform designator (Q4) | §4.4 (build-target boundary; ADR-003 grounding) |
| 5 | `profile` | build profile (Q3) | §4.5 (build-choice governance) |
| 6 | `version` | project release (Q1) | W16 `project_version` |
| 7 | `revision` | exact source state (Q2) | W16 `source_revision` |
| mod | `+dirty` | dirty-tree indication (Q2) | W16 `dirty` policy |

A class may omit a dimension only when the per-class applicability table
(§6) declares the omission; omitted dimensions are left out entirely — never
replaced by empty or placeholder fields. A dimension not in this table must
not appear in any name.

## 3. Grammar and machine processability

### 3.1 Grammar

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

### 3.2 Character set and parseability rules

- Field values contain only lowercase letters and digits `[a-z0-9]`; the
  `version` field additionally allows `.` (three-part version per W16).
- `-` is the field separator and is forbidden inside values; `+` appears only
  in the `+dirty` modifier; spaces, uppercase letters, and underscores are
  forbidden everywhere in the class-part.
- Because the separator cannot occur inside values and the per-class
  applicability table fixes the field subset and order for each class, a
  name is parsed by splitting on `-` and checking the fields against that
  class's declared subset — no human memory and no content inspection
  required.
- Values that would need a hyphen (board and platform names such as
  `qemu-virt`, `orangepi-3b`) are concatenated, and the vocabulary tables map
  the token to its repository display name, so the mapping lives in a
  declared table rather than in a reader's memory.
- The `ext` is not part of identity. Two artifacts differing only in format
  version or compression are distinguished by their owning design's format
  rules, never by smuggling identity into the extension.

### 3.3 Revision and dirty modifier

- `revision` is the W16 `source_revision` designator (version-control commit
  abbreviation, or a release-tag designator once a release scheme exists).
  It must contain enough characters to be unambiguous within the
  repository's history at generation time; the producing design records the
  chosen minimum.
- `+dirty` is appended exactly when the W16 `dirty` policy marks the build
  dirty. A governed name without `+dirty` asserts a clean build; asserting
  clean for a dirty build is a review failure in the producing package.

## 4. Vocabularies and their governance

### 4.1 Name

The value is the lowercase repository working token `zelyr`. The official
project name is pending under ADR-054; resolution is a reviewed migration:
every governed class's vocabulary note gains the new token, and previously
produced, still-circulating artifacts keep their historical names as
historical evidence. This contract does not decide the official name.

### 4.2 Class

Class tokens are declared by the category inventory (§6). Initial tokens:
`hypervisor` (P0-required), and the reserved tokens `validationguest`,
`linuxguest`, `bootpackage`, `controldomain`, `dtb`, `firmware`, `snapshot`,
`migration`, `symbols`, `report`. Adding a token is an ordinary reviewed
change to the inventory.

### 4.3 Architecture

Initial entry: `aarch64` (ADR-002 grounds the family). The concrete
bare-metal target definition is the build-target baseline's; if a finer
designator is ever needed, it is added here by reviewed change referencing
that delivered contract. Future architectures (ADR-002 defers x86_64) enter
the same way.

### 4.4 Platform

Initial entries: `qemuvirt` (QEMU `virt`, the ADR-003 reference platform)
and `orangepi3b` (Orange Pi 3B, the first real-hardware target), mapped to
their repository display names. A `host` entry is **not** pre-declared:
host-side build/test outputs are governed only when a consuming design first
needs to name one, and that design adds the token with its meaning. Platform
rules and support tiers remain with the [platform portability
rules](platform-portability-rules.md) and later platform designs; the
vocabulary here records designators only.

| Token | Repository display name |
|---|---|
| `qemuvirt` | QEMU virt |
| `orangepi3b` | Orange Pi 3B |

### 4.5 Profile

The profile vocabulary starts **empty** and is populated from the
[build-choice governance](build-profile-governance.md)'s delivered profile
semantics (the ADR-047 long-term profile set is Reserved). A class whose
applicability table includes `profile` cannot produce a governed name until
the vocabulary has the token — a deliberate fail-closed so that no one
invents a profile value to satisfy the grammar.

### 4.6 Adding a vocabulary entry

An ordinary reviewed change states: the token, the display name it maps to,
the owning package/design that requires it, and the date. A token without an
owning consumer is not added.

## 5. Stability and identity linkage

- Governed names are pure functions of their identity inputs: the same
  identity inputs must produce the same name, on any machine, at any time.
  Wall-clock timestamps are forbidden in governed names (the W16 timestamp
  policy is the authority).
- Repeated builds of the same identity may produce the same name; collision
  handling between successive builds is an output-layout concern owned by
  the producing build design, never a reason to mutate names with
  non-identity information.
- Renames of vocabulary tokens follow §4.6; renaming a class token that has
  already produced artifacts requires migration guidance in the same change.

Identity mapping (W16's schema is the sole authority for field meaning):

| Name field | W16 identity field | Notes |
|---|---|---|
| `name` | (project identity) | working token per §4.1; not a W16 field, tracked pending ADR-054 |
| `class` | — (category, not identity) | answers "what is this artifact"; complements identity |
| `arch` | `target_architecture` | family designator; the build-target baseline owns the concrete target |
| `platform` | `platform` | designator; the target boundary's vocabulary |
| `profile` | `build_profile` | token from the build-choice governance |
| `version` | `project_version` | three-part per W16 |
| `revision` | `source_revision` | commit designator per W16 |
| `+dirty` | `dirty` | appended only when dirty per W16 |

Rules: a name is a **view** of identity, never a second authority; no field
outside the W16 schema (except `class` and the pending `name` token) may
appear; `capability_summary` and the reserved compatibility positions
(`schema_version`, `machine_version`, `management_abi_version`) are
deliberately **not** name-encodable — they stay in the metadata record so
names remain bounded and parseable.

## 6. Category inventory

Registers the artifact categories, their class tokens, applicability, and
status. `P0-required` means the grammar must support the category when the
first consumer's design lands — not that an artifact exists today. The
inventory registers **categories, not files**: no row creates a concrete
path or commits a file format. A category not listed may be added only
through §7's reviewed-change rule; a new token requires a named first
consumer. Omitted dimensions are declared per row; no other omission is
authorized. If a consuming design needs a different subset, the row is
edited by reviewed change in the same change that delivers the consumer's
design.

| Class token | Category | First consumer | Applicability (canonical order) | Status | Expected extension (informative) | Format/generation owner |
|---|---|---|---|---|---|---|
| `hypervisor` | hypervisor image (AArch64 bare-metal baseline artifact) | build-target baseline (first consumer; P1+ continuous) | name, class, arch, platform, profile, version, revision [+dirty] | P0-required | to be declared by the owning build design (the ignore policy already covers common binary suffixes) | the owning build designs |
| `validationguest` | validation guest image | Reserved (guest bring-up is P4 per the architecture roadmap) | full form | Reserved | to be declared by the owning P4+ design | owning P4+ design |
| `linuxguest` | Linux guest image set (kernel/initramfs/DTB set as delivered) | Reserved (Linux guest is P8) | full form | Reserved | to be declared by the owning P8 design | owning P8 design |
| `bootpackage` | boot package (hypervisor + control domain + manifest bundle) | Reserved (boot-package stage per the architecture roadmap) | full form | Reserved | to be declared by the owning design | owning design |
| `controldomain` | Control Domain image | Reserved (Control Domain stages) | full form | Reserved | to be declared by the owning design | owning design |
| `dtb` | device-tree blobs produced by the project (for example a future guest DTB) | Reserved (platform-discovery and machine-model designs) | full form minus `profile` unless the owning design needs it | Reserved | `.dtb` is conventional but not committed here | owning design |
| `firmware` | firmware-related build outputs (TF-A/U-Boot chain artifacts the project packages) | Reserved (real-hardware bring-up design) | full form | Reserved | to be declared by the owning design | owning design |
| `snapshot` | VM snapshot artifacts | Reserved (advanced-memory design) | full form | Reserved | to be declared by the owning design | owning design |
| `migration` | migration stream artifacts | Reserved (live-migration design) | full form | Reserved | to be declared by the owning design | owning design |
| `symbols` | symbol/map/diagnostic sidecar files for images | Reserved (crash-diagnostics and inspection designs) | name, class, arch, platform, version, revision [+dirty]; `profile` only when mirroring a profiled image | Reserved | to be declared by the owning design | owning design |
| `report` | generated test/QEMU-run report outputs (not the governed stage records under `docs/stages/`) | Reserved (QEMU runner and regression designs) | name, class, platform, version, revision [+dirty]; `arch` only for target-specific runs | Reserved | to be declared by the owning design | owning design |

Non-categories (explicitly out of inventory): host-side build/test
intermediate outputs of a future build tool (internal to the build tree);
tracked stage documents (fixed by stage governance and the documentation
baseline); dependency artifacts such as vendored crates and lockfiles
(dependency-governance package and owning build designs).

## 7. Evolution rules

- **Ordinary reviewed change:** adding a class token or vocabulary entry per
  §4.6; correcting informative text; adding an informative parsing example.
- **Recorded policy decision** (issue and owner decision, grammar version
  bump with migration guidance): adding, removing, or reordering a dimension;
  changing the character set or separators; changing the dirty modifier;
  populating or re-scoping a vocabulary's authority.
- **ADR Required:** treating governed names as an external ABI or
  compatibility contract; any change that would make a name load-bearing for
  machine-model or management-ABI compatibility.

## 8. Format non-commitment (plan out-of-scope guard)

- Per-category expected extensions in §6 are **informative**: they record the
  currently plausible suffix so reviews can discuss artifacts, and they
  commit to nothing. The owning design declares the real format and extension
  when the category activates.
- Nothing in this contract prescribes a generation process, output directory,
  packaging step, or tool invocation.
