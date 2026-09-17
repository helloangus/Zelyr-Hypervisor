# P0-W01 Artifact Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W01 detailed design](README.md).

## 1. Logical modules and ownership

W01 is documentation/configuration work, so its logical modules are tracked
artifact groups, not Rust crates or source modules.

| Module | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Root navigation | Root `README.md` | approved project identity and P0 boundary | newcomer entry and top-level layout; it does not describe build commands or toolchain details owned by W02/W19 |
| Contributor routing | root `AGENTS.md` and `docs/README.md` | document governance | mandatory reading and document routing; it does not replace the stage task book or Coding Guidelines |
| Version-control bootstrap | `.git/`, initial `main` commit, and configured remote when supplied | W01 artifact inventory and owner-provided remote URL | a cloneable baseline; it does not define CI branch protections or release policy |
| Tracked-layout contract | root `README.md` plus a tracked marker in every intentionally empty reserved directory | current directory inventory | a clone contains every named placeholder; it does not define future crate contents |
| Text and generated-file policy | `.editorconfig` and `.gitignore` | repository file classes | portable editing and ignored output rules; it does not define build output locations for later packages |
| License location | root `LICENSE` | owner-approved license text | one discoverable repository license; it must not invent a legal choice |

The artifact named in the second column is the sole authoritative home for the
statement in its row.  Other documents may link to it but must not duplicate or
contradict it.

## 2. Required artifact changes

### 2.1 Version-control bootstrap contract

W01 creates the repository if the supplied P0 scaffold is not already inside a
Git work tree.  Treat Git initialization and the first commit as part of the
deliverable: without them there is no repository to clone or a tracked-file
view against which P0-V01 can be proven.

The implementation procedure is:

```text
if the root is already a Git work tree:
    verify its top level is this project root and preserve its existing history
else:
    initialize a new repository with initial branch name main

before staging:
    complete the W01 ignore policy and verify local-only files are ignored
    inventory all files intended for the baseline

create the initial baseline commit:
    stage only approved W01 project artifacts
    inspect the staged name/status list
    commit with a message identifying the P0 repository baseline

if the owner provides a canonical remote URL:
    add or verify the named remote and push only with the owner's authorization
else:
    record remote publication as pending; do not invent a hosting service/URL
```

Suggested commands are `git rev-parse --show-toplevel`, `git init -b main`,
`git status --short`, `git add`, `git diff --cached --name-status`, and
`git commit`.  The exact command spelling may vary, but the branch, staged
scope review, initial commit, and non-invention of remote identity are required.
Do not set global Git identity.  If commit identity is not configured, obtain
the owner-approved identity or configure it only for this repository after
approval.

The initial commit is the owner of no future build/runtime contract: it records
only the W01 baseline.  Later packages must use their own scoped commits.

### 2.2 Root README contract

Keep `README.md` as the first human entry point.  Update it only as necessary
to meet all of these content requirements:

- identify Zelyr as an AArch64-first Rust-first Type-1 Hypervisor P0 scaffold;
- link to `AGENTS.md` and `docs/README.md` as mandatory navigation, with no
  implication that reading README authorizes code changes;
- retain a top-level layout table/list naming `docs`, `crates`, `hypervisor`,
  `soc`, `boards`, `guests`, `control`, `scripts`, `tests`, and `.github`, and
  state that their detailed contents await later approved designs;
- state that intentionally reserved empty directories are represented by a
  tracked marker and are not manual post-clone prerequisites;
- link to root `LICENSE` after the owner supplies it; and
- avoid commands, toolchain versions, target names, crate names, runtime
  guarantees, or QEMU invocation details owned by later work packages.

Use repository-relative Markdown links.  A link must work when the document is
viewed from a fresh checkout on a hosting site or locally.

### 2.3 Reserved-directory marker contract

For every intentionally empty directory named in the root layout, add a
tracked `.gitkeep` **only if** it has neither a substantive tracked file nor a
more appropriate scoped README.  A scoped README is preferred when the
directory needs a human-facing boundary statement; otherwise `.gitkeep` is the
minimal marker.  Do not add a Cargo manifest, dummy Rust source, build file, or
generated output merely to retain a directory.

Before adding markers, inspect the actual tracked file list, not only the
working-tree directory list.  This prevents retaining directories that are
untracked locally by accident and prevents placing a marker next to a real
package description.  Expected P0 candidates include empty future source,
board, SoC, guest, control, scripts, test-support, and CI subdirectories; the
implementer must record the exact final list rather than treating this list as
an immutable module tree.

### 2.4 Text and ignore-policy contract

Review `.editorconfig` and `.gitignore` against these exact rules:

- UTF-8, LF, final newline, four-space indentation, and trailing-whitespace
  behavior must be expressed in `.editorconfig`; Markdown may preserve
  intentional line-ending whitespace.
- `.gitignore` may ignore generated outputs and local/editor noise only.  It
  must not ignore source, normative documentation, `LICENSE`, tracked
  placeholder markers, CI configuration, or an error log required as formal
  verification evidence.
- Local developer helper scripts are untracked machine-local tools and must be
  ignored.  They have no project interface, implementation contract, or
  documentation entry.
- Do not add a `.gitattributes` policy unless a concrete tracked file class
  requires semantics that `.editorconfig` cannot express.  Such a need is not
  established by W01.

The current baseline entries for common artifacts (`target`, `build`, `dist`,
`out`, logs, images, and editor noise) may be retained after review.  W01 does
not declare those paths to be outputs of a particular tool.

### 2.5 License artifact contract

When the owner supplies the decision, create exactly one root `LICENSE` with
the approved complete text and the required copyright holder/year notice.
`README.md` links to it.  Do not put a partial license, SPDX guess, copied
third-party notice, or a placeholder asserting a license.  If a dependency
notice or a `NOTICE` file is later required, W18 owns that analysis; W01 only
makes the project license discoverable.

## 3. Explicitly excluded code interfaces

There are no Rust structs, enums, traits, functions, shell interfaces, Cargo
packages, modules, or public APIs in this design.  Adding any of them is a
scope conflict requiring the applicable detailed design (at minimum W02/W03
and potentially P1).
