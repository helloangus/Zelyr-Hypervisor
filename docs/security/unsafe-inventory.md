# Zelyr Unsafe Inventory

**Status:** Normative register — **zero entries; no first-party `unsafe`
exists in the repository.**  
**Version:** v0.1  
**Owner/change context:** P0-W10 unsafe Rust governance; entries are created
only by real, merged unsafe changes under the policy's review rules.  
**Supersedes:** None.

Entry schema, lifecycle, and update rules: see the [unsafe Rust
policy](unsafe-rust-policy.md); this register carries entries only.

## Entries

None. An empty inventory must never be populated with speculative or
hypothetical entries; entries exist only for real, merged unsafe.

## Entry format (for the first entry)

Each entry carries all fields of the policy-governed schema: `id`
(`U-<nnn>`, creation order, never reused), `status`
(`proposed` → `accepted` → `superseded` | `removed`), `title`,
`boundary-category` (exactly one of `arch-register`, `mmio-volatile`,
`memory-mgmt`, `low-level-struct`, `asm-glue`, `boot-state`), `location`,
`necessity`, `safety-preconditions`, `establishment`, `failure-class`,
`authorizing-design`, `owner`, `review-record`, `validation`,
`audit-status`, `permanence`. Field semantics, lifecycle, and update rules
are defined in the policy and its governing design; a partially filled entry
may exist only in the `proposed` state and blocks merge until completed.

## History

Superseded and removed entries remain in this register with their final
status, preserving the audit trail (ADR-049). The register has no history
yet.
