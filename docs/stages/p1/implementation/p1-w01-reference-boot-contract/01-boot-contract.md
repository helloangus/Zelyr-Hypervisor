# P1-W01 Boot Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W01 detailed design](README.md).

## 1. Authoritative artifact groups

W01 is contract definition plus a small enforced boundary, so its logical
modules are authoritative artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Reference boot contract content | this design (materialized in `../p1-w01-reference-boot-contract-record.md` when implementation begins; assembled into the stage contract document by [P1-W12](../p1-w12-p1-documentation-handoff/README.md)) | this design's §2–§8, P0 contracts as observed | the sole statement of entry assumptions, rejection semantics, and the canonical recipe; it does not document runtime establishment (W02) or verdict rules (W10) |
| Pre-transfer validation + rejection reporter | the boot entry module delivered by W02, implementing [02-entry-validation-contracts.md](02-entry-validation-contracts.md) | §5 boundary semantics, §3 entry checks | an enforced boundary with fixed behavior; it does not report anything about post-transfer conditions |
| Canonical invocation recipe | this design's §7 fields, exact spelling recorded at implementation | P0-W09 runner contract, P0 artifact baseline | one recipe used by W02 evidence, W10 regression, and W11 scenarios; it does not define runner behavior |
| Implementation record | `../p1-w01-reference-boot-contract-record.md` (created when work starts) | actual decisions taken | implementation-selected fields, changed artifacts, deviations; no command logs |
| Verification record | `../../verification/p1-w01-reference-boot-contract-verification.md` (created when evidence exists) | actual reviews and runs | run/not-run evidence per [the validation matrix](03-implementation-and-review.md); not part of the design |

The artifact named in the second column is the sole authoritative home for its
row's statement; other documents link to it and must not duplicate or
contradict it.

## 2. Canonical boot path

The canonical reference boot path is:

```text
QEMU system emulator, aarch64
  machine:          virt, with virtualization enabled (the property spelling
                    is consumed from the P0 runner contract and recorded)
  CPU model:        exactly one fixed AArch64 CPU model with EL2 support,
                    selected at implementation from the reference platform's
                    supported set and recorded (suggested candidate:
                    cortex-a57; the selection, not this suggestion, is
                    normative once recorded)
  CPU count:        1 (single-CPU canonical path; multi-CPU is Reserved, P3)
  memory:           one fixed size, recorded at implementation, at or above
                    the §4.4 minimum-memory rule
  image:            the P1 hypervisor image as produced by the P0 AArch64
                    target/build baseline, delivered by the platform's direct
                    kernel loading mechanism
  firmware:         the emulator's built-in direct-kernel boot path only;
                    no U-Boot, no TF-A, no UEFI
  console:          the reference platform's first serial port (used only by
                    the rejection reporter pre-transfer and, later, by the
                    W06 channel)
```

Properties the canonical path guarantees to the image (consumed contract of
the direct-kernel boot mechanism, verified by W01-DV02 review and by the
first boots; not re-established by W01 code):

- the boot CPU enters the image entry point in AArch64 execution state with
  `CurrentEL` reporting EL2, in Non-secure state;
- the MMU is disabled for the boot CPU and translation controls hold reset
  values;
- data and instruction caches are disabled; no cache maintenance is required
  before the image's own first writes;
- all interrupt sources are architecturally masked or routed such that the
  image takes no exception before it establishes its own handling (the image
  additionally masks `DAIF` itself in the entry and does not rely on the
  firmware state);
- general registers other than those named in §4.2 carry no load-bearing
  content.

The image entry point is the entry symbol of the P1 image as fixed by the
[P1-W02](../p1-w02-minimal-rust-el2-runtime/README.md) design; the load
address and image form are consumed from the P0 target/artifact baseline and
recorded in the implementation record. W01 defines no linker layout.

## 3. Entry-state contract (what the image may rely on after the pre-transfer tier)

Once the pre-transfer tier of
[02-entry-validation-contracts.md](02-entry-validation-contracts.md) has
passed, the boot path guarantees the following to every downstream package.
These statements are the authoritative answer to "which firmware register
contents may W02 rely on?"

