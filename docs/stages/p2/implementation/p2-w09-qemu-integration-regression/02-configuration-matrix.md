# P2-W09 Configuration Matrix and Expected Observations

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W09 detailed design](README.md).

## 1. Configuration identity

A configuration is the tuple `(machine model, CPU count, RAM
specification, P1 boot recipe)` with a stable label `virt-cpu<N>-mem<K>`.
Machine model and boot recipe follow the P1 reference boot contract
(assumed contract A2); this design invents neither.

## 2. Pinning procedure (implementation-time, before expectations)

For each RAM variant, dump the DTB QEMU supplies (documented QEMU
mechanism, e.g. `-machine dumpdtb=`, recorded with the exact QEMU version)
and derive from the *actual blob*: memory-bank count, bank base/length
list, CPU-node count, GIC/PSCI/console classes. These observations fill the
configuration's expectation file ([README](README.md) Decision 3). The
multi-bank cell (2 GiB) is kept only if the dump shows ≥ 2 memory nodes;
if the observed QEMU version yields one bank, the expectation records one
bank and the multi-bank exercise falls back to the host matrices (W08
S306) with the gap recorded for W10 — the matrix must reflect the platform
as it is, not as assumed.

## 3. Required configurations

| Config label | CPU count | RAM | Required because | Boot repetitions |
|---|---|---|---|---|
| `virt-cpu1-memB` | 1 | baseline (P1 recipe) | Degenerate single-CPU reference; P2-K01 chain | 5 |
| `virt-cpu2-memB` | 2 | baseline | Minimum multi-CPU description; P2-K02 | 2 |
| `virt-cpu4-memB` | 4 | baseline | Matches RK3566 topology class and P3's early target; P2-K02 | 2 |
| `virt-cpu1-mem512` | 1 | 512 MiB | RAM-size variation within one bank; P2-K03 | 2 |
| `virt-cpu1-mem2g-hs` | 1 | 2 GiB, high-memory configuration | Multi-bank map exercise (pinned per §2); P2-K03 | 2 |

8-CPU is deliberately excluded: it is P3's stress count (p3-w13) and adds
no P2 fact (plan scope: CPU-count variants for P2 facts). Adding it later
is a recorded revision.

## 4. Per-configuration expected observations

Expectation file per configuration, schema mirroring W07's
([expectation pattern](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)
§3): rows carry `binding` flags and payload marks; classes:

| Observation group | Expected class (binding) | Payload marks (pinned from dump/boot) |
|---|---|---|
| Intake marker | PASS — "DTB intake validated", no diagnostics | blob size, version |
| CPU inventory | PASS | usable CPU count == configuration CPU count; boot-CPU matched |
| GIC / timer / PSCI / console | PASS (GICv3, PSCI Usable{method}, console `stdout-path` present) | method, path-presence |
| RAM banks | PASS | bank count and per-bank frame totals == §2 dump |
| Reservations | PASS/WARN per dump | `/reserved-memory` + rsvmap counts |
| Map summary | PASS — equation `ram = allocatable + protected` exact | `ram_frames` == `-m` in 4 KiB frames (per bank sums); protected total inside domain (§5) |
| Page allocator | PASS — ready marker; conservation holds | per-region managed/free totals; metadata frames == sealed plan |
| Heap | PASS — init; budget respected | pages_used; per-class zero-used at boot |
| Inspection consistency | PASS — no `ConsistencyBroken` | render present with all sections |

Values are pinned from the first executed boot of each configuration (with
the §2 dump as corroboration), then binding. The baseline configuration's
values additionally cross-check against the P1 recipe's documented
parameters.

## 5. Accounting domain (P2-K04)

- **Exact part:** `ram_frames == allocatable_frames + protected_frames`;
  allocator `managed == allocatable − metadata`; heap
  `pages_used ≤ allocator used`. These hold per W03/W04/W05 contracts;
  violation is a failed row, never a domain adjustment.
- **Bounded part:** `protected_total ∈ [min, max]` per configuration,
  where `min` is pinned from the first boot's observed total and `max` is
  `min + slack`, with slack covering P1 image-size variance and DTB-size
  variance across QEMU versions. Slack is declared in the expectation
  file with its rationale (default: a small multiple of the observed
  reservation magnitude, e.g. `min + 4 MiB`). A total outside the domain
  is a failed row and a finding (unexpected reservation or metadata
  growth), not noise.
- **Not-bounded part:** allocatable *addresses* are not pinned (except
  bank membership per the dump); pinning addresses would over-constrain
  harmless allocator evolution.

## 6. Variant interaction rules

The matrix crosses CPU count with baseline RAM only, and RAM size with 1
CPU only — a deliberate 5-cell design, not a full cross product: P2's
facts vary independently along the two axes, and P2-V11 requires variation,
not exhaustion. A full cross product is Reserved for P3/P4 matrices that
need it.
