# Zelyr Host-Test Baseline

**Status:** Normative host-side testing policy.  
**Scope:** The host-testable logic boundary, host-test organization rules,
the repository's single host-test execution entry, the coverage-category
matrix, and the proof boundary for host results. It does not define quality
gates (P0-W07), CI (P0-W20), QEMU-based verification (P0-W09), or the
workspace itself (see the [build-target baseline](../development/build-target-baseline.md)).  
**Version:** v0.1  
**Owner/change context:** P0-W08 host-side testing baseline; operationalizes
ADR-049's host-side validation layer.  
**Supersedes:** The absence of host-test policy (`docs/testing/` was an
unlinked placeholder).

## 1. Testable-logic boundary

**Host-verifiable classes** — logic whose semantics are fully determined by
inputs and pure state, with no reliance on EL2 execution, device timing,
cache or TLB behavior, or physical interrupts:

- input parsing, decoding, and validation of bounded formats;
- configuration and spec normalization;
- pure state machines and lifecycle transition tables;
- arithmetic and semantic newtype behavior: address/ID/length types, checked
  conversions, overflow and alignment rules;
- codec and wire-format encode/decode round trips for versioned formats;
- capability-slot/handle table logic over explicit inputs (semantics only,
  not the EL2 enforcement path);
- pure policy/scheduling decision functions given an explicit queue state.

**Bare-metal-bound classes** — logic whose correctness depends on executing
at EL2, on hardware timing, or on physical state; host tests may model or
falsify pieces of these, but passing host tests never proves them:

- EL2 entry/exit and exception-vector behavior; stack/BSS establishment;
- MMIO, volatile access, barriers, cache and TLB maintenance;
- Stage-2 page-table effects, VMID/TLB semantics, interrupt delivery;
- assembly boundary sequences and boot-time CPU state.

Rules:

1. A unit belongs on the host exactly when its specification is expressible
   without the machine. Where a design is unsure, the burden is on the design
   to say which hardware assumption the test would encode.
2. Host tests never require QEMU, a network, a display, or machine-local
   state; a test that needs an emulator is a QEMU-integration concern owned
   by the P0-W09 runner entry and its stage consumers.
3. A host test may encode a *model* of hardware behavior, but its name and
   report must not claim hardware verification.

## 2. Organization rules

- Unit tests for a module live with that module, in the workspace member that
  owns it; cross-member behavior is tested at the member's integration-test
  location; host-only shared test utilities live where the build-target
  policy designates for host-side development support. If no location is
  designated, that is a prerequisite gap to record, not a reason to invent a
  crate.
- A test is named for the contract or behavior it tests, not for the function
  it happens to call.
- Host tests are deterministic: fixed inputs, no real time, no randomness
  without a recorded seed, no cross-test ordering dependence.
- Host tests exercise public (crate-visible) contracts; they do not reach
  into private internals through test-only back doors, and no test-only
  export is added to a crate's public API for their sake.
- No `unsafe` is introduced for testability (unsafe governance applies to
  test code equally).
- Tests and their fixtures track the contract change that alters their
  subject, in the same change.

## 3. Execution entry — semantics

- **Toolchain:** the pinned repository toolchain (see the [toolchain
  baseline](../development/toolchain-baseline.md)), host target as delivered
  by the build-target policy. No separate toolchain, channel, or flag set.
- **Scope:** every host-target test in the workspace — unit and integration —
  in one invocation. The bare-metal hypervisor member is not a host-target
  member and is excluded by name; future host-class members are included
  automatically. The entry must not grow per-directory entry points,
  filters-by-default, or a second "fast" variant; selective runs are a local
  developer convenience achieved by standard tool flags, not new entries.
- **Truthfulness:** exit status is success only if every test passed and none
  were skipped silently; skipped tests must be visible in output with a
  recorded reason. Compilation failure of any host-target member is entry
  failure.
- **Environment:** no network, no display, no machine-local prerequisites
  beyond the repository-declared toolchain; the entry must behave identically
  for a developer and for CI.
- **Evidence:** the entry's summary output names the number of tests run,
  passed, failed, and skipped, so a result can be quoted in a verification
  record without re-deriving it.

## 4. Execution entry — canonical spelling

Recorded invocation (selected 2026-09-18 under the pinned toolchain
`1.98.1`):

```sh
cargo test --workspace --exclude hypervisor
```

