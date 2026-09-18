# P0-W10 Unsafe Policy Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W10 detailed design](README.md).

This file is the source of the normative sections of
`docs/security/unsafe-rust-policy.md`. Section numbers below are design
material, not the document's required headings; the workflow fixes how they
map into the document.

## 1. Artifact groups

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Unsafe policy document | `docs/security/unsafe-rust-policy.md` | ADR-006/ADR-049 constraints, Coding Guidelines unsafe rules, this design | the sole normative home of the safe-Rust-first principle, justification and SAFETY-comment requirements, boundary categories, forbidden patterns, review rules, inventory schema, and escalation thresholds; it decides no concrete API or code |
| Unsafe inventory | `docs/security/unsafe-inventory.md` | policy schema; actual future unsafe changes | the sole register of first-party unsafe segments; starts empty; carries no policy prose beyond the schema pointer |
| Documentation routing | one row in `docs/README.md` (a pointer line in `docs/security/README.md` is allowed only without restating policy) | policy location | discoverability; no policy duplication |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; never a completion claim |
| Implementation record | `../p0-w10-unsafe-rust-governance-record.md` (created when work starts) | decisions taken | changed artifacts and deviations; no command logs |
| Verification record | `../../verification/p0-w10-unsafe-rust-governance-verification.md` (created when evidence exists) | actual review output | run/not-run evidence per the validation matrix; never an unsafe-introduction claim |

## 2. Safe-Rust-first principle (policy content)

`unsafe` is a measured concession, never a convenience. The policy must
state, as binding rules:

1. Every proposed `unsafe` requires a **necessity statement**: why safe Rust
   (including the standard library's safe abstractions, existing audited
   abstractions introduced through dependency governance, or a redesigned
   interface) cannot express the operation.
2. `unsafe` is used to implement a **safe abstraction**, not exposed raw:
   the smallest practical boundary encloses it, and callers receive a safe
   interface whose contract the abstraction guarantees — or, where an
   `unsafe` public API is genuinely the contract, that API is itself a
   documented, reviewed safety contract.
3. Scope minimality: the `unsafe` region is the smallest expression of the
   operation; speculative "while we're here" unsafe, `unsafe` in tests for
   convenience, and widening an existing boundary are prohibited.
4. Prefer removing unsafe over documenting it: a change that lets an entry
   leave the inventory does so; the inventory is expected to shrink over
   time, and growth is the signal review watches.

## 3. Justification requirements (policy content)

Every non-trivial `unsafe` block or function must carry, in the source, a
`SAFETY` comment with exactly these required contents:

```text
// SAFETY: <precondition> — <establishment argument> — <failure class> — <inventory id>
```

- **Precondition:** the invariant(s) that must hold for the operation to be
  sound, stated checkably (validity, alignment, lifetime, exclusivity,
  hardware state).
- **Establishment:** why the precondition holds at this point — which code,
  type invariant, or earlier check guarantees it.
- **Failure class:** the consequence if the precondition is in fact broken,
  named per the failure-classification baseline (P0-W14,
  `p0-w14-panic-failure-classification`; subject); until that taxonomy is delivered, the policy's placeholder rule
  applies: state whether the consequence is contained, VM-local, or a
  hypervisor-invariant violation.
- **Inventory id:** the back-link to the inventory entry (schema and lifecycle
  in §3–§4 of the [inventory contract](02-inventory-contract.md)).

Trivial `unsafe` (if any class is ever recognized) is still inventoried; only
the template's depth is at the reviewer's discretion, and recognizing a
trivial class is a policy-decision threshold, not a reviewer habit.

## 4. Review rules (policy content)

An unsafe-bearing change is acceptable only when all of the following hold:

1. an **approved detailed design** explicitly names the unsafe segment, its
   category, and its necessity statement — an unsafe appearing in code that
   its design did not name is a scope violation to stop at review;
2. a **completed SAFETY comment** per §3 accompanies the code;
3. an **inventory entry** per the [inventory contract](02-inventory-contract.md)
   is created or updated **in the same change**;
