# P8-W15 Fixture Manifest Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W15 detailed design](README.md).

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Fixture manifest | one tracked manifest document, authored at implementation; home recorded in the implementation record (suggested: beside this design in the P8 implementation area) | this design, approved W03/W04 facts, machine-configuration citation | the sole authoritative declaration of source identity, configuration identity, route requirements, artifact digests, bootargs, initramfs content, and license metadata; it carries no policy prose beyond pointers |
| Acquisition/verification procedure | the manifest's procedure sections + this design's §4 | pinned identities | reproducible acquisition, digest verification, and evidence generation; it does not define Host loading mechanics or W16 harness behavior |
| Initramfs functional content table | manifest §5 (this design's table) | W09–W12 declared needs | functional requirements per program; not the Guest-side implementations and not a built-image claim |
| Machine-configuration citation | manifest §3 | approved v1 configuration record ([W14](../p8-w14-machine-abi-compatibility/README.md)/[W02](../p8-w02-machine-contract-governance/README.md)) | a citation binding the fixture to one approved configuration; it selects no machine value |
| Evidence records | `../../verification/p8-w15-reproducible-linux-fixture-verification.md` and `../p8-w15-reproducible-linux-fixture-record.md` (created when work starts) | actual procedure output | run/not-run evidence, digests, deviations; not part of the design |

The artifact named in the second column is the sole authoritative home for its
row's statements; other documents may link but must not duplicate or contradict.

## 2. Manifest schema

The manifest must state, per group, at minimum:

```text
fixture_identity
  fixture_id            stable identifier (e.g., p8-linux-fixture-a)
  machine_binding       citation of the approved v1 configuration record
                        (machine identity + version); blocked until approved
  status                draft | pinned | verified | invalidated
kernel_source
  upstream_identity     exact source identity: project, repository/origin
                        class, exact version tag or commit identity, and the
                        source's own integrity reference (e.g., release
                        signing/tag annotation), cited — not mirrored here
  version_selection     the rule used to select it (01 §3) and selection date
kernel_configuration
  config_identity       canonical identity of the exact kernel configuration
  config_content        the tracked configuration source or its digest plus
                        acquisition path; must be sufficient to reproduce the
                        configuration byte-for-byte
  config_rationale      per-option justification for non-default choices,
  keyed to the validation needs they serve
build_route
  route_descriptor      the recorded, reproducible route: required inputs,
                        ordered stages, and the toolchain/tooling identities
                        observed to produce the pinned artifacts; the concrete
                        tooling is an implementation choice recorded here,
                        not mandated by the design
  determinism_claim     which artifacts the route claims reproducible, and the
                        evidence class supporting the claim (01 §4)
guest_dtb
  provenance            citation of the W04 contract facts the DTB instance
  realizes and the generation source; digest recorded on verification
initramfs
  content               per 01 §5; component identities and licenses
boot_args
  bootargs_identity     citation of the approved W03 bootargs facts; the exact
  string recorded here must match that contract verbatim
artifacts
  per artifact (kernel image, DTB instance, initramfs): digest algorithm +
  digest, size class, and the route stage producing it; recorded when
  verified, never pre-filled
license_metadata
  per component: license identifier (kernel: GPL-2.0; BusyBox: GPL-2.0; others
  as pinned), source availability statement, and the redistribution position
  of README decision 1
```

Field rules: a `blocked` group (machine binding, bootargs, DTB provenance while
their contracts are unapproved) stays explicitly blocked — never provisionally
filled. The manifest is the only place consumers read artifact identities from;
a digest in any other tracked file is a single-source violation.

## 3. Version selection rules

- Select exact, immutable upstream identities: a tagged release or an exact
  commit identity — never a moving branch, "latest", or a distribution package
  whose content varies by repository state.
- Selection rule: the newest stable upstream release of the pinned kernel
  series that satisfies the approved boot/CPU-compatibility contracts' declared
  requirements; if no approved contract declares a minimum, the selection is
  recorded with the boot-evidence rationale and remains revisable via §6.
  Selection happens at implementation time and is recorded with date and
  resolution source (mirroring the P0-W02 toolchain-pin pattern).
- Configuration identity is selected once, reviewed per §6, and changed only
  with per-option justification tied to a declared validation need.
- The initramfs component set uses exact upstream identities under the same
  rules.

## 4. Acquisition, pinning, and verification procedure

The manifest's procedure sections must satisfy this contract:

1. **Acquire** each pinned source by its exact identity from its upstream
   origin class over the network; verify the source's own integrity reference.
   Failure to acquire or verify is a recorded blocker; substitution of a mirror
   or nearby version is prohibited without a §6 revision.
2. **Regenerate** the artifacts by the recorded route, or acquire pre-built
   artifacts only when the route's determinism claim covers them and their
   digests are the pinned ones; either way the artifact digests are computed
   and compared with the manifest.
3. **Verify conformance**: the bootargs string matches the approved W03 facts
   verbatim; the DTB instance matches the W04 contract facts (consistent with
   the W04 design's consistency checks); the initramfs content matches the §5
   table; the machine binding resolves to an approved configuration.
4. **Record evidence**: commands, outputs, digests, environment (host OS,
   toolchain identities), and timestamps into
   `../../verification/p8-w15-reproducible-linux-fixture-verification.md`;
   selected identities and manifest location into the implementation record.
5. **Re-verification** (reproducibility evidence): a second, independent
   execution of steps 1–3 must produce identical digests for every artifact the
   route claims reproducible. This re-derivation — not the first build — is
   the P8-V20 reproducibility evidence. Artifacts the route does not claim
   reproducible are recorded as pinned-only, with their digests still
   mandatory.

Failure boundary: a digest mismatch, a failed re-derivation, or an
unresolvable identity is recorded as failed/blocked with diagnosis. It is
never resolved by loosening the schema, changing identities silently, or
committing binaries (README decision 1).

## 5. Initramfs functional content by validation need

Required functional content (realizations are W15 implementation choices;
functional requirements and markers come from the owning designs):

| Program / capability | Validation need served | Needing package | Functional requirement source |
|---|---|---|---|
| Interactive shell | userspace-shell completion criterion | W09; W16/W19 | W09 design's acceptance path |
| `spin-cpu` | CPU-bound scheduler load | W11 | `../p8-w11-scheduler-linux-integration/02-observation-and-load-contracts.md` §5 |
| `sleep-wake` | timer/WFI/wakeup load with elapsed-time markers | W11 (also W08-adjacent evidence) | same §5 |
| `sched-fanout` | process fan-out load | W11 | same §5 |
| `mem-exercise` | allocator/page-table working-set exercise | W12 | `../p8-w12-linux-memory-model/02-probe-and-workload-contracts.md` §5 |
| `fork-storm` | COW + kernel/user-switch exercise | W12 | same §5 |
| `map-probe` | boundary/negative fault induction (N1–N5) | W12 | same §5 |
| Interrupt/timer observation | IRQ/timer counters and timing visible in userspace, written as console markers | W16/W18/W19 rows; task book tool categories | userspace-only observation per README decision 5; exact capability recorded with the configuration identity |
| Minimal process tools | fork/exec/wait baseline (process tool category) | W16 matrix | conventional userspace behavior; identities pinned |

Rules: every content entry names its needing package and requirement source;
an entry without a need is removed at review. The kernel configuration may add
the options these tools require (e.g., timer/counter interfaces), each recorded
in `config_rationale`. Performance tooling, debuggers, and convenience
utilities are Reserved (W17's intake) and must not enter the baseline content.

## 6. Versioning and update rules

- **Routine maintenance** (ordinary PR review + re-running the W15-DV04
  procedure): a pin bump to a newer exact upstream identity under the §3 rules;
  a content-table change with a named new need; a digest refresh after a route-
  deterministic rebuild; a record-keeping correction that changes no identity.
- **Policy decision required** (recorded issue and owner decision before the
  change): changing the pinning model or determinism-claim class; adding a
  binary-distribution posture (README decision 1 is stage-local policy);
  reclassifying Reserved content; switching the machine binding to a new
  approved machine version (with W14's C3 route already satisfied).
- **ADR-required:** any change that would make the fixture a distribution
  commitment, host a third-party binary service, or alter a decision reserved
  to the ADR baseline (e.g., touching ADR-055's open distribution question).
- Every manifest revision re-runs the §4 procedure for affected groups and
  updates consumer-facing identities in the same change; consumers are never
  left reading a stale identity.

## 7. Licensing and evidence-record rules

- Licenses of every pinned component are recorded in `license_metadata` with
  the component's identity. The repository's posture is definitions-plus-
  digests: it distributes no fixture binary, so no GPL source-corresponding
  obligation is triggered by the repository itself; anyone who builds and
  redistributes artifacts does so under the components' licenses, which the
  manifest states. Changing this posture is a policy decision (§6), and making
  the project a binary distributor is ADR-required.
- Evidence-record needs: procedure runs, digest verifications, and
  re-derivation results live in the verification record; identity selections
  and manifest location live in the implementation record; no evidence lives
  in the manifest beyond the pinned identities themselves.
- A fixture claim without digest evidence is invalid: the plan forbids
  asserting an artifact exists, and this contract makes digests the existence
  proof.
