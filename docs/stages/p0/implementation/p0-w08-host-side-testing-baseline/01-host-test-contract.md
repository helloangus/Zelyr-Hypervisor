# P0-W08 Host-Test Baseline Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W08 detailed design](README.md).

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Host-test baseline contract | `docs/testing/host-test-baseline.md` | this design, ADR-049 layering, W02 toolchain guarantee, W03-delivered workspace conventions | the sole normative home of the testable-logic boundary, organization rules, execution entry, coverage categories, and proof boundary; it does not define gates (W07), CI (W20), or the workspace itself (W03) |
| Minimal baseline test | one placeholder test inside the W03-delivered workspace, placed per its conventions | the entry contract | proof that the entry executes and reports truthfully; it is not product coverage and asserts nothing beyond entry health |
| Documentation routing | one row in `docs/README.md`; a line in `docs/testing/README.md` pointing to the contract is allowed only if it does not restate policy | contract location | discoverability; no policy duplication |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; never a completion claim |
| Implementation record | `../p0-w08-host-side-testing-baseline-record.md` (created when work starts) | decisions taken, entry spelling, placement decision | changed artifacts and deviations; no command logs |
| Verification record | `../../verification/p0-w08-host-side-testing-baseline-verification.md` (created when evidence exists) | actual commands and output | run/not-run evidence per the validation matrix; not part of the design |

The record and verification paths are future locations; this design does not
create them.

## 2. Testable-logic boundary

The contract must classify future logic into two boundaries and state the
rule that separates them.

**Host-verifiable classes** — logic whose semantics are fully determined by
inputs and pure state, with no reliance on EL2 execution, device timing, cache
or TLB behavior, or physical interrupts:

- input parsing, decoding, and validation of bounded formats (for example the
  fuzzable parser layering the P5 stage assigns to hypercall dispatch);
- configuration and spec normalization (translate an external description
  into an internal one);
- pure state machines and lifecycle transition tables (for example the
  VM/vCPU lifecycle the ADR defines, as a transition function);
- arithmetic and semantic newtype behavior: address/ID/length types, checked
  conversions, overflow and alignment rules (the semantics W15 governs as
  policy);
- codec and wire-format encode/decode round trips for versioned formats;
- capability-slot/handle table logic over explicit inputs (lookup, generation
  matching, rights bit operations — semantics only, not the EL2 enforcement
  path);
- pure policy/scheduling decision functions given an explicit queue state.

**Bare-metal-bound classes** — logic whose correctness depends on executing
at EL2, on hardware timing, or on physical state; host tests may model or
falsify pieces of these, but passing host tests never proves them:

- EL2 entry/exit and exception-vector behavior; stack/BSS establishment;
- MMIO, volatile access, barriers, cache and TLB maintenance;
- Stage-2 page-table effects, VMID/TLB semantics, interrupt delivery;
- assembly boundary sequences and boot-time CPU state.

**Rules the contract must state:**

1. A unit belongs on the host exactly when its specification is expressible
   without the machine. Where a design is unsure, the burden is on the design
   to say which hardware assumption the test would encode.
2. Host tests never require QEMU, a network, a display, or machine-local
   state; a test that needs an emulator is a QEMU-integration concern owned by
   the [W09 entry](../p0-w09-qemu-automation-entry-baseline/README.md) and its
   stage consumers.
3. A host test may encode a *model* of hardware behavior (for example a
   page-table walker's walk algorithm), but its name and report must not
   claim hardware verification.

## 3. Organization rules

Normative rules for where and how host tests live, expressed over the
W03-delivered workspace conventions without inventing crate names:

- Unit tests for a module live with that module, in the workspace member that
  owns it; cross-member behavior is tested at the member's integration-test
  location; host-only shared test utilities live where the W03 contract
  designates for host-side development support. If no location is designated,
  that is a prerequisite gap to record, not a reason to invent a crate.
- A test is named for the contract or behavior it tests, not for the function
  it happens to call.
- Host tests are deterministic: fixed inputs, no real time, no randomness
  without a recorded seed, no cross-test ordering dependence.
- Host tests exercise public (crate-visible) contracts; they do not reach
  into private internals through test-only back doors, and no test-only
  export is added to a crate's public API for their sake.
- No `unsafe` is introduced for testability (unsafe governance is
  [W10](../p0-w10-unsafe-rust-governance/README.md)'s and applies to test
  code equally).
- Tests and their fixtures track the contract change that alters their
  subject, in the same change.

## 4. Execution entry — semantics

The contract fixes these entry properties; together they are the
machine-facing surface W07 and W20 consume:

- **Toolchain:** the pinned repository toolchain (W02), host target as
  delivered by W03. No separate toolchain, channel, or flag set.
- **Scope:** every host-target test in the workspace — unit and integration —
  in one invocation. The entry must not grow per-directory entry points,
  filters-by-default, or a second "fast" variant; selective runs are a local
  developer convenience achieved by standard tool flags, not new entries.
- **Truthfulness:** exit status is success only if every test passed and none
  were skipped silently; skipped tests must be visible in output with a
  recorded reason. Compilation failure of any host-target member is entry
  failure (this is what makes P0-V03 "host build through the documented
  entry" observable).
- **Environment:** no network, no display, no machine-local prerequisites
  beyond the repository-declared toolchain; the entry must behave identically
  for a developer and for CI.
- **Evidence:** the entry's summary output names the number of tests run,
  passed, failed, and skipped, so a result can be quoted in a verification
  record without re-deriving it.

## 5. Execution entry — canonical spelling rule

The contract records one canonical spelling, selected at implementation time
by this rule and stable thereafter: the shortest standard invocation of the
workspace's test mechanism that realizes all §4 semantics over every
host-target member — no broader flags than the semantics require and none
narrower. The recorded spelling, the toolchain it ran under, and the date are
entered in the contract document and the implementation record. Later
workspace changes that would alter the spelling are a recorded minor change
under the mutation thresholds; semantic entry changes are design-level
changes shared with W07's register.

## 6. Proof boundary (mandatory content)

The contract must contain, verbatim in meaning:

- Host-side results prove host-side logic semantics under the stated
  environment only. They do not prove EL2 behavior, MMIO or interrupt
  behavior, cache/TLB effects, timing, or any real-hardware property.
- A passing host suite is never sufficient evidence for a stage exit
  criterion that names QEMU, guest, or hardware evidence.
- QEMU-based verification enters through the [W09 runner
  entry](../p0-w09-qemu-automation-entry-baseline/README.md) and its stage
  consumers; hardware verification belongs to the stages that own hardware
  bring-up.

Every verification record that reports host-test results must restate this
boundary for the results it reports.

## 7. Explicitly excluded interfaces

No crate, module path, function signature, trait, target triple, gate, CI
workflow, script, or hypervisor mechanism is authorized. The placeholder test
asserts only entry health. If implementing the baseline appears to require
creating a workspace member or choosing a target, that is the W03 boundary:
stop and record the conflict.
