# P2-W01 Intake Boundary Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W01 detailed design](README.md).

## 1. Package outcome

One validated-intake boundary with this observable outcome: given the boot
DTB location supplied by the P1 boot environment, either (a) a
`ValidatedBootDtb` handle exists — meaning placement, header, structure block,
strings block, and reservation block have all passed bounded validation — and
every later read through its cursor is in-bounds by construction, or (b) boot
stops with exactly one distinct local diagnostic from the taxonomy in §3 and
no handle exists. There is no third state: a partially validated handle is
never observable.

## 2. Assumed prerequisite contracts and failure boundaries

W01 consumes four inputs that P1/P0 must supply. The repository today contains
their plans only — no P1 implementation exists — so these are **assumed
contracts**: W01 designs against the declared semantics, and a divergence or
absence at integration time is an upstream defect per task book §2 ("Missing
P0/P1 inputs are upstream defects; P2 must not work around them").

| # | Assumed contract | Declared source | W01 relies on | Failure boundary if delivered differently |
|---|---|---|---|---|
| A1 | Boot-information handoff: the active DTB's physical address and byte length are known to EL2 at entry and stable for the whole boot phase | P1-W01 (reference boot contract, "DTB treatment"); P1-W09 stable-runtime state | A single `(physical address, length)` pair, fixed after handoff | If P1 cannot state a stable pair, intake has no input to validate: blocked, record upstream defect; do not scan memory for a DTB magic |
| A2 | Host physical-access window: a P1-established stage-1 mapping through which bytes at physical addresses inside a declared coverage range can be read without further mapping work | P1-W08 (host Stage-1 address space, "boot-time data" region) | A read-only byte window function `(PhysAddr, ByteLen) -> &[u8]`-shaped access for ranges inside the declared coverage | If absent or coverage excludes the DTB, W01 cannot read the blob: blocked upstream defect; improvising an identity mapping is forbidden (P1-W08's no-identity-map rule) |
| A3 | Hypervisor image physical range: one authoritative `[start, len)` range of the loaded EL2 image | P1-W01/P1-W08; consumed by W03 | One immutable range used for the overlap check | If P1 states no range, the hypervisor-overlap check (R-A01) cannot run: blocked upstream defect; the check must not be skipped silently |
| A4 | Diagnostics and unsafe governance: leveled boot diagnostics with local detail, panic/failure classification, `SAFETY`-comment discipline and unsafe inventory | P0-W12, P0-W14, P0-W10 | A diagnostic emission path usable before allocation exists; failure classes | If absent, W01 diagnostics have no channel: blocked upstream defect; `printf`-style improvisation is not authorized |

The host-side unit-test environment is the P0-W08 baseline. All W01 validation
logic (§4–§6 contracts) runs on host against synthesized byte fixtures; the
only code paths that cannot run on host are the A2 window adapters, which are
thin and excluded from unit-test scope (they are exercised by W09 QEMU
evidence).

## 3. Untrusted-input model and diagnostic taxonomy

### 3.1 Untrusted-input model

The DTB is firmware-controlled data at a security boundary: a corrupted or
hostile blob must never cause an out-of-bounds access, an unchecked
arithmetic overflow, unbounded recursion or iteration, or a nondiagnostic
halt. Concrete rules, binding on every contract in
[03](03-code-contracts-intake.md):

1. Every length/offset is `u64`/`u32` read as big-endian and converted with
   checked arithmetic; any `checked_add`/`checked_sub` failure is a
   diagnostic, never a wrap.
2. Every derived span (block range, property value, string, reservation
   entry) is validated as `offset + length <= bounds` before the first read.
3. Iteration over the structure block and reservation list is bounded by both
   the validated block span and explicit structural caps
   ([02 §4](02-architecture-and-state.md)); caps exceeded are diagnostics.
4. No allocation occurs during intake ([02 §5](02-architecture-and-state.md)).
5. Reads go only through the cursor or the A2 window boundary; no raw pointer
   arithmetic outside [03 §3](03-code-contracts-intake.md).

### 3.2 Diagnostic taxonomy

All intake failures are fatal (README Decision 4), but each failure class is
distinct so hosts, offline checking (W07), and boot logs (W09) can assert
exactly what was rejected. The design fixes these stable diagnostic classes;
per-class detail fields are defined in
[03 §2](03-code-contracts-intake.md):

| Class | Meaning | Typical trigger |
|---|---|---|
| `DtbAbsent` | No boot description was supplied (null/zero address or zero length) | Firmware passed no DTB pointer |
| `DtbMisaligned` | Base or a derived block offset violates §6 alignment rules | Firmware placed blob at odd address |
| `DtbUnreachable` | Blob range is not inside the A2 access-window coverage | P1 window excludes the blob |
| `DtbSizeInvalid` | Length below minimum header size, above the configured maximum, or `total_size` disagrees with the supplied length | Truncated or absurd length |
| `DtbImageOverlap` | Blob range overlaps the A3 hypervisor image range | Firmware loaded blob over the image |
| `DtbHeaderInvalid` | Magic, version, `last_comp_version`, or a block offset/size pair fails validation | Corrupted header |
| `DtbStructureInvalid` | Token stream malformed: unbalanced nodes, missing `FDT_END`, trailing tokens, string-table offset out of bounds, property length out of bounds, depth/count cap exceeded | Corrupted or hostile structure block |
| `DtbReservationInvalid` | Reservation list does not terminate in bounds, or entry spans are inconsistent | Corrupted reservation block |

Each diagnostic carries the failing field or offset class as structured detail
(field name or byte offset class) sufficient for a boot log line and a host
test assertion. W01 never prints DTB content beyond offsets and field names;
blob bytes are data, not log input.

## 4. Placement and overlap rules (R-A01)

Given input pair `(dtb_phys, dtb_len)` from A1, image range from A3, and the
A2 window coverage:

1. **Presence.** `dtb_phys == 0 || dtb_len == 0` → `DtbAbsent`.
2. **Size sanity.** `dtb_len < DTB_HEADER_LEN (40)` or
   `dtb_len > config.max_dtb_size` → `DtbSizeInvalid`. The maximum is a boot
   configuration constant (stage-local value fixed in
   [03 §2](03-code-contracts-intake.md): 8 MiB default, rationale: the
   largest expected reference/fixture DTB is well under 1 MiB; 8 MiB bounds
   DoS exposure while leaving margin).
3. **Alignment.** `dtb_phys % 4 != 0` → `DtbMisaligned`. (Finer rules apply
   to derived offsets per README Decision 6; the base rule is checked before
   any byte access.)
4. **Reachability.** The span `[dtb_phys, dtb_phys + dtb_len)` must lie
   inside the A2 coverage; checked with `checked_add` → else
   `DtbUnreachable`.
5. **Image overlap.** The span must not intersect the A3 image range;
   intersection by closed/open interval arithmetic → else `DtbImageOverlap`
   (fatal per README Decision 5).

Note explicitly: W01 does **not** check whether the DTB lies inside RAM.
RAM knowledge does not exist until W02 discovery; claiming it at intake would
invert the W01→W02 dependency. The task book's "host-access range" outcome is
satisfied by rule 4 (window reachability). Whether the DTB sits inside a RAM
bank, and its consequences, are W03 map facts.

## 5. Required foundational DT encodings (R-A02, R-A03)

The format rules W01 must enforce before any consumer runs; all are enforced
against the blob's own declared bounds:

- **Header:** magic `0xd00dfeed`; `total_size` equal to the A1 length and
  within §4 bounds; `off_dt_struct`, `off_dt_strings`, `off_mem_rsvmap` each
  4-byte aligned and within `total_size`; `size_dt_struct`,
  `size_dt_strings` non-negative spans within `total_size` with
  `checked_add` block ranges; `version >= 17`, `last_comp_version <= 17`
  (README Decision 3); `boot_cpuid_phys` is recorded verbatim for W02 (no
  semantics at intake); `size_dt_struct` present and validated (v17 field).
- **Structure block:** a flat token stream of `FDT_BEGIN_NODE`/`FDT_END_NODE`/
  `FDT_PROP`/`FDT_NOP`/`FDT_END` big-endian u32 tokens; exactly one root node
  with an empty name; node nesting balanced; exactly one `FDT_END`, as the
  last structural token (trailing `FDT_NOP`s allowed, any other trailing
  token is invalid); property `len`/`nameoff` pairs with `nameoff + name_len`
  inside the strings block and `value` span inside the structure block; node
  names NUL-terminated, non-empty except root, without interior NULs; depth
  and count caps per [02 §4](02-architecture-and-state.md).
- **Strings block:** consumed only via validated offsets; W01 does not
  require NUL termination of the block itself, but every referenced property
  name must be NUL-terminated within the block (this is the encoding
  requirement the walker enforces when it resolves names).
- **Reservation block:** a sequence of 8-byte-aligned `(u64 address,
  u64 size)` entries terminated by an entry with `address == 0 && size == 0`;
  the terminator must occur within the validated blob bounds and within the
  entry cap ([02 §4](02-architecture-and-state.md)); non-terminated lists are
  `DtbReservationInvalid`. Entries are recorded verbatim, unsorted, with
  `checked_add` span arithmetic; `address == 0 && size > 0` entries are
  recorded and flagged for W03 treatment (zero-address reservations are a
  known ambiguity — W03 owns the policy; W01 only must not drop them
  silently).

What W01 explicitly does **not** do at this layer: interpret `reg` cells
(needs `#address-cells` semantics — W02), interpret property values beyond
the reservation block (W02), decide usability of any device node (W02), or
sort/dedupe anything (W03). W01's contract is: the token stream and the
reservation list are structurally trustworthy; every later read is in-bounds
by construction.

## 6. Unknown nodes and diagnostically relevant shapes (R-A04)

Intake is semantics-blind, so "unknown node" handling is: any node or property
whose name or content W01 does not understand is structurally validated and
safely ignorable at this layer. Two exceptions are diagnostically relevant and
must be *reported without rejecting*:

1. **Structural anomalies that pass the hard bounds** — e.g., depth exactly
   at cap, property names containing non-printable bytes, an empty
   (zero-property, zero-child) non-root node. These increment a recorded
   anomaly counter with the diagnostic class `StructureAnomaly` (informative,
   non-fatal). Rationale: they may indicate firmware or blob-generation
   defects worth surfacing to W07/W09 evidence, but rejecting them would
   exceed the intake contract.
2. **Caps near exhaustion** — recorded as anomalies for the same audience.

Everything else unknown is silently ignorable; silence is the specified
behavior, and the anomaly counter is the only observable trace. W02 defines
its own unknown-node policy for the nodes it walks; W07 defines which
anomalies are warnings in offline reports.

## 7. DTB lifetime and ownership

The DTB's memory is owned by firmware until W03 claims the range as protected.
W01's ownership obligations are exactly: read-only access, no write, no
release, no relocation, and publication of the validated range so W03 can
protect it. The handle's logical lifetime is "from successful intake until
hypervisor restart"; P2 has no revocation mechanism (no unmap, no copy),
which is precisely why a borrow-typed handle tying the blob bytes to the
boot-phase state is sufficient (see [02 §3](02-architecture-and-state.md)).
Copying or releasing the DTB later is Reserved (README Decision 1) and would
require a superseding design for W03's map and the handle's lifecycle.