| # | Field | Guaranteed value | How established | Residual assumption |
|---|---|---|---|---|
| E1 | Exception level | EL2 (`CurrentEL.EL == 0b10`) | pre-transfer check T1 | none — checked |
| E2 | Execution state | AArch64 | canonical-path property (§2); not separately checked pre-transfer | recorded assumption of the direct-kernel mechanism |
| E3 | Security state | Non-secure | canonical-path property; **not directly observable at EL2 without exception machinery P1 does not own before W05** | recorded assumption per ADR-008; defense-in-depth re-derivation is W03's required-fact set |
| E4 | Boot CPU | the single CPU firmware entered; no secondary CPUs are released or expected | canonical path is `-smp 1` | none in the canonical environment; multi-CPU behavior is Reserved (P3) |
| E5 | MMU / caches | MMU off; D-cache and I-cache off; reset translation controls | canonical-path property (§2) | P1 enables neither before the W08 transition; the entry does not enable them |
| E6 | Boot CPU identity | `MPIDR_EL1` readable and valid | architectural guarantee at EL2 | value is a recorded fact (W03), not a check |
| E7 | DTB pointer | `x0` non-zero, retained for P2 | pre-transfer check T2 (non-zero only) | alignment and content are **not** validated in P1 (P2 owns DTB intake) |
| E8 | Other registers | `x1`–`x3` and all other general registers: no load-bearing content | canonical-path property; the image establishes everything else itself | none — downstream designs must not consume them |

The entry code itself establishes, and therefore never assumes: stack
selection and top, `DAIF` masking, SPSel discipline, BSS state, and all EL2
control-register content that later phases own (W04). "Must not rely on
accidental firmware register contents" is the W02 runtime theme; W01's
contract is the part of that theme fixed at the boundary.

## 4. Boot parameters, DTB treatment, and memory

### 4.1 Boot parameters

`x0` carries the physical address of the platform-supplied device tree blob.
`x1`–`x3` are reserved: the contract assigns them no meaning, downstream
designs must not read them, and the entry's Rust transfer passes them solely
so that W02's `BootContext` can retain them uninterpreted for future
contracts (their retention is a W02 concern, not a reliance).

### 4.2 DTB treatment

P1 retains the DTB pointer and never examines the blob. Consequences, which
are scope statements and not laziness: P1 performs no discovery, accepts no
platform variation through the DTB, and hands the pointer to P2 unchanged.
The only pre-transfer validation is T2 (non-zero). Alignment, size, and
content validation are P2 intake responsibilities. A `DTB` rejection therefore
means "the canonical environment did not deliver the contractually required
input", not "the DTB is invalid".

### 4.3 Boot CPU

The boot CPU is whatever single CPU the canonical path entered. P1 records its
affinity (W03) but performs no topology reasoning, no CPU selection, and no
secondary-CPU handling. Any multi-CPU entry environment is outside the
canonical contract; its behavior is undefined by this contract and Reserved to
P3.

### 4.4 Minimum memory

The contract requires one recorded minimum-memory value for the canonical
path, selected at implementation to satisfy all of: image footprint (as
linked per the P0 artifact baseline), the W02 boot stack, the platform DTB
and its placement, and a stated slack factor — with the arithmetic recorded in
the implementation record. P1 performs **no** runtime memory-adequacy check:
verifying usable physical memory is discovery, which is P2 scope. The minimum
is therefore a recipe constraint (a property of the invocation), not an
entry-time check.

## 5. Rejection boundary

The boundary classifies environments, not DTB content or memory adequacy.
Classification is exhaustive over the pre-transfer tier:

| Check | Disallowed condition | Rejection reason token | Rationale |
|---|---|---|---|
| T1 | `CurrentEL.EL != 0b10` (firmware entered at EL1, EL0, or EL3-visible state) | `EL` | the entire stage is an EL2 runtime; continuing at EL1 would silently build on wrong privilege (task book P1-V02) |
| T2 | `x0 == 0` (no DTB pointer delivered) | `DTB` | the canonical path's one required input is absent; continuing would fabricate a canonical boot |

Boundary semantics, all **Required**:

- R1 Rejections occur strictly before any runtime establishment (no stack
  switch, no BSS write, no Rust execution).
- R2 Each rejection emits exactly one line in the fixed token format of
  [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §3.
- R3 After emitting, the boot CPU executes the bounded stop; execution never
  reaches the runtime, no W09 runtime marker is ever emitted, and nothing
  on the boot path retries, falls back, or "continues degraded".
- R4 The bounded stop is guaranteed unconditionally; the reason output is
  required evidence on the reference platform (where the raw polling write is
  reliable without peripheral initialization) and best-effort by design
  elsewhere — a rejection whose output was not observed is still a rejection,
  and is recorded as such in evidence.
- R5 No rejection path writes to any register whose later ownership belongs
  to another package; the boundary owns nothing beyond its own decision.

Environments that pass T1/T2 but violate a §3 *residual assumption* (E2, E3,
E5) are outside the canonical contract but **not** rejection classes: they
cannot be detected pre-transfer, so the contract records them as assumptions
with the evidence obligations of §9, not as checked conditions. Inventing a
check that cannot be implemented pre-transfer would move the boundary after
transfer and break R1.

## 6. Transfer guarantee

Reaching the W02 Rust entry point (`el2_rust_entry`,
[P1-W02](../p1-w02-minimal-rust-el2-runtime/README.md)) implies that T1 and
T2 passed and that §3's checked fields hold. Consequences:

- W02-owned code records the W09 `entry` lifecycle events at the start of the
  Rust entry on this authority; W01 implements no recording.
- Every downstream package may code against §3's table without re-checking
  E1/E7. W03's re-derivation of the execution level is defense in depth
  against environment drift, not a prerequisite of its inventory.
- The guarantee's failure mode is not silent: if the entry module's tier
  implementation diverged from this contract, W11's NC1 scenario (unsupported
  environment) and W10's normal boots are the standing detectors.

