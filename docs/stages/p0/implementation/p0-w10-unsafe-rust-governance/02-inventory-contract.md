# P0-W10 Inventory Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W10 detailed design](README.md).

This file fixes the schema, lifecycle, and update rules of
`docs/security/unsafe-inventory.md`. The inventory is a register (data), not
a policy document; it starts with zero entries because no unsafe can exist in
the current tree.

## 2. Inventory identity and location

- Location: `docs/security/unsafe-inventory.md` — the directory the
  repository reserves for unsafe inventory/audits.
- Status header: normative register, version `v0.1`, owner/change context,
  supersedes: none, plus one pointer line: "Entry schema, lifecycle, and
  update rules: see the unsafe Rust policy at `docs/security/unsafe-rust-policy.md`;
  this register carries entries only."
- Single-source rule: every first-party unsafe segment in the repository is
  registered exactly once, here. No other document may carry an unsafe
  register, and code comments never substitute for an entry — the SAFETY
  comment back-links to the entry.

## 3. Entry schema (auditable fields)

Every entry carries all fields; a partially filled entry may exist only in
the `proposed` state and blocks merge until completed.

| Field | Content | Filled by |
|---|---|---|
| `id` | stable inventory identifier, format `U-<nnn>` assigned in creation order, never reused | author, at entry creation |
| `status` | `proposed` → `accepted` → (`superseded` | `removed`); lifecycle in §4 | author then reviewer |
| `title` | one-line statement of what the unsafe operation is | author |
| `boundary-category` | exactly one policy category (`arch-register`, `mmio-volatile`, `memory-mgmt`, `low-level-struct`, `asm-glue`, `boot-state`) | author, confirmed by reviewer |
| `location` | file and module path of the unsafe region (symbol-level once code exists) | author, in the same change as the code |
| `necessity` | why safe Rust cannot express the operation, including alternatives considered | author; from the approving design |
| `safety-preconditions` | the invariants that must hold for soundness | author; from the approving design |
| `establishment` | where each precondition is established (type invariant, earlier check, hardware state) | author |
| `failure-class` | consequence class if a precondition breaks, per the failure-classification baseline (P0-W14 subject; policy placeholder until then) | author |
| `authorizing-design` | link to the approved detailed design that names the segment | author |
| `owner` | the named person/role accountable for the boundary's soundness | author, confirmed at review |
| `review-record` | second reviewer, date, and link to the review evidence | reviewer |
| `validation` | host-test categories and QEMU/hardware validation covering the wrapped operation ([W08](../p0-w08-host-side-testing-baseline/README.md) categories where applicable) | author |
| `audit-status` | last audit date, auditor, and result | auditor, at each audit trigger |
| `permanence` | whether the boundary is expected to be permanent (with rationale) or scheduled for removal/reduction, and the intended replacement | author; updated at audits |

## 4. Entry lifecycle and update timing

```text
(proposed, in the change that introduces the unsafe)
  -> accepted (merge: all fields complete, review recorded)
  -> unchanged until an audit or design trigger
  -> superseded (boundary reworked: new entry created, old links forward)
  -> removed (unsafe eliminated: entry marked removed, kept for audit history)
```

Update rules the policy must state:

1. **Same-change rule:** an entry is created or updated in the same change as
   the unsafe code it describes; a change adding unsafe without its entry (or
   an entry without its code) fails review — and later fails the future
   consistency gate (policy §8).
2. **Pre-merge review:** `proposed` → `accepted` happens at merge review,
   with the second reviewer recorded.
3. **Audit triggers:** an entry is re-audited when its authorizing design
   changes in a way touching its preconditions, when its category's
   placement rules change, when the toolchain or a governing abstraction it
   relies on changes through a policy decision, and at each stage completion
   review that claims the containing subsystem works. Audit results are
   recorded in `audit-status`; a failed audit reopens the entry.
4. **No silent edits:** changing `boundary-category`, `safety-preconditions`,
   or `failure-class` after acceptance is a policy-threshold event, recorded
   in the entry's history section of the change, not a quiet field update.
5. **History retention:** superseded and removed entries remain in the
   register with their final status, preserving the audit trail ADR-049
   expects.

## 5. Relations to consumers

- **Gate predicate:** the bidirectional consistency check in policy §8 reads
  this register; the register's completeness is the gate's input.
- **Dependency unsafe:** third-party unsafe is not entered here; it is
  evaluated under [W18](../p0-w18-dependency-governance/README.md) intake.
  An entry's `establishment` field may reference an audited dependency
  abstraction as part of its argument.
- **Failure classification:** `failure-class` values follow
  the P0-W14 failure taxonomy once
  delivered; entries written before that delivery carry the policy's
  placeholder wording and are re-audited at the next trigger.

## 6. Explicitly excluded interfaces

The inventory authorizes no code, no tooling, and no automation in P0. It is
maintained by hand until a future design introduces tooling (which would then
consume the schema, not redefine it). An empty inventory must never be
populated with speculative or hypothetical entries; entries exist only for
real, merged unsafe.
