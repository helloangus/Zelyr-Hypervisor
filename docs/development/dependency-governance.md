# Zelyr Dependency Governance

**Status:** Normative dependency policy.  
**Scope:** The review tiers, the fixed evaluation checklist, lifecycle and
escalation rules, consistency rules, and the [dependency
register](dependency-register.md) pointer. It selects no crate, designs no
wrapper API, and defines no build/lockfile mechanics; the wrapper/isolation
design is the consuming design's.  
**Version:** v0.1  
**Owner/change context:** P0-W18 dependency governance; operationalizes
ADR-006's permission for mature crates under TCB discipline.  
**Supersedes:** The absence of a dependency policy.

## 1. Purpose and fail-closed default

Dependency use is legitimate (ADR-006 permits mature crates). What is not
permitted is **unreviewed expansion of hypervisor-TCB dependencies**. The
default is fail-closed: **an unregistered dependency is unapproved.** The
rules bind decisions made after adoption; the register being empty at
adoption time means exactly zero approved dependencies — it is retroactive-
proof, not a placeholder.

## 2. Review tiers

| Tier | Definition | Examples of membership questions | Approval | Checklist depth |
|---|---|---|---|---|
| **D1** — host-only development dependency | Used exclusively on development machines or CI; never linked into any repository-produced runtime image and not needed by consumers of the repository's outputs | formatters' auxiliary tools, local generators, CI-only utilities | register entry approved in ordinary PR review | abbreviated (identity, purpose, license) |
| **D2** — general project dependency | Any reviewed dependency that is neither D1 nor D3 — for example linked into host-side tooling or test binaries | host test harness support, host analysis tools with in-repo entry points | full checklist recorded; ordinary PR review | full checklist |
| **D3** — hypervisor-TCB candidate | Linked into any repository-produced bare-metal image (hypervisor or validation guest), or otherwise part of the hypervisor TCB | anything destined for EL2 or the guest runtime | full checklist **plus** explicit maintainer approval recorded in the register | full checklist plus TCB proportionality justification |

Classification rules:

- Classification is by **what the dependency is linked into**, decided by the
  consuming design; when a dependency would serve both a host tool and a
  bare-metal image, the D3 boundary governs (or the uses are split into two
  recorded decisions).
- Reclassification upward (D1/D2 → D3) follows the D3 path in full, including
  maintainer approval; downward reclassification requires evidence that no
  bare-metal image links the dependency.
- The register records the tier of every entry; the tier is data, not prose.

## 3. Evaluation checklist