## 7. Canonical invocation recipe

The recipe is a fixed field set, documented as the one reference invocation;
its exact machine-readable spelling is recorded at implementation (resolved
decision 7) because the runner entry is a P0 deliverable:

| Field | Contract requirement |
|---|---|
| machine | `virt` with virtualization enabled |
| CPU | exactly one fixed EL2-capable model, single CPU |
| memory | the recorded minimum-memory value (§4.4) |
| image | the P0-built P1 hypervisor image, unmodified |
| loader | direct-kernel mechanism only |
| console | the reference platform's first serial port, captured by the runner |
| timeout/exit | consumed from the P0-W09 runner contract; not defined here |

The Reserved second recipe (U-Boot/TF-A delivery to Non-secure EL2) is
recorded in the implementation record as a named alternative with its intended
role (later-stage determinism and real-firmware rehearsal); it has no
acceptance criteria in P1 and must not be presented as a supported path.

Parameter variations of the canonical recipe (RAM size, CPU model, machine
options) are Reserved: W10 may add bounded variations for its own scenarios,
and W11 varies the environment for NC1/NC2; neither falls back into this
contract as a second canonical path.

## 8. Layering and reserved-scope reconciliation

The reference boot path necessarily contains one reference-platform constant
(the raw UART base address used only by the rejection reporter) and one
reference-mechanism dependency (the direct-kernel entry conventions). The
ADR's layering rules (ADR-041, ADR-043, ADR-052; P0-W11 guardrails) forbid
platform-name branches in generic **Core**; the P1 boot entry is not generic
Core — it is the reference-platform bring-up path, and it is expected to be
replaced piecewise by P2 discovery. Reconciliation statements, which the
implementation record must preserve:

- the rejection reporter's UART constant is documented as a
  reference-platform fact, confined to the boot entry module, shared by
  contract with W02's early diagnostic writer (single-source rule,
  [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §6);
- no other package or module may consume the constant; W06's console defines
  its own reference-console assumption per its plan's scope;
- the direct-kernel entry conventions are a consumed property of the canonical
  recipe, not a module dependency;
- nothing in W01 authorizes a `qemu`/board-name conditional anywhere in
  generic code.

Reserved scope touched but not implemented: the second recipe (decision 1),
multi-CPU environments (§4.3), DTB validation (§4.2), memory adequacy (§4.4).
Out of scope and not touched by any W01 artifact: Guest entry protocols,
EL3/TF-A bring-up, Orange Pi boot, platform discovery.

## 9. Acceptance evidence definitions

P1-V01 (normal path) requires, per the task book: entry/image assumptions,
EL/security state, boot parameters, DTB treatment and failure policy are
documented, and reference boot reaches EL2. W01's contribution and its
evidence split:

| Evidence | Produced by | Contract location |
|---|---|---|
| Contract content review (assumptions complete, no unchecked assumption stated as checked) | W01-DV02 review | §3, §4, §5 |
| Boundary semantics review (exhaustive classes, R1–R5 hold by construction) | W01-DV03 review | §5 |
| Canonical boot reaching validated EL2 | executed boot through W10's regression on the integrated path | §2, §6 — execution evidence recorded by W10, consumed by W01 |
| Recipe usability by automation | W10-DV01 consumability review | §7 |

P1-V02 (unsupported path) requires a rejection with a reason and no normal
continuation. W01's contribution: the boundary classes and reason vocabulary
(§5), the reporter and stop contracts ([02-entry-validation-contracts.md](02-entry-validation-contracts.md)),
and scenario NC1's expected observable. Executable rejection evidence is
produced by W11 executing NC1 through W10's conventions; W01's own validation
remains review plus the evidence definitions, because no image or runner
exists at W01's point in the chain. This deferral is stated, not silent: the
verification record must contain explicit `not run — deferred to W10/W11`
entries for the two executed proofs.