4. a **second reviewer** with arch/systems context reviews soundness
   explicitly — approval of the feature is not approval of the unsafe;
5. **minimality review** confirms the smallest boundary and that no
   forbidden pattern (§6) is present;
6. the change states what **host tests** ([W08](../p0-w08-host-side-testing-baseline/README.md)
   categories) and what QEMU/hardware validation will cover the wrapped
   operation — host coverage never substitutes for the latter.

## 5. Allowed boundary categories (policy content)

Categories implement ADR-006's concentration requirement. Each entry fixes
what belongs in it and where it may live (layering per
P0-W11
  (`p0-w11-platform-portability-guardrails`) subject):

| Category | Contains | Placement constraint |
|---|---|---|
| `arch-register` | system-register and system-instruction access at EL2 | architecture layer only; never Core |
| `mmio-volatile` | volatile device-register access and memory-mapped I/O | arch/driver/HAL boundary per layering; never Core |
| `memory-mgmt` | page-table manipulation, TLB invalidation glue, address-space primitives | arch/memory-mechanism layers; Core sees only safe APIs |
| `low-level-struct` | intrusive or pointer-based data structures with provable invariants (rings, slabs, per-CPU areas) | dedicated low-level modules; invariant proofs referenced in SAFETY comments |
| `asm-glue` | Rust↔assembly call boundaries (context save/restore, entry trampolines) | arch layer; the assembly side is documented in the same design |
| `boot-state` | establishment of pre-Rust or pre-invariant machine state (stack, BSS, EL state) where invariants cannot yet exist | earliest entry code only; each use must state the handover point where invariants begin to hold |

Rules: a use must fit exactly one category; a use fitting none is a
policy-decision threshold (§7). Placement violating the layering constraints
is a review failure regardless of category fit. Categories are not quotas —
membership in a category justifies nothing by itself; the per-use necessity
statement still carries the argument.

## 6. Forbidden patterns (policy content)

Prohibited without an explicit, recorded policy-decision exception naming the
alternative considered and rejected:

- `static mut` (the boot-phase case, if ever needed, goes through
  `boot-state` with its handover argument);
- `transmute` except where a design proves no typed alternative exists;
- `unsafe` to evade the borrow checker or to silence errors;
- unchecked indexing/pointer arithmetic over externally influenced
  (guest/device/management) data — such data crosses a validated boundary
  first;
- `unsafe` inside Core policy logic or any board/SoC-named branch (violates
  ADR-006 concentration and the layering invariants);
- widening an existing `unsafe` boundary's scope without re-review of the
  whole boundary;
- new `unsafe` in a change whose design does not name it.

## 7. Escalation thresholds (policy content)

- **Ordinary reviewed change:** adding a justified, categorized unsafe
  segment inside an approved design, with SAFETY comment and same-change
  inventory entry.
- **Policy decision** (recorded issue and owner decision before the change):
  adding a boundary category; recognizing a "trivial" unsafe class;
  permitting a forbidden pattern in a named case; changing the SAFETY
  template or inventory schema.
- **ADR required:** anything that would spread unsafe beyond the ADR-006
  concentration constraints, normalize convenience unsafe, or weaken the
  audit obligations — handled through the ADR change path
  (P0-W06,
  `p0-w06-adr-governance`, subject).

## 8. Future gate predicate (handoff content)

The policy must state, for [W07](../p0-w07-development-quality-gates/README.md)'s
future-class promotion: the check a future inventory-consistency gate would
perform is "every `unsafe` occurrence in the tree maps to a current inventory
entry, and every inventory entry maps to existing code" — bidirectional
consistency. The policy defines the predicate only; classification, wiring,
and enforcement are W07/W20.

## 9. Explicitly excluded interfaces

No code artifact of any kind is authorized. The template and schema govern
future code review; they name no API, path, or type. Third-party dependency
unsafe is out of scope ([W18](../p0-w18-dependency-governance/README.md)).