The `--exclude hypervisor` term is semantically required, not a convenience
filter: the hypervisor member is a bare-metal-class freestanding binary whose
sources cannot compile for a host target (its assembly and panic-handler
contract are AArch64-specific), so a whole-workspace test invocation fails on
it by construction. The exclusion names exactly the non-host class. Later
workspace changes that would alter the spelling are a recorded minor change
under §6; semantic entry changes are design-level changes shared with the
P0-W07 gate register.

## 5. Coverage categories

Future host tests must address these required categories; the matrix maps
them to host-verifiable unit classes. "When fallible"/"when stateful" means
the category applies when the unit's contract includes that property.

| Unit class (host-verifiable) | Normal | Boundary | Invalid input | Resource failure | Repeated lifecycle |
|---|---|---|---|---|---|
| Parser/decoder/validator | required | required | required | when fallible | when stateful |
| Config/spec normalization | required | required | required | when fallible | when stateful |
| Pure state machine | required | required | required | when fallible | required |
| Arithmetic/newtype semantics | required | required | required | n/a unless fallible | n/a unless stateful |
| Codec round trip | required | required | required | when fallible | when stateful |
| Handle/table logic | required | required | required | when fallible | required |
| Pure policy/decision function | required | required | required | when fallible | when stateful |

Category definitions: **normal behavior** (valid in-range inputs produce the
specified output and state); **boundary values** (edges of the valid domain:
empty/zero, single element, maxima, exact alignment and off-by-one, first/last
transitions); **invalid input** (out-of-domain inputs are rejected or
recovered per contract without panic, with classified rejection — mandatory
for any unit consuming externally influenced input, per the ADR's
untrusted-input principle); **resource failure** (exhaustion, unavailability,
or mid-operation failure: allocation failure, capacity reached, dependency
errors, rollback — at P0 as logic-level error-path simulation); **repeated
lifecycle** (create/use/destroy repeated, destroy-then-recreate,
use-after-close rejected, generation/staleness handling).

**Concurrency** is the named conditional extension: once a unit's contract
specifies concurrent access, its host tests must cover the interleaving
aspects its design calls out, to the extent expressible without the machine;
full SMP behavior belongs to the stages that own SMP. A design claiming "not
concurrent" for shared mutable state must record that claim.

**Mapping rule:** every future module design that introduces host-testable
logic must contain a test-mapping statement — per unit, which categories
apply, which are excluded, and one-line reasons for each exclusion. An
unmapped unit or an unmotivated exclusion is a design review finding; a host
test that silently covers a different category than claimed is a test review
finding.

**Reserved extensions (defined by their future owners):** fuzz/property
harnesses (ADR-049 layer; invalid-input coverage above is the unit-level
floor); concurrency/stress suites (SMP stage); QEMU integration, guest
self-test, Linux regression, hardware (the P0-W09 runner entry and owning
stages) — never reported as host-test results.

## 6. Change thresholds

- **Recorded minor change** (ordinary PR review, recorded in the
  implementation record): recording or updating the entry spelling;
  relocating the placeholder baseline; content additions within a category.
- **Design-level change** (a reviewed design change before the change):
  adding a coverage class; changing entry semantics (toolchain source, scope
  rule, truthfulness rule); moving the proof boundary. Semantic entry changes
  are shared with the P0-W07 gate register — a silent edit is a review
  failure.

## 7. Proof boundary

**Host-side results prove host-side logic semantics under the stated
environment only. They do not prove EL2 behavior, MMIO or interrupt behavior,
cache/TLB effects, timing, or any real-hardware property. A passing host
suite is never sufficient evidence for a stage exit criterion that names
QEMU, guest, or hardware evidence.** QEMU-based verification enters through
the P0-W09 runner entry and its stage consumers; hardware verification
belongs to the stages that own hardware bring-up.

Every verification record that reports host-test results must restate this
boundary for the results it reports.

## 8. Placeholder baseline status

P1-W03 adds source-shared production capability decoder/policy tests under
`crates/host-test-baseline/tests/p1_capabilities.rs`; their unit cases remain
in the owning pure source module. This is a recorded minor content addition,
not a workspace or canonical entry change. Privileged reads and publication
are excluded from host execution. The entry-health test remains infrastructure
evidence and is counted separately from product tests.

The `crates/host-test-baseline` member is the P0-W08 placeholder: its single
test proves the entry executes and reports truthfully. It is infrastructure
verification, never product coverage, and is expected to be superseded by the
first real host-tested logic; retirement is a recorded minor change. A future
design that treats a passing placeholder as product coverage is a review
failure.
