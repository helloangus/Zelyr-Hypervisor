# Zelyr Unsafe Rust Policy

**Status:** Normative safety governance.  
**Scope:** The safe-Rust-first principle, `unsafe` justification and
`SAFETY`-comment requirements, allowed boundary categories, forbidden
patterns, review rules, escalation thresholds, the future gate predicate, and
the [unsafe inventory](unsafe-inventory.md) schema pointer. It decides no
concrete API or code; third-party dependency unsafe is out of scope
(dependency governance, P0-W18).  
**Version:** v0.1  
**Owner/change context:** P0-W10 unsafe Rust governance; operationalizes
ADR-006 (necessary, reviewed unsafe) and ADR-049 (unsafe audit layer).  
**Supersedes:** The absence of an unsafe policy (the Coding Guidelines' unsafe
rules predate this document and are preserved; this policy adds the
institution around them).

## 1. Safe-Rust-first principle

`unsafe` is a measured concession, never a convenience. Binding rules:

1. Every proposed `unsafe` requires a **necessity statement**: why safe Rust
   (including the standard library's safe abstractions, existing audited
   abstractions introduced through dependency governance, or a redesigned
   interface) cannot express the operation.
2. `unsafe` implements a **safe abstraction**, not exposed raw: the smallest
   practical boundary encloses it, and callers receive a safe interface whose
   contract the abstraction guarantees — or, where an `unsafe` public API is
   genuinely the contract, that API is itself a documented, reviewed safety
   contract.
3. **Scope minimality:** the `unsafe` region is the smallest expression of
   the operation; speculative "while we're here" unsafe, `unsafe` in tests
   for convenience, and widening an existing boundary are prohibited.
4. **Prefer removing unsafe over documenting it:** a change that lets an
   entry leave the inventory does so; the inventory is expected to shrink
   over time, and growth is the signal review watches.

## 2. Justification requirements

Every `unsafe` block or function carries, in the source, a `SAFETY` comment:

```text
// SAFETY: <precondition> — <establishment argument> — <failure class> — <inventory id>
```

- **Precondition:** the invariant(s) that must hold for the operation to be
  sound, stated checkably (validity, alignment, lifetime, exclusivity,
  hardware state).
- **Establishment:** why the precondition holds at this point — which code,
  type invariant, or earlier check guarantees it.
- **Failure class:** the consequence if the precondition is in fact broken.
  Until the P0-W14 failure-classification taxonomy is delivered, state
  whether the consequence is *contained*, *VM-local*, or a
  *hypervisor-invariant violation* (placeholder rule; entries written with
  placeholder wording are re-audited at the next audit trigger after W14
  delivers).
- **Inventory id:** the back-link to the inventory entry
  ([unsafe inventory](unsafe-inventory.md)).

Every unsafe segment is inventoried; only the template's depth is at the
reviewer's discretion, and recognizing a "trivial" class is a
policy-decision threshold (§6), never a reviewer habit.

## 3. Review rules

An unsafe-bearing change is acceptable only when all of the following hold:

1. an **approved detailed design** explicitly names the unsafe segment, its
   category, and its necessity statement — an unsafe appearing in code that
   its design did not name is a scope violation to stop at review;
2. a **completed `SAFETY` comment** per §2 accompanies the code;
3. an **inventory entry** per the [unsafe inventory](unsafe-inventory.md)
   schema is created or updated **in the same change**;
4. a **second reviewer** with arch/systems context reviews soundness
   explicitly — approval of the feature is not approval of the unsafe;
5. **minimality review** confirms the smallest boundary and that no forbidden
   pattern (§5) is present;
6. the change states what **host tests** (host-test baseline categories) and
   what QEMU/hardware validation will cover the wrapped operation — host
   coverage never substitutes for the latter.

## 4. Allowed boundary categories

Categories implement ADR-006's concentration requirement. A use must fit
exactly one category; a use fitting none is a policy-decision threshold (§6).
Placement violating the layering constraints is a review failure regardless
of category fit. Categories are not quotas — membership justifies nothing by
itself; the per-use necessity statement carries the argument.

| Category | Contains | Placement constraint |
|---|---|---|
| `arch-register` | system-register and system-instruction access at EL2 | architecture layer only; never Core |
| `mmio-volatile` | volatile device-register access and memory-mapped I/O | arch/driver/HAL boundary per layering; never Core |
| `memory-mgmt` | page-table manipulation, TLB invalidation glue, address-space primitives | arch/memory-mechanism layers; Core sees only safe APIs |
| `low-level-struct` | intrusive or pointer-based data structures with provable invariants (rings, slabs, per-CPU areas) | dedicated low-level modules; invariant proofs referenced in `SAFETY` comments |
| `asm-glue` | Rust↔assembly call boundaries (context save/restore, entry trampolines) | arch layer; the assembly side is documented in the same design |
| `boot-state` | establishment of pre-Rust or pre-invariant machine state (stack, BSS, EL state) where invariants cannot yet exist | earliest entry code only; each use must state the handover point where invariants begin to hold |

## 5. Forbidden patterns

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

## 6. Escalation thresholds

- **Ordinary reviewed change:** adding a justified, categorized unsafe
  segment inside an approved design, with `SAFETY` comment and same-change
  inventory entry.
- **Policy decision** (recorded issue and owner decision before the change):
  adding a boundary category; recognizing a "trivial" unsafe class; permitting
  a forbidden pattern in a named case; changing the `SAFETY` template or
  inventory schema.
- **ADR required:** anything that would spread unsafe beyond the ADR-006
  concentration constraints, normalize convenience unsafe, or weaken the
  audit obligations — handled through the ADR change path
  ([ADR index](../adr/README.md)).

## 7. Future gate predicate (handoff to P0-W07/P0-W20)

The check a future inventory-consistency gate would perform is: **"every
`unsafe` occurrence in the tree maps to a current inventory entry, and every
inventory entry maps to existing code"** — bidirectional consistency. This
policy defines the predicate only; classification, wiring, and enforcement
are the quality-gates and CI packages'.

## 8. Inventory pointer

The sole register of first-party unsafe segments is the
[unsafe inventory](unsafe-inventory.md); its schema, lifecycle, and update
rules live there and are governed by this policy's thresholds. No other
document may carry an unsafe register.
