# P4-W02 Architecture, Objects, and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W02 detailed design](README.md).

## 1. Logical modules

Placement rule: Stage-2 is an AArch64 architecture mechanism (ADR-041), so the
first three modules live in the Arch layer; the vocabulary module is the
Core-visible surface. Crate/file placement follows the P0 workspace design
(assumed contract M8 in [01 §2](01-scope-and-foundations.md)); this design
fixes logical modules and their boundaries, not file paths.

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `s2-table` | Descriptor encoding/decoding, root/table frame management, walk-for-mutation and walk-for-read over P4's descriptor representation | none persistent; operates on frames owned by `s2-space` | IPA ranges, flags, frame references | descriptor mutations, walk results | allocation policy, TLB invalidation, Guest entry |
| `s2-space` | Address-space object: lifecycle, mapping ledger, VMID, lock, public map/unmap/protect/query/activate/invalidate operations | root frame, table frames, ledger, VMID, state | `MappingGrant`s, IPA ranges, flags | success/error, query snapshots, activation effects | Guest RAM contents, image loading, fault diagnosis |
| `s2-vmid` | Minimal VMID allocator (distinct VMIDs, no recycling) | free/reserved VMID set | allocation/release requests | VMID values or exhaustion error | VMID programming into sysregs (done by `s2-space` activate) |
| `s2-tlb` | Invalidation backend + seam: encode and apply invalidation operations for the current path; barriers | none | operations from `s2-space` | applied invalidations, ordering guarantees | shootdown policy, transport internals (P3 seam) |
| `s2-vocab` (Core-visible) | Stage-2-neutral vocabulary: mapping flags value object, memory-type value object, error type, query result type | none | — | types consumed by W03/W04/W06 | descriptor bits, VMIDs, sysregs (never appear here) |

Layering check: `s2-vocab` depends only on Core-level address newtypes (M1)
and error conventions; `s2-table`/`s2-space`/`s2-tlb`/`s2-vmid` are
Arch-internal. No board, SoC, or QEMU name appears in any module; QEMU
behavior differences are recorded as Specification Investigation items, never
as branches.

## 2. Core objects and ownership

### 2.1 `GuestAddressSpace` (one per Guest address space; P4 expects exactly one live instance at a time, enforced by scope, not by a global)

- **Owned state:** root table frame; table frames; VMID (from `s2-vmid`);
  mapping ledger (sorted map of mapped IPA page ranges → frame base, flags,
  memory type); lifecycle state; the space lock (D5).
- **Immutable after create:** root frame, VMID, IPA span bounds.
- **Not owned:** mapped Guest frames' data content (owned by the Guest RAM
  object from P4-W03), the allocator's global accounting, any pCPU.
- **Destruction:** legal only from the quiescent state; unmaps all ranges,
  invalidates (all-current), releases table frames and the VMID, then returns
  everything to the allocator. Guest frames themselves are returned by W03's
  Guest RAM teardown, which must be sequenced after address-space destroy
  (see [03 §3.8](03-code-contracts-stage2-core.md)).

### 2.2 `MappingGrant` (stage-local transfer value, decision D8)

A validated (frame base, page count, flags) tuple produced by P4-W03's Guest
RAM construction. `map` consumes it and the ledger records the mapping. It is
not the ADR `MemoryObject`; it carries no sharing, pinning, or DMA state.

### 2.3 `Vmid` (newtype)

Arch-internal numeric identity for TLB tagging. Never exposed through
`s2-vocab`. Uniqueness invariant: at most one live address space holds a given
VMID (no recycling makes this trivially true in P4).

### 2.4 Per-pCPU activation register (conceptual)

P4 has no persistent "current space" object; activation is an operation on the
pCPU's `VTTBR_EL2`/`VTCR_EL2` state. The pCPU's current-space identity is
tracked by P4-W04's per-pCPU world-switch state (which owns the Guest run
context); `s2-space` records only whether the space believes it is active and
on which pCPU identity, to make deactivate-on-destroy verifiable. This avoids
a second owner for "current vCPU" (Plan Agent guardrail: registries are not
implicit owners).

## 3. Address-space lifecycle state machine

```text
             create()                 activate()                (Guest runs; W04 owns run state)
  [absent] -----------> Constructed ---------> Active ------------> (concurrent path)
                          |  ^                   |  ^
                    map/unmap/protect         mutations with BBM
                          |  |                   |  |
                          v  |                   v  |
                       Constructed (quiescent mutation, no TLBI needed
                                    for non-current spaces)
 Active ---deactivate/destroy--> Destroying --> Destroyed (absent)
```

Rules:

- `Constructed` (quiescent): all mutations are ledger+descriptor updates; no
  TLB operation is required because no CPU translates through it.
