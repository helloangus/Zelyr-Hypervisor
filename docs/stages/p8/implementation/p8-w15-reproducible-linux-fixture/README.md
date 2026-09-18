# P8-W15 Reproducible Linux Fixture — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The definition, pinning, acquisition, and verification procedure for
a versioned or fully reproducible Linux test asset, as required by
[P8-W15](../../plans/p8-w15-reproducible-linux-fixture.md).  
**Owner-change context:** P8-W15 implementation handoff; this package defines
the fixture contract and evidence — it does not build, commit, or assert the
existence of the fixture artifacts.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W15. The plan makes W15 the
package that gives P8 a versioned or fully reproducible Linux test asset:
source/version, configuration, build route, initramfs, bootargs, machine
configuration, and the minimal shell/CPU/memory/IRQ/timer/process/stress tool
set. This design converts that into (a) a fixture manifest contract that
carries provenance, pinning, and reproducibility metadata, (b) an
acquisition/pinning/verification procedure with explicit evidence, (c)
initramfs functional content derived from declared validation needs, and (d)
versioning, licensing, and update rules. It deliberately does **not** select a
distribution, define Host filesystem loading, write package-manager
instructions, add performance tooling, or assert that any artifact exists; the
fixture build itself is implementation work that happens only after this
contract is approved and its prerequisites deliver.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [Fixture manifest contract](01-fixture-manifest-contract.md) — read before
  authoring or reviewing the manifest, the pinning metadata, the initramfs
  content table, or the update/licensing rules.
- [Implementation and review](02-implementation-and-review.md) — read before
  executing; ordered steps, validation matrix, failure/security/observability
  model, and handoff checklist.

Before editing, the agent must also satisfy the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, ADR baseline, P8 task book, and
the P8-W15 plan. Nothing here claims that a kernel has been pinned, built, or
booted; P8-V20 requires the pinned or reproducibly generated fixture
*definition*, and is explicit that it is not a built-image claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → approved boot/DTB contract
facts → P8-W15 plan → this design → Coding Guidelines. In particular:

- ADR-049 makes "Linux boot regression" part of the verification strategy and
  ADR-010 brings in a customized Linux after the Validation Guest; both require
  a stable, reviewable test asset rather than a moving one. The choice of a
  pinned-source-plus-recorded-build-route model (versus committing binaries) is
  stage-local design freedom this design owns, with the licensing rationale
  recorded ([01 §6](01-fixture-manifest-contract.md)).
