# P5-W07 Scenario Matrix

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W07 detailed design](README.md).

## 1. How to read this matrix

Every scenario has exactly one expected marker, expressed as the outcome
class it must produce. Outcome classes are consumed from the W06 containment
table (`Completed` and the `Denied` classes) — this matrix never invents a
class. "Setup" refers to the deterministic P5 test bootstrap of
[02 §2](02-two-context-isolation-and-harness.md): each context (A, and B in
the isolation scenarios) enters the boot with explicit W05 bootstrap grants
to its own test object(s) and knows no other authority by default.

Column "Proves / does not prove" is the proof boundary for the run record.
All rows are **QEMU-only** evidence in the declared environment; none proves
AArch64 hardware behavior beyond what the architecture reference
independently states, and none proves behavior for hypercalls outside the
delivered W02 supported set.

Scenario ID ranges: A = P5VG-001–003, B = P5VG-010–016, C = P5VG-020–027,
D = P5VG-030–033, E = P5VG-040–042, F = P5VG-050–057, G = P5VG-060–062,
H (isolation) = P5VG-100–103, defined in
[02](02-two-context-isolation-and-harness.md).

## 2. Group A — discovery and the valid path

| ID | Input / precondition | Expected marker class | Dispatch category (W06) | Proves / does not prove |
|---|---|---|---|---|
| P5VG-001 | Discovery/version query per the W02 boundary, issued correctly | `Completed` | `completed` | discovery works and is Guest-observable; not that any other call is supported |
| P5VG-002 | Valid query call (A0 object): owned reference, own authority, well-formed buffer | `Completed` with attributes matching the W04-described class/generation | `completed` | the full S1–S8 chain succeeds once, end to end (W06-DV03's Guest side); not general VM management |
| P5VG-003 | Repeat of P5VG-002 in the same boot with unchanged state | `Completed` again, identical attribute fields | `completed` | repeatable read-only behavior (no hidden one-shot state); not concurrent correctness (W08) |

## 3. Group B — malformed and ABI-negative

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-010 | Call number outside the delivered supported set | `UnknownCall` | `denied-structural` | unknown calls are contained; not that the number space is closed forever |
| P5VG-011 | Defined call requesting a feature the boundary marks unsupported | `UnsupportedFeature` | `denied-structural` | feature gating is observable; not future feature policy |
| P5VG-012 | Version field incompatible with the boundary's declared compatibility | `VersionMismatch` | `denied-structural` | version refusal works; not a compatibility promise |
| P5VG-013 | Reserved/flag field carries a non-zero value where the boundary requires zero | `Malformed` | `denied-structural` | reserved-field policy is enforced (W02 contract); not the specific wire encoding |
| P5VG-014 | Argument length field structurally invalid for the call | `Malformed` | `denied-structural` | structural length checks precede object access; not buffer-content validation (group F) |
| P5VG-015 | Request fails the boundary's structural format check (e.g., inconsistent field combination) | `Malformed` | `denied-structural` | malformed containment (INV-P5-07); nothing else |
| P5VG-016 | Call issued with valid structure but referencing the query operation on the discovery-only path (operation not applicable to the boundary object class) | `UnsupportedFeature` or `WrongType` — exactly the class W06's table maps for this shape | per mapping | the mapping, not Guest guesswork, decides the class; not new semantics |

## 4. Group C — object-reference negatives (P5VG-020–027)

All use the valid query call; only the reference value varies. "Test
constants" (zero, max, forged random) are build-time constants of the asset.

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-020 | Handle value zero | `InvalidReference` | `denied-reference` | zero never resolves (INV-P5-03); not the internal slot encoding |
| P5VG-021 | Handle value all-ones (max) | `InvalidReference` | `denied-reference` | max never resolves; not the encoding |
| P5VG-022 | Forged random values (declared fixed set, e.g., 64 pseudo-random constants) | `InvalidReference` for every value | `denied-reference` | random forging cannot hit a live reference the caller may use; not secrecy of values (isolation is authority-based, see group H) |
| P5VG-023 | Handle of A0 with corrupted generation field (test constant derived from the bootstrap value) | `InvalidReference` | `denied-reference` | generation protection (INV-P5-03); not the generation representation |
| P5VG-024 | Handle to an object destroyed earlier in the same boot (uses P5VG-061's destroyed object) | `InvalidReference` | `denied-reference` | destroyed references stay invalid; not object reuse timing |
| P5VG-025 | Handle to a slot reused after destroy (test bootstrap creates A1 in a recycled slot per the W04 contract) | `InvalidReference` for the stale value; the new object's own handle resolves | `denied-reference` / `completed` | stale-reference protection after reuse (INV-P5-03); not allocator internals |
| P5VG-026 | Repeated destroy of the same object (destroy A0-valid, then destroy A0 again) | first destroy: per its contract (`Completed` or its defined class); second destroy: `InvalidReference` | per W04 contract | repeated destruction is contained (P5-V11 row); not a general object system |
| P5VG-027 | Existing handle of a different object class supplied where the query operation requires the queryable class | `WrongType` | `denied-reference` | type safety (P5-V05, INV-P5-03); not cross-class conversion rules |

## 5. Group D — authority negatives

All use structurally valid calls on live references known to context A.

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-030 | Valid reference to an object for which the caller holds no capability (test bootstrap creates A2 but grants A no authority over it) | `NoAuthority` | `denied-authority` | reference validity is not authority (INV-P5-04); not capability representation |
| P5VG-031 | Capability with an insufficient right class for the query's required right (bootstrap grants observe-only where the operation requires more, per W05's class model) | `NoAuthority` | `denied-authority` | operation-level rights enforcement (INV-P5-04); not the rights bitset encoding |
| P5VG-032 | Query against the caller's own object while a second capability of the caller for the same object was revoked (uses group E state) — the unrevoked capability's use must still succeed | `Completed` | `completed` | revocation is capability-scoped, not object-scoped destruction (W05/§95 distinction); not delegation semantics |
| P5VG-033 | Any attempt to have authority inferred from identity: the Guest supplies its own VM/security-context identifier as if it conferred authority (field misuse; no such parameter exists in the ABI — the attempt is a reserved-field/format violation) | `Malformed` | `denied-structural` | no Guest-writable field can manufacture authority (INV-P5-10); the Guest-side complement of W06's authorization review |

## 6. Group E — revocation lifecycle

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-040 | Baseline: A0 valid use before revocation | `Completed` | `completed` | grant-then-use works (P5-V08 sequence start); not delegation |
| P5VG-041 | After the test bootstrap revokes A's capability for A0 (W05 revoke contract): same query as P5VG-040 | `Revoked` | `denied-authority` | formerly valid authority is rejected after revoke (INV-P5-06); not a revocation tree |
| P5VG-042 | Re-grant of equivalent authority to A after P5VG-041 (test bootstrap re-grants), then the same query | `Completed` | `completed` | revoke/re-grant lifecycle is deterministic; nothing else |

## 7. Group F — Guest-data negatives (valid reference and authority; only the buffer varies)

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-050 | Zero-length buffer | per W03 contract (accepted-with-nothing-written or controlled denial — the W03 design decides; the matrix records the delivered rule) | per mapping | zero-length rule is the delivered contract, exercised; not W03 internals |
| P5VG-051 | Maximum declared length (boundary value) | `Completed` with exactly the declared payload | `completed` | max-length rule honored; not a general size policy |
| P5VG-052 | Address + length that overflow the address arithmetic | `InvalidAddress` | `denied-address` | overflow cannot wrap into a valid range (INV-P5-02); not overflow of Host-internal types |
| P5VG-053 | Buffer entirely outside any mapped Guest region | `InvalidAddress` | `denied-address` | unmapped ranges are rejected; not Stage-2 fault internals (P4 evidence) |
| P5VG-054 | Buffer crossing a mapping boundary (partially mapped: first page mapped, second not) | `InvalidAddress` (or `GuestDataFault` if the delivered W03 contract fault-stops — whichever W03 declares; fixed before runs) | `denied-address` | partial mappings are controlled (P5-V03); the class is W03's, not invented here |
| P5VG-055 | Write into a read-only Guest buffer (direction violation) | `InvalidAddress` | `denied-address` | permission/direction enforcement (INV-P5-02); not Stage-2 permission encoding |
| P5VG-056 | Buffer of a memory type the W03 boundary forbids for hypercall buffers | `InvalidAddress` | `denied-address` | type rule enforced; not device memory semantics |
| P5VG-057 | Buffer spanning a boundary W03 declares forbidden (e.g., straddling the forbidden edge case fixed by W03) | per delivered W03 rule | `denied-address` | the forbidden-boundary rule is exercised; not the rule's rationale |

Note: rows P5VG-050, 054, 057 defer to the delivered W03 contract where the
plan leaves the rule to W03's design. Before implementation, the delivered
W03 design's rule is transcribed into these rows; if W03's design leaves
them open, that transcription is a Step-2 action in
[03](03-implementation-workflow-and-validation.md), not a run-time choice.

## 8. Group G — lifecycle and state negatives

| ID | Input / precondition | Expected marker class | Dispatch category | Proves / does not prove |
|---|---|---|---|---|
| P5VG-060 | Query against an object whose lifecycle state, per the W04 contract, does not permit the query (state the bootstrap places the object in deliberately) | `BadState` | `denied-state` | operation-state checks are enforced (P5-V06); not a full VM lifecycle |
| P5VG-061 | Destroy of A0 (authorized destroy-class call per the delivered supported set), observed as P5VG-024's setup | destroy's contract outcome; subsequent query: `InvalidReference` | per mapping | destroy invalidates references; not general resource accounting (W08) |
| P5VG-062 | Immediate repeat of the whole Group A valid path after P5VG-061 destroyed and re-created the object (fresh handle from the bootstrap) | `Completed` for the new handle | `completed` | create/destroy/recreate leaves the service path consistent; not concurrency correctness (W08) |

## 9. Group H — two-context isolation

Defined and evidenced in
[02 §2](02-two-context-isolation-and-harness.md) (P5VG-100–103): owning use
succeeds, cross-context use of the same raw value is denied, the negative
control succeeds in-boot, and the isolation result is reproducible across
declared repeat boots. These rows satisfy P5-V12; no single-context scenario
in this file may be reported as isolation evidence.

## 10. Coverage accounting

The matrix covers every scenario family the plan requires: discovery
(A); valid HVC with valid reference/authority/data (A); unknown, invalid
version, flags, length, address, overflow, and type cases (B, F); zero/max/
random/stale/destroyed generation cases (C); insufficient/no/revoked/cross-VM
authority (D, E, H); repeated destruction (C); wrong lifecycle state (G).
Gaps found during implementation (a required family without a row, or a row
whose dependent mechanism was delivered differently) are added or amended in
this file with a recorded rationale — never silently at run time.
