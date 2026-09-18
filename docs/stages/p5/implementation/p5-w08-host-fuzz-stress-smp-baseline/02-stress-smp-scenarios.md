# P5-W08 Stress and SMP Scenarios

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W08 detailed design](README.md).

## 1. Scope and authority of these scenarios

Stress and concurrency scenarios exercise the **delivered** W04/W05
lifecycle and authority contracts through their public contracts, in
process (host-side) and, for the SMP rows, on the P3-evidenced two-pCPU
QEMU configuration. They prescribe operations, ordering constraints, and
invariants — never lock types, index structures, or memory ordering
choices, which remain with the owning designs. Where a scenario needs a
deterministic interleaving, it uses the synchronization and observation
points the delivered contracts provide; if none exist, that is a blocked
prerequisite to record, not a reason to add test-only locking to
hypervisor code beyond what the owning design authorizes.

## 2. Lifecycle stress scenarios

All host-side unless stated. Cycle counts are declared per run and recorded;
the counts below are minimums for the evidence to count, chosen to exercise
slot reuse and generation advance without implying a scaling claim.

| ID | Scenario | Procedure | Invariants audited | Pass condition |
|---|---|---|---|---|
| ST-01 | create/lookup/destroy/recreate churn | N ≥ 1000 cycles: create object, resolve reference, destroy, recreate in the expected reused slot | PR-01 after every cycle; capacity/accounting returns to baseline after the run | all audits hold; no oracle hit |
| ST-02 | repeated authority grant/use/revoke | N ≥ 1000 cycles over one object and one caller | PR-02 after every cycle; no residual authority after revoke | all audits hold; no oracle hit |
| ST-03 | quota exhaustion and release | fill capacity to the declared limit, attempt one more (expect resource outcome), release, retry (expect success) | PR-05; no partial allocation observable at any step | determinate outcomes at both limits |
| ST-04 | mixed workload churn | interleaved ST-01/ST-02/ST-03 operations over k ≥ 4 objects and k ≥ 4 callers in a recorded pseudo-random order | full audit after each batch of 100 operations | all audits hold; per-operation outcomes within the delivered vocabulary |
| ST-05 | destroy-under-knowledge | issue lookups and authority checks for a reference concurrently known to a "borrower" test context while the owner destroys and recreates it | borrower never obtains a live authorized outcome on the stale value (INV-P5-03/05) | stale acceptance never observed |

Stress runs report their limits as facts: the cycle counts, host
environment, and any watchpoint statistics. They do not establish maximum
capacity or performance.

## 3. Declared two-pCPU concurrency scenarios

Environment: the P3-evidenced multi-pCPU QEMU configuration, exactly two
pCPUs for P5 evidence, each running the delivered hypervisor stack; the
scenarios drive the W04/W05 contracts from both pCPUs. Each scenario's
observable requirement is **serializability**: the set of observed outcomes
must be consistent with at least one serial order of the issued operations.
Scenario parameters (operation counts, rendezvous points) are declared per
run and recorded.

| ID | Race exposed | Procedure | Required observable | Failure iff |
|---|---|---|---|---|
| SMP-01 | concurrent create/destroy of distinct objects | both pCPUs create, resolve, and destroy disjoint objects in a synchronized loop | all operations complete within the vocabulary; final audit equals initial state | loss, duplicate-live objects, hang, or audit mismatch |
| SMP-02 | revoke racing use of the same object | pCPU 0 repeatedly grants-and-revokes; pCPU 1 repeatedly issues authorized uses | every observed use outcome is consistent with a serial grant/revoke/use order; no unauthorized success after the final revoke | an authorized success attributable to no serial order (lost revocation) |
| SMP-03 | destruction racing stale use | pCPU 0 destroys and recreates an object; pCPU 1 hammers the old reference value | every use of the stale value fails once destruction is observed complete; recreation works | stale acceptance (INV-P5-03 under concurrency) |
| SMP-04 | concurrent authority checks during churn | pCPU 1 resolves and checks references while pCPU 0 churns the table (ST-01 loop) | checks return only contract outcomes for some consistent table state; no hang or unclassified result | unclassified outcome, hang, or corruption |
| SMP-05 | concurrent Guest-data validation | both pCPUs validate overlapping ranges against a shared fake delivered descriptor | outcomes consistent with a serial order of descriptor state; no acceptance of a never-mapped range | accepted-invalid under concurrency |

Watchdog: every scenario runs under a declared wall-clock watchpoint;
expiry is the O2 oracle (hang), recorded as a failure with the last
observed state, not as a slow pass.

## 4. Invariant audit and failure detection

- **Boundary audits:** after each cycle/batch (per scenario), the harness
  audits: object accounting equals the expected live set; no capability
  grants exist beyond the expected set; capacity counters match; no slot is
  both live and free.
- **Post-run audit:** the full audit plus a sweep that every recorded
  reference value is either live-and-expected or invalid.
- **Observation points:** audits use the delivered contracts' observation
  or debug facilities. If a needed observation does not exist, the scenario
  is narrowed and the gap is recorded against the owning package — audit
  code must not add hypervisor state beyond what the owning design
  authorizes (test-only builds may add debug facilities only as the owning
  designs permit; debug features must not change security semantics).
- **Failure classification:** any oracle hit (O1–O6) or audit mismatch is a
  **failed** scenario. Concurrency failures additionally record the
  interleaving evidence available (operation log, rendezvous timestamps)
  for diagnosis. Findings are fixed by the owning packages, not patched in
  the harness.

## 5. Non-success and evidence rules

Every stress/SMP run records: scenario ID, configuration (counts,
rendezvous, pCPU count), seed where pseudo-random ordering is used,
environment, result (passed / failed / blocked / not run), audit outputs,
and any limit observed. `blocked` is the required status when the P3
foundation or a delivered contract cannot support the scenario; it is never
downgraded to `not run` without a recorded reason, and never to `passed`.
