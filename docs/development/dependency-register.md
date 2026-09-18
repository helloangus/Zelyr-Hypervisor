# Zelyr Dependency Register

**Status:** Normative decision ledger — **zero entries; the repository
approves no dependencies.**  
**Version:** v0.1  
**Owner/change context:** P0-W18 dependency governance; entries are created
only by real decisions under the [dependency
governance](dependency-governance.md) policy.  
**Supersedes:** None.

Rules: this register records **real decisions only** — hypothetical or
illustrative candidates are rehearsal material for verification records,
never entries. It is append-oriented: lifecycle events are added with dates
and references; history is corrected by adding a correcting event, not by
rewriting past entries. This register never overrides the policy; a conflict
between an entry and the policy is a review failure resolved in one change.
Entry schema and event types are defined by the policy's governing design;
entries carry all schema fields and a complete evaluation ("unknown" is
prohibited).

## Entries

None. At adoption time the workspace exists with zero dependencies named in
any manifest (verified at register creation), so the empty register is the
truthful representation of the repository's dependency state. No crate name
appears anywhere in this register; no evaluation is pre-filled with
illustrative answers.

## Lifecycle event log

None.

## Maintenance

- This register is updated in the **same change** as the decision it records.
- The policy's register↔manifest consistency rule (§5.1) is checkable from
  adoption: the current manifests name zero dependencies, matching the zero
  entries above. The direction of authority is register → manifest.
