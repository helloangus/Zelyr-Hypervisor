# P0-W17 Artifact Category Inventory

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W17 detailed design](README.md).

## 1. Purpose

This file specifies the category inventory section that the naming contract
document (`docs/development/artifact-naming.md`) must contain. The inventory
registers the artifact categories named in the P0-W17 plan's work sequence 1,
assigns each a class token, declares which naming dimensions it carries, and
records what is deliberately **not** decided (format, generation, extension)
so future designs can activate categories without renegotiating the grammar.

## 2. Inventory columns

| Column | Meaning |
|---|---|
| Class token | the `class` vocabulary value; lowercase, hyphen-free per the grammar |
| Category | the artifact kind from the plan's inventory list |
| First consumer | the stage/package expected to produce the first instance; "Reserved" means none is approved yet |
| Applicability | which grammar dimensions the class carries (subset of the §3 dimension list, in canonical order) |
| Status | `P0-required` (the grammar must support it now) or `Reserved` (registered now, activated later by the consumer's design) |
| Expected extension | informative only; the owning design declares the real format and suffix |
| Format/generation owner | the design that will own the file format and production; never W17 |

## 3. Required inventory

| Class token | Category | First consumer | Applicability | Status | Expected extension (informative) | Format/generation owner |
|---|---|---|---|---|---|---|
| `hypervisor` | hypervisor image (AArch64 bare-metal baseline artifact) | W03 (first consumer; P1+ continuous) | full form: name, class, arch, platform, profile, version, revision [+dirty] | P0-required | to be declared by W03 (current ignore policy already covers common binary suffixes) | W03's design; later the owning build designs |
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

Rules the inventory section must state:

- A category not listed may be added only through the naming contract's
  reviewed-change rule; a new token requires a named first consumer.
- `P0-required` means the grammar and applicability must support the category
  when the first consumer's design lands; it does not mean an artifact exists
  today. The `hypervisor` row's first concrete name instance is W03's
  deliverable and is expected `not run` at W17 closure.
- Omitted dimensions are declared per row; no other omission is authorized.
  If a consuming design needs a different subset, the row is edited by
  reviewed change in the same change that delivers the consumer's design.
- The inventory registers **categories, not files**: no row creates, renames,
  or reserves a concrete path, and no row commits a file format.

## 4. Non-categories (explicitly out of inventory)

- Host-side build/test intermediate outputs of a future build tool: internal
  to the build tree, outside the governed scope.
- Tracked stage documents (plans, designs, implementation and verification
  records): their locations are fixed by stage governance and the
  documentation baseline, not by this grammar.
- Dependency artifacts (vendored crates, lockfiles): governed by the
  dependency-governance package (P0-W18) and the owning build designs, not by
  artifact naming.
