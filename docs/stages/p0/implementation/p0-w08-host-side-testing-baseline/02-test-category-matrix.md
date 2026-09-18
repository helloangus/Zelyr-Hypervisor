# P0-W08 Test Category Matrix

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W08 detailed design](README.md).

This file is the source of the coverage-category section of the host-test
baseline contract. It prescribes which categories future host tests must
address; it is not a test plan and defines no test. Future module designs map
their host-testable units to these categories via the rule in §8.

## 2. Required categories

The five categories below are the plan's named classes. For each: definition,
typical application, and what a gap means.

### 2.1 Normal behavior

- **Definition:** the intended operation on valid, in-range inputs produces
  the specified output and state.
- **Application:** every host-testable unit's primary path; the baseline any
  further categories refine.
- **Gap meaning:** the unit's contract is untested at its most common use;
  review finding.

### 2.2 Boundary values

- **Definition:** inputs and states at the edges of the valid domain:
  empty/zero, single element, maximum representable values, exact alignment
  and alignment-off-by-one, boundary of a valid range, first/last transition
  of a state machine.
- **Application:** numeric and address-like semantics, length/index
  arithmetic, capacity logic, state machines (first and final transitions).
- **Gap meaning:** off-by-one and overflow defects survive; review finding
  for any unit whose contract mentions a limit.

### 2.3 Invalid input

- **Definition:** inputs outside the valid domain: malformed structure,
  out-of-range values, wrong versions, conflicting fields, adversarial
  combinations. The unit must reject or recover per its contract without
  panic, and must classify the rejection.
- **Application:** every unit that consumes externally influenced input —
  parsers, codecs, normalization layers, handle/table lookups. The ADR's
  untrusted-input principle (Guest/device/management input is untrusted)
  makes this category mandatory for any such unit.
- **Gap meaning:** the untrusted-input boundary is unverified; review finding
  that blocks the subject design's acceptance.

### 2.4 Resource failure

- **Definition:** behavior when an underlying resource is exhausted,
  unavailable, or fails mid-operation: allocation failure, capacity reached,
  dependency returning an error, partial completion followed by rollback.
- **Application:** units that allocate, buffer, or call fallible subservices.
  At P0 this applies as logic-level simulation (error-return paths); real
  resource behavior belongs to later stages.
- **Gap meaning:** error paths exist but are never exercised; review finding
  for any unit with fallible operations.

### 2.5 Repeated lifecycle

- **Definition:** create/use/destroy repeated many times, including
  destroy-then-recreate, use-after-logical-close rejected, and idempotent
  repeated operations; generation/staleness handling where the unit models
  it.
- **Application:** table/handle logic, cache-like structures, state machines
  re-entered after terminal states.
- **Gap meaning:** leak and stale-state defects survive; review finding for
  any unit with a lifecycle.

## 3. Conditional extension: concurrency

Concurrency is not a standalone required category at P0 (no concurrent
hypervisor logic exists), but it is a named extension: once a unit's contract
specifies concurrent access, its host tests must cover the interleaving
aspects its design calls out (atomicity of updates, no lost wakeups in a
modeled queue, generation checks under reuse) to the extent expressible
without the machine; full SMP behavior belongs to the stages that own SMP
(P3 per its plan index). A design claiming "not concurrent" for shared
mutable state must record that claim.

## 4. Category-to-boundary applicability

The matrix crosses the [testable-logic boundary](01-host-test-contract.md) §2
with the categories. It is contract content:

| Unit class (host-verifiable) | Normal | Boundary | Invalid input | Resource failure | Repeated lifecycle |
|---|---|---|---|---|---|
| Parser/decoder/validator | required | required | required | when fallible | when stateful |
| Config/spec normalization | required | required | required | when fallible | when stateful |
| Pure state machine | required | required | required | when fallible | required |
| Arithmetic/newtype semantics | required | required | required | n/a unless fallible | n/a unless stateful |
| Codec round trip | required | required | required | when fallible | when stateful |
| Handle/table logic | required | required | required | when fallible | required |
| Pure policy/decision function | required | required | required | when fallible | when stateful |

Bare-metal-bound classes have no host-category requirements; their evidence
belongs to QEMU/guest/hardware layers.

## 5. Mapping rule for future designs

Every future module design that introduces host-testable logic must contain a
test-mapping statement: per unit, which categories apply (per §4), which are
excluded, and one-line reasons for each exclusion. Reviewers and the
[quality gates](../p0-w07-development-quality-gates/README.md) use the
statement as the acceptance baseline for that unit's host tests; an
unmapped unit or an unmotivated exclusion is a design review finding, and a
host test that silently covers a different category than claimed is a test
review finding.

## 6. Reserved extensions (not defined here)

- **Fuzz/property testing:** ADR-049 lists it as a validation layer; its
  harnesses, targets, and gates are owned by the later designs that introduce
  fuzzable subjects (for example the P5 dispatch/parser layering). This
  matrix fixes only that invalid-input coverage is the unit-level floor.
- **Concurrency/stress suites:** owned by the SMP stage's plans.
- **QEMU integration, guest self-test, Linux regression, hardware:** owned by
  the [W09 runner entry](../p0-w09-qemu-automation-entry-baseline/README.md)
  and the owning stages; never reported as host-test results.

## 7. Explicitly excluded interfaces

The matrix authorizes no code, test framework choice, harness, macro, or
dependency. Category membership is documentary data used in design review and
the contract.