- The task book boundary for P0 is "reproducible fixtures and evidence; no
  toolchain redesign", and P0-W19
  (`../../../p0/plans/p0-w19-reproducible-development-workflow.md`) requires
  that every path be executable from repository declarations alone. The fixture
  contract therefore records provenance and route *requirements*; concrete
  builder tooling is an implementation choice recorded in the manifest, not
  mandated here (the plan's "without choosing implementation tooling").
- The boot contract ([P8-W03](../p8-w03-linux-boot-contract/README.md)) owns
  the boot-input facts (Image, DTB, optional initramfs, bootargs, artifact
  regions/lifetime) and the Guest DTB contract
  ([P8-W04](../p8-w04-guest-dtb-contract/README.md)) owns DTB content; W15 pins
  and verifies instances of those contracts, never redefines them.
- The machine configuration the fixture targets is the approved v1
  configuration per [P8-W14](../p8-w14-machine-abi-compatibility/README.md);
  the fixture is tied to the approved machine version and is invalidated by a
  machine-version change.

Classification:

- **Required:** the fixture manifest schema (provenance, pinning, digests,
  route descriptor, bootargs, machine-configuration citation, initramfs
  functional content, license metadata); the acquisition/pinning/verification
  procedure with evidence destinations; the versioning/update rules; the
  consumer handoff to W09–W10 and W16–W19.
- **Reserved:** performance tooling and profiling assets (P8-W17's need);
  multi-fixture matrices beyond the declared content (e.g., alternate
  configurations); binary artifact hosting/distribution if an authority ever
  requests it.
- **Out of Scope:** distribution/rootfs selection (ADR-055 stays open);
  Host-side loading mechanics; package-manager instructions; the fixture build
  itself and any built-image claim; Linux feature development; machine ABI
  values; validation harness mechanics (W16).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W03/W04 and P0 reproducibility governance | [ledger](#current-state-findings-and-goal-to-baseline-ledger); [manifest contract](01-fixture-manifest-contract.md) §1 | W15-DV01 source review |
| Define fixture provenance and reproducibility metadata | [manifest contract](01-fixture-manifest-contract.md) §2–§4 | W15-DV02 schema review |
| Define initramfs functional content by validation need | [manifest contract](01-fixture-manifest-contract.md) §5 | W15-DV03 content review |
| Relate artifact inputs to boot and Guest-only DTB contracts | [manifest contract](01-fixture-manifest-contract.md) §3, §6 | W15-DV02/DV03 |
| Review versioning, licensing, and evidence-record needs | [manifest contract](01-fixture-manifest-contract.md) §6–§7; [workflow](02-implementation-and-review.md) §2 step 5 | W15-DV05 policy review |
| P8-V20 pinned or reproducibly generated fixture definition | [workflow](02-implementation-and-review.md) §2 steps 3–4, §3 | W15-DV04 procedure exercise |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation scaffold only. No fixture of any kind exists — no kernel source
pin, no configuration, no initramfs definition, no artifact digests, no
`tests/` content (placeholder `.gitkeep` only), and no build route. The boot
and DTB contracts are plans. Each ledger row states the missing foundation the
plan outcome requires and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Versioned or fully reproducible fixture definition (P8-V20) | No fixture definition exists | The manifest contract (01 §2–§4) and, at implementation, the authored manifest | Without a tracked definition there is nothing to pin, acquire, or verify | W15 (this design) | W15-DV02; W15-DV04 exercise |
| Pinning actually verifiable | Nothing pinned; no digest anywhere | Acquisition/verification procedure producing digest and provenance evidence | A pin without a verification path is an assertion, not reproducibility | W15 procedure (01 §4) | W15-DV04 |
| Boot-contract conformance | W03 is a plan | Approved W03 boot-input facts as the manifest's citation targets | Fixture artifacts must be instances of the boot contract, not parallel decisions | W03/W04 designs | W15-DV01; DTB consistency checks with W04 |
| Initramfs carries the validation tool set | No initramfs; W11/W12 declare needs | Functional content table (01 §5) integrating W09–W12 needs and the plan's tool categories | The tool set exists to serve declared validation needs, not convenience | W11/W12 needs; W15 table | W15-DV03; consumer reviews |
| Machine configuration basis | v1 facts unapproved (ADR section 18) | Machine-configuration citation targeting the approved v1 configuration | The fixture must target one approved configuration or it cannot anchor drift detection | [W14](../p8-w14-machine-abi-compatibility/README.md); W02 route | W15-DV01; blocked state until approved |
| Reproducible from declarations | No recorded route; P0-W19 planned | Route descriptor requirements (inputs, determinism, recorded toolchain identity) | P0-W19's clone-to-evidence rule extends to the fixture | P0-W19; W15 route rules (01 §4) | W15-DV04 re-derivation evidence |
| Consumers receive stable expectations | W09–W19 are plans | Handoff section with the stability guarantees consumers may rely on | Consumers must code against markers and artifact identities, not guesses | W15 (this design) | W15-DV06 consumability review |

No row authorizes W15 to build artifacts or select tooling; where an upstream
contract delivers differently, [the workflow](02-implementation-and-review.md)
§1 failure boundary applies.

## Resolved design decisions and their authority

1. **Definitions are tracked; binaries are not.** The repository carries the
   fixture manifest, configuration identity, route descriptor, and digests;
   kernel/initramfs/DTB artifacts are acquired or regenerated per the manifest
   and verified by digest, and are never committed. Rationale: binary
   redistribution raises licensing obligations (GPL-2.0 kernel, GPL BusyBox)
   that no authority has accepted; committing binaries would also break the
   "reproducibly generated" goal. Stage-local design freedom owned here; a
   future distribution decision is Reserved.
2. **Pinning model: exact upstream source identity + configuration identity +
   digest-verified artifacts + recorded route.** Version selection rules are
   recorded ([01 §3](01-fixture-manifest-contract.md)); the concrete version is
   selected at implementation time. Rationale: the plan requires "versioned or
   fully reproducibly generated"; this model satisfies both readings and keeps
   ADR-055 (distribution choice) untouched.
3. **The manifest is the single authoritative fixture declaration.** All
   consumers (W09–W19) read the manifest; no consumer may carry a private
   artifact expectation. Suggested home at implementation: a manifest document
   under the P8 implementation area next to this design, named in the
   implementation record. Rationale: mirrors the P0-W02 single-source pattern
   for the toolchain baseline.
4. **Initramfs content is derived from declared validation needs.** The table
   in [01 §5](01-fixture-manifest-contract.md) maps each program to its
   needing package (W09 shell path; W11 scheduler loads; W12 memory workloads
   and probes; W10/W16–W19 shared needs) and to the plan's tool categories
   (shell/CPU/memory/IRQ/timer/process/stress). Rationale: the plan's "by
   validation need"; prevents convenience tooling from accreting.
5. **IRQ/timer observation is console-marker based.** Guest-side IRQ/timer
   evidence tools are userspace programs writing console markers (e.g., sleep
   timing, `/proc`-style interrupt counts if the pinned kernel exposes them);
   no kernel module or Guest driver is designed here. Rationale: keeps the
   fixture inside userspace content; kernel-internal instrumentation is a
   configuration matter recorded in the configuration identity.
6. **Fixture identity is bound to the approved machine version.** A
   machine-version change (W14 C3) invalidates the fixture and requires a
   manifest revision with re-verification. Rationale: the fixture anchors
   P8-V19's "same approved configuration"; a stale anchor would defeat drift
   detection.
7. **No build occurs under this contract until its prerequisites are
   approved.** Until the boot/DTB contracts and machine configuration are
   approved, acquisition is blocked and P8-V20 records the procedure exercise
   against the contract only. Rationale: mirrors W14's blocked-until-approved
   treatment and the plans-index evidence rule.

## Work breakdown and loading order

1. Read [the fixture manifest contract](01-fixture-manifest-contract.md) to
   understand the artifact groups, manifest schema, pinning and verification
   procedure, initramfs content table, and versioning/licensing rules.
2. Read [the implementation and review workflow](02-implementation-and-review.md)
   to author the manifest, exercise the acquisition/verification procedure,
   and record evidence.
3. Author and review in the order given there: source review, manifest,
   procedure exercise, policy review, consumer handoff.
4. Store procedure evidence in
   `../../verification/p8-w15-reproducible-linux-fixture-verification.md`, and
   record the manifest location, selected version identities, and deviations
   in `../p8-w15-reproducible-linux-fixture-record.md` only when implementation
   begins. Neither this design nor a written record may claim W15 complete,
   and no record may state that a fixture binary exists unless its digest
   evidence is recorded.

## Explicitly excluded interfaces

W15 designs no code interface — no type, function, crate, module, kernel
patch, kernel module, or harness internal; no distribution or rootfs; no
Host-side loading mechanism; no package-manager instructions; no performance
tooling; no machine ABI value, bootargs value, or DTB property value (those
are W03/W04/W02 facts the manifest cites). The only artifact fixed here is the
fixture manifest contract in
[01-fixture-manifest-contract.md](01-fixture-manifest-contract.md). Builder
tooling, fetch mechanics, and storage locations for acquired artifacts are
implementation choices recorded in the manifest, not interfaces designed here.

## Downstream handoff

- **W09** ([design](../p8-w09-virtual-console-single-cpu-linux/README.md)) and
  **W10** ([design](../p8-w10-linux-smp-bringup/README.md)) receive the
  guarantee that the fixture's boot inputs (Image, DTB instance, initramfs,
  bootargs) conform to their contracts with stable identities and markers, so
  their boot evidence is attributable to a fixed asset.
- **W11** ([design](../p8-w11-scheduler-linux-integration/README.md)) and
  **W12** ([design](../p8-w12-linux-memory-model/README.md)) receive the
  realization of their declared workload programs and marker formats
  (`spin-cpu`, `sleep-wake`, `sched-fanout`; `mem-exercise`, `fork-storm`,
  `map-probe`) with the machine-readable formats they specified.
- **W16** ([design](../p8-w16-automated-linux-regression/README.md)) receives
  the manifest as the sole source of artifact identities, digests, and
  acquisition/verification steps for its automation; W16 owns storage location
  policy for acquired artifacts at run time.
- **W17** ([design](../p8-w17-linux-performance-baseline/README.md)) may
  request Reserved performance tooling through a manifest revision; W15 owns
  the intake rule, not the tooling choice.
- **W18** ([design](../p8-w18-security-isolation-regression/README.md)) and
  **W19** ([design](../p8-w19-validation-guest-dual-track/README.md))
  receive the same stable-fixture expectations for their regression and
  dual-track runs.
- **[W14](../p8-w14-machine-abi-compatibility/README.md)** receives the
  fixture-identity binding: a machine-version change invalidates the fixture
  and is visible in the manifest.
- **P0-W19 alignment:** the fixture procedure must slot into the
  clone-to-evidence workflow as the Linux-fixture step; deviations route back
  to this design.
