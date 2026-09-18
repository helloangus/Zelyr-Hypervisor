# P0-W18 Dependency Register Schema

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W18 detailed design](README.md).

## 1. Purpose and authority

The dependency register (`docs/development/dependency-register.md`) is the
sole authoritative record of which dependencies are approved, at what tier,
on the strength of which evaluation, and with which lifecycle history. It
starts **empty**: at adoption time the repository has zero dependencies
(observed state — no workspace, manifest, lockfile, or vendored source
exists), and the empty register is the truthful representation of that, not a
placeholder to be filled with examples.

Rules the register document must state up front:

- The register records **real decisions only**. Hypothetical or illustrative
  candidates are rehearsal material for verification records, never register
  entries.
- The register is append-oriented: lifecycle events are added with dates and
  references; history is corrected by adding a correcting event, not by
  rewriting past entries.
- The register never overrides the policy; a conflict between an entry and
  the policy is a review failure resolved in one change.

## 2. Entry schema (one section per dependency)

Each approved dependency gets one section with these fields:

| Field | Content |
|---|---|
| Identity | dependency name; source (registry/URL); approved version(s) |
| Tier | `D1` / `D2` / `D3`, with the one-line classification rationale (what it is linked into) |
| Evaluation record | per-question answers to the policy checklist with evidence sources and evaluation date; "unknown" is prohibited in a complete evaluation |
| TCB proportionality | D3 only: the written justification required by checklist dimension 1 |
| Approval | approver (role and name), approval date, and the decision reference (PR or issue link once the integration workflow's PR path produces them) |
| Unsafe summary | the footprint conclusion from checklist dimension 5 |
| Transitive summary | the footprint conclusion from checklist dimension 6 |
| License and notice status | compatibility conclusion; notice obligations and whether they have been discharged (notice production is Reserved until the first real dependency) |
| Re-evaluation triggers | which future events force re-review (defaults per the policy's upgrade rules, plus any dependency-specific ones) |
| Status | `approved` / `deprecated` / `removed` / `exception-active` |

## 3. Lifecycle event records

Events are appended either inside the dependency's section or in a
chronological event log section, in one consistent style chosen by the
implementer and stated in the register's header. Each event records: date,
event type, actor/approver, and decision reference.

| Event | Required content |
|---|---|
| `introduction` | identity, tier, evaluation record reference, approval, approved version(s) |
| `upgrade` | from-version → to-version, trigger (routine / advisory / feature need), re-evaluation delta or full re-evaluation reference, approval |
| `advisory-response` | advisory identifier, affected versions, decision (upgrade / mitigate / accept-with-risk), expiry date when risk is accepted, approval |
| `deprecation` | reason (upstream abandonment / superseded / unused), replacement plan, removal target |
| `removal` | confirmation that no manifest or design reference remains, approval |
| `exception` | what is excepted, why, compensating control, expiry date, owner; exceptions reference the dependency section they modify |
| `reclassification` | from-tier → to-tier, evidence, approval (upward follows the full D3 path) |

## 4. Maintenance rules

- The register is updated in the **same change** as the decision it records
  (the introducing/upgrading PR edits the register; the integration workflow's
  review then sees policy and record together).
- When the first Cargo manifest appears (owned by the build-baseline design),
  the register↔manifest consistency rule from the policy becomes checkable;
  until then it is a binding forward requirement.
- The register's header must carry the status/version header required by
  `docs/README.md`, state that it starts empty, and link to the policy
  instead of restating any rule.

## 5. Explicitly excluded content

No crate name may appear anywhere in the register at adoption time (there are
no dependencies); no evaluation template may be pre-filled with illustrative
answers; no manifest snippet, lockfile fragment, or wrapper-API sketch is
permitted. The register is a decision ledger, and inventing entries would
falsify the audit trail it exists to provide.