- `Active`: the space is installed on one pCPU (P4: the Guest pCPU). Mutations
  remain legal (D6) and must follow BBM + invalidation
  ([§6](#6-stage-2-ordering-barrier-and-invalidation-semantics)).
- `Destroying`: transient; the lock is held, no new mutations are accepted;
  unmap-all, invalidate-all-current, page release, VMID release.
- Illegal transitions (destroy while active without deactivation is *handled*
  by performing deactivation inside destroy; use of a destroyed space is an
  internal-invariant violation → fatal per D10) are enumerated in the
  contracts.
- The ADR vCPU/VM lifecycle machines are **not** implemented here; W04 owns
  the P4 vCPU run states and P7+ owns the full lifecycle. This machine is
  strictly the translation-context lifecycle.

## 4. Activation and register programming

Activation writes, in order: `VTCR_EL2` (Stage-2 translation control: T0SZ,
SL0, granule, shareability/normal-memory attributes for the P4 temporary
layout), `VTTBR_EL2` (VMID + root table HPA), then a context-synchronization
`ISB`. The values are computed from the space's immutable create parameters;
they are not runtime-tunable policy. Deactivation on destroy restores a
defined disabled context (Stage-2 disabled for the current path) before
releasing frames, so no translation can reference freed tables.

Register-programming exactness: the field layouts follow the pinned Arm
architecture reference; reserved-bit preservation (read-modify-write with
reserved bits kept) is mandatory (Coding Guidelines). QEMU acceptance of a
particular encoding never substitutes for the architectural check (W01 A7).

## 5. Permission and memory-type model (P4 subset)

Stage-2 permission vocabulary exposed via `s2-vocab`:

- `S2Access`: combinations of Read / Write (Stage-2 has no user/kernel split;
  execute is separate).
- `S2Execute`: Executable / ExecuteNever (XN).
- `S2MemType`: NormalCacheable / NormalNonCacheable / Device (P4 uses
  NormalCacheable for Guest RAM, Device for the console page mapped by W03).

Mapping from this vocabulary to descriptor S2AP/XN/MemAttr fields happens
only in `s2-table`. Core code (W03 loader, W04 run loop, W06 diagnostics)
never sees S2AP/XN bit values. Permission faults produced by the hardware are
*observed* as Stage-2 fault information (W06/W04); they are never simulated by
this module.

## 6. Stage-2 ordering, barrier, and invalidation semantics

### 6.1 Break-before-make (BBM)

Every transition of an existing IPA page between mappings (unmap, protect
change, remap) performs: (1) clear/replace the descriptor to an invalid entry,
(2) ensure visibility to the page-table walker (`DSB ISHST`), (3) invalidate
the affected translation for the current path, (4) write the new valid
descriptor. For a protect change on a mapped page, P4 uses the two-step
(invalidate old, then set new) sequence rather than an in-place valid-entry
flag update, so no intermediate state can produce a stale permission
translation (P4-V08's "does not rely on stale translations" condition).

### 6.2 First-time map

A first map of an IPA that was never valid does not require a pre-invalidate
for the current path (there is no stale entry to remove), but P4 performs the
uniform sequence anyway when the space is active, because "never valid" is a
ledger property, not a hardware guarantee, under future recycling. Uniformity
is reviewable; per-case optimization is out of scope.

### 6.3 Invalidation operations and seam

Operations (decision D7): `InvalidateAll` (current VMID, all IPsas) and
`InvalidateRange(ipa_range)` (current VMID). P4 implements both against the
current pCPU only; the backend is a small trait/object seam so the P3
transport can later provide masked-broadcast without changing `s2-space` call
sites. After any invalidation for entry-affecting mutations, ordering is:
`DSB` (completion of invalidation) then `ISB` (context sync) before Guest
(re-)entry; the world-switch (W04) performs its own `ISB` after context
installation, and both are retained (redundant barriers are cheap; missing
ones are unsound).

### 6.4 Cross-pCPU reservation

P4 records as a factual limitation: invalidation covers the current pCPU
only. The single-Guest/single-vCPU P4 run model (W04) means the Guest path
translates on one pCPU; if a second translating context ever existed, the
seam must be extended **before** that context runs, not after. This statement
is the W02 contribution to "must not preclude multi-pCPU handling."

## 7. Concurrency model

- One address-space lock per space (D5). Lock hold times are bounded: no
  allocation of unbounded size, no invalidation broadcast (none exists in P4),
  no Guest execution under lock.
- Table frames are exclusively owned by the space once allocated; no other
  module writes descriptor memory. All descriptor memory writes happen in
  `s2-table` under the space lock (enforced by API shape: the writer takes
  locked access tokens, not raw pointers, outside `unsafe`).
- The ledger is the authoritative mapping state; it is mutated under the same
  lock as the descriptors, so query snapshots (D9) are consistent with
  hardware-visible state as of lock release plus completed invalidation.
- pCPU activation: P4 has one Guest execution path; concurrent `activate` from
  a second pCPU is rejected as a programming error (fatal invariant, D10),
  because P4 defines no legitimate second caller. The rejection is explicit,
  not an accidental data race.
- Interrupt context: `s2-space` operations are never called from interrupt
  context in P4 (EL2 IRQ handling is minimal and does not touch Stage-2);
  the lock is a plain non-recursive spin lock per P3-W06 semantics (assumed
  M6-era primitives) with the IRQ rules P3 documents.

## 8. Telemetry points (W02-scope)

Per event-namespace routing (W01 A8), `s2-space` emits: `s2.space.create`,
`s2.space.destroy`, `s2.map`, `s2.unmap`, `s2.protect`, `s2.activate`,
`s2.invalidate` (with operation kind). W07 consumes counts; W06 consumes
correlation fields (space identity, IPA range). Events carry no Guest data
beyond addresses already exposed by fault paths.
