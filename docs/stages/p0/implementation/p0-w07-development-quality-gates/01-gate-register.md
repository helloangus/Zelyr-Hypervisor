# P0-W07 Gate Register Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W07 detailed design](README.md).

## 1. Logical artifact groups and ownership

W07 is policy and documentation work, so its logical modules are authoritative
artifact groups, not code modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Quality-gates contract | `docs/development/quality-gates.md` | this design (register §2–§5, standards file), ADR/task-book constraints, delivered W02/W03/W08 entries | the sole normative home of gate classification, standards, minimum sets, and failure semantics; it does not implement checks, configure CI, or define the bound build/test entries |
| Documentation routing | one row in `docs/README.md` | contract location | discoverability of the contract; it does not restate gate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `../p0-w07-development-quality-gates-record.md` (created when work starts) | decisions taken, prerequisite-surface findings, recorded minor changes | changed artifacts and deviations; no command logs (those live in verification) |
| Verification record | `../../verification/p0-w07-development-quality-gates-verification.md` (created when evidence exists) | actual dry-run commands and output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it. The record and verification paths above are future locations;
this design does not create them.

## 2. Gate register

The register is the complete list of gates W07 defines. Every row is normative
content of the contract document.

| Gate ID | Name | Class | Blocks merge | Scope (summary) | Invocation binding | Evidence label | Stage validation |
|---|---|---|---|---|---|---|---|
| `QG-FMT` | Formatting | Required | Yes | All workspace Rust sources formatted under the pinned toolchain | W07-fixed; see [standards](02-gate-minimum-standards.md) §2 | `QG-FMT` + commit/PR ref | P0-V06 |
| `QG-LINT` | Lint | Required | Yes | Clippy analysis with the warning policy applied | W07-fixed; see [standards](02-gate-minimum-standards.md) §3 | `QG-LINT` | P0-V07 |
| `QG-WARN` | Warning policy | Required | Yes | No compiler/lint warning suppressed or tolerated in gate-bearing builds | policy W07; mechanics in W03's build baseline; see [standards](02-gate-minimum-standards.md) §4 | `QG-WARN` | P0-V07 |
| `QG-TEST-HOST` | Host tests | Required | Yes | Every host-target test executes and passes, independent of QEMU | bound to the W08 host-test entry; see [standards](02-gate-minimum-standards.md) §5 | `QG-TEST-HOST` | P0-V03/V04 |
| `QG-BUILD-TARGET` | AArch64 target build | Required | Yes | The bare-metal AArch64 baseline artifact builds through the delivered entry | bound to the W03 build entry; see [standards](02-gate-minimum-standards.md) §6 | `QG-BUILD-TARGET` | P0-V05 |
| `QG-DOCS` | Documentation consistency | Required | Yes | Tracked-document link integrity and entry-point reachability | semantics W07; CI realization W20; see [standards](02-gate-minimum-standards.md) §7 | `QG-DOCS` | P0-V09 |

Register rules:

- The register contains exactly these six members at W07 closure. No required
  gate may be added or removed except through the mutation thresholds in §3.
- A gate row is complete only when all seven fields are filled. A row with an
  unresolved binding is a register defect and fails review, unless the
  workflow §1 failure boundary for a not-yet-delivered entry applies and says
  so explicitly in the row.
- The evidence label is the attribution key that must appear in check output,
  CI check names, and saved evidence so any gate result can be traced to its
  register row and stage validation ID.

## 3. Classification and promotion rules

- **Required:** proves a property the task book assigns to P0 quality. Always
  merge-blocking, locally and in CI. A required gate that cannot run (missing
  prerequisite surface) is not satisfied by silence; it is recorded as
  blocked, and the PR carrying the cause may not merge while the gate it
  disabled is load-bearing.
- **Informational:** reports a useful quality signal without blocking. The
  class is defined at P0 and its membership may legitimately be empty; expected
  future members include rustdoc generation warnings and dependency-audit
  reporting. An informational check never silently converts to required; that
  is a promotion.
- **Future:** a named, owned check that cannot be defined yet because its
  subject does not exist (EL2 smoke, Linux guest regression, hardware,
  fuzz/property, unsafe audit). Future members are non-blocking by definition
  until promoted. Promotion requires: an owning approved design, a register
  row with all seven fields, a recorded rationale, and — where the check would
  become load-bearing for hypervisor-TCB claims — review under the ADR change
  path. W20 must present future-class checks as explicitly not-verified, per
  its plan's prohibition on misreporting future checks as done.
- **Mutation thresholds:**
  - *Recorded minor change* (ordinary PR review, recorded in the
    implementation record): adjusting a bound invocation spelling to track a
    delivered W03/W08 entry without semantic change; rewording a standard for
    clarity without changing its meaning.
  - *Design-level change* (recorded against the contract, reviewer-approved):
    adding/removing a register member; changing a class or blocking flag;
    changing a passing condition.
  - *ADR-level:* any change that would let hypervisor-TCB-affecting code merge
    without a gate the baseline ADR's validation strategy (ADR-049) expects.

## 4. Machine-consumability requirements

W20 consumes the register as data. The contract document must therefore keep,
per gate, exactly the seven fields of §2 in a stable table form, plus:

- the class and blocking flag as explicit words, not prose implications;
- the invocation binding as a pointer to the authoritative owner (this file
  for W07-fixed gates; the W08/W03 contract documents for bound gates);
- the evidence label as a stable string.

W20 maps each Required row to one GitHub check, sets its required status,
implements `main` protection, and owns evidence that enforcement is real.
W20's workflow-level realization (job layout, tool installation, caching,
artifact upload) is its own implementation freedom, bounded by these rules: a
GitHub check must not claim a gate it does not fully execute, and it must not
execute a gate whose register semantics it contradicts.

## 5. Explicitly excluded interfaces

The register authorizes no executable artifact. It does not name a script
path, define a program interface, or commit CI YAML. The only machine-facing
surface is the documentary field set in §2/§4. If implementing W07's own
dry-runs appears to require committing a gate script or a workflow file, that
is the W20/W03 boundary appearing early: stop and record the conflict instead
of creating the artifact.