Every D2/D3 candidate (and D1's abbreviated set) is evaluated against this
fixed checklist; answers are recorded in the register entry, with sources.

| # | Dimension | Mandatory questions |
|---|---|---|
| 1 | Purpose and TCB proportionality | What does it provide? Could less dependency (smaller crate, in-house code, std facility) provide it? Is the TCB increase proportionate to that value? (D3: mandatory written justification) |
| 2 | `no_std` compatibility | Does it work in the target's `no_std` context? With default features disabled? What does it require of the allocator, if anything? (D3: mandatory; D2: as applicable) |
| 3 | License | Is its license compatible with the project's Apache-2.0 license (root `LICENSE`)? What attribution/notice obligations arise? |
| 4 | Maintenance health | Release activity, maintainer count, issue/PR responsiveness, commitment signals; is it a single-maintainer point of failure? |
| 5 | Unsafe footprint | How much `unsafe` does it contain and with what posture? How does its usage pattern align with the project's unsafe boundary classes (per the [unsafe policy](../security/unsafe-rust-policy.md), by reference)? Third-party unsafe cannot be modified by this project and is therefore weighted, not adopted. |
| 6 | Transitive dependencies | How many, of what quality, and are any of them themselves TCB-relevant? Does the total footprint stay proportionate? |
| 7 | Architecture applicability | Does it support the AArch64 bare-metal context (compilation, link model, no OS assumptions)? Does it hide host-OS assumptions that would fail on target? |
| 8 | Allocation behavior | Does it allocate? On which paths? Can allocation be bounded or avoided where the consuming design requires it? (D3: mandatory) |
| 9 | Stability and versioning | Versioning discipline (semver adherence), pre-1.0 churn risk, breaking-change history; what version is proposed and why? |
| 10 | Security posture | Known advisory history (for example RUSTSEC entries), response patterns, and current open advisories affecting the proposed version? |
| 11 | Alternatives considered | Which alternatives (including no dependency) were considered and why were they rejected? |

Rules: a checklist question answered "unknown" is an incomplete evaluation;
the evaluation cites its evidence sources (repository, registry, advisory
database) rather than assertions; the checklist may gain questions by
reviewed change but never loses the task-book dimensions (TCB, `no_std`,
license, maintenance, unsafe, platform risk).

## 4. Lifecycle, triggers, and escalation

### 4.1 Introduction

Path: classify tier → run checklist → obtain the tier's approval → record the
entry per the [dependency register](dependency-register.md) → only then
reference the dependency in the consuming design/manifest. The consuming
design owns how the dependency is wrapped or isolated.

### 4.2 Upgrades

- Patch-level upgrade of a D1/D2 dependency: ordinary review; register event
  records the new version.
- Minor upgrade, or any upgrade of a D3 dependency: re-run checklist
  dimensions affected by the change (new transitive dependencies, new
  unsafe, behavior changes) and record the delta event.
- Major upgrade: full re-evaluation with a new checklist record.
- **Expedited security path:** a security advisory affecting an approved
  dependency is acted on promptly; the record states the advisory, the
  affected versions, the decision (upgrade, mitigate, or accept-with-risk and
  expiry), and the approver. Accepting an unpatched advisory on a D3
  dependency is a maintainer decision with an expiry date.

### 4.3 Deprecation and removal

A dependency abandoned upstream, superseded, or no longer used gets a
deprecation event (replacement plan, removal target) and later a removal
event (confirming no manifest/design reference remains). Removal requires the
same reviewer level as its introduction.

### 4.4 Exceptions

An exception grants a time-boxed, owner-approved deviation (for example:
evaluation incomplete but needed for a bounded experiment; or a D2 dependency
temporarily linked into an image). Every exception records: what is excepted,
why, the compensating control, the expiry date, and the owner. Expired
exceptions are violations, not precedents. Exceptions never apply to the
fail-closed default itself.

### 4.5 ADR thresholds

`ADR Required` when a dependency decision would:

- introduce an alternative runtime or execution model into EL2 (for example
  an async executor or a global allocator policy change as an architectural
  commitment);
- alter the crate layering rules (Core/Arch/SoC/Board dependencies); or
- capture a core abstraction so that later replacement is itself an
  architecture change.

Ordinary D3 introductions are within ADR-006's permission and do not need an
ADR. Escalation follows the [ADR process](../adr/README.md).

## 5. Consistency rules

### 5.1 Register↔manifest

Every dependency named in any Cargo manifest must have a register entry with
a tier and a current evaluation; a manifest dependency without a register
entry is a review failure. The direction of authority is **register →
manifest**. Automated checking is Reserved (candidate quality-gates/CI
future class).

### 5.2 Unsafe interface with the unsafe policy

This policy assesses third-party unsafe (checklist dimension 5); the
project's own unsafe justification, inventory, and review mechanics belong to
the [unsafe Rust policy](../security/unsafe-rust-policy.md). That policy's
boundary categories (`arch-register`, `mmio-volatile`, `memory-mgmt`,
`low-level-struct`, `asm-glue`, `boot-state`) are the reference for
dimension 5's alignment question. W18 neither restates nor pre-empts the
unsafe mechanics; third-party unsafe is assessed, never adopted into the
first-party inventory. A divergence is reconciled in the same change that
delivers it.

### 5.3 Version reproducibility and lockfile

What is approved is what is built: the register pins the approved version(s),
and the build baseline's mechanism (the workspace `Cargo.lock`, owned by the
build-baseline design) makes the built version traceable to the approved one.
This policy defines no lockfile mechanics.

## 6. Mutation rules for this policy

- **Ordinary reviewed change:** adding checklist questions; clarifying
  definitions; adding informative examples.
- **Recorded policy decision** (issue and owner decision before the change):
  changing tier boundaries or approval authority; changing re-evaluation
  triggers; changing the exception mechanism.
- **ADR Required:** anything matching §4.5's thresholds, or a change that
  would abandon the fail-closed default or the register's authority.
