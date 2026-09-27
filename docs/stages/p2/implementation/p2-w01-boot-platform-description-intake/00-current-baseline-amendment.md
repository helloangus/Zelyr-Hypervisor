# P2-W01 current-baseline and DTB access amendment

**Status:** Implementation design authorized by the request to resolve the
P2-W01/W02 prerequisite conflict.
**Scope:** Pointer-only P1 handoff, P2-owned bounded DTB access and specification
corrections needed by W01/W02.
**Version:** v0.1
**Owner/change context:** P2-W01, 2026-09-26.
**Supersedes:** W01 A1/A2 assumed prerequisites and conflicting pseudocode in
the W01/W02 designs; the remaining package boundaries continue to apply.

## Ownership and scope

P1's completed handoff is the current baseline: x0 supplies an opaque physical
DTB pointer, the image bounds are linker-owned, and the DTB is unmapped at
P1 stable. P2-W01 owns obtaining its length and establishing read-only access.
This resolves the [prerequisite conflict](../p2-w01-w02-prerequisite-conflict.md)
without expanding the completed P1 validation claim. The user's request to
resolve the problem authorizes this stage-local design reconciliation.

The pure code lives in `hypervisor/src/platform/`, source-shared by the existing
host-test member. It consumes semantic address newtypes, not a base crate
assumed to exist. The architecture adapter lives under AArch64 Stage-1; the
reference boot adapter supplies the trusted read envelope. No crate, dependency,
allocator, guest mechanism, or permanent physical direct map is introduced.

## Input and access contract

`intake::validate(bytes, physical, coverage, image)` is the pure boundary.
It returns an immutable `ValidatedBootDtb` or one structured diagnostic. Bytes
are borrowed, the caller supplies their lifetime, and constructors of cursor
nodes remain private. Host fixtures supply owned bytes and synthetic ranges.

A `BootReadEnvelope` has a private constructor in the reference boot adapter;
the architecture window cannot be opened with an arbitrary caller-created span.
The reference boot adapter supplies a conservative physical read envelope
`[0x40000000, 0x48000000)`, justified only by the canonical QEMU recipe's
128 MiB minimum RAM. This is a bootstrap accessibility premise, not discovered
RAM or allocation ownership. Other firmware requires a separately reviewed
adapter. A DTB outside that envelope is explicitly unreachable, even if a
larger machine happens to contain additional RAM.

The architecture adapter runs once on the boot CPU after P1 stable, with DAIF
masked and before any allocator or AP exists:

1. Validate pointer presence, 8-byte alignment, checked 40-byte header extent,
   read-envelope containment and page-rounded image non-overlap before access.
2. Map only the header's one or two pages at a P2-private VA aperture starting
   at `0x80000000`. Read the header; reject bad magic/version or a declared
   size outside 40 bytes through 8 MiB.
3. Check the complete declared span and its rounded pages against the same
   trusted envelope and image exclusion. Extend the aperture only with new
   invalid-to-valid leaf descriptors. Do not replace a valid mapping.
4. Validate the complete borrowed blob using W01, then pass its handle to W02.
   Failure emits one P2 diagnostic and stops; no normalized result escapes.

One new L2 and five L3 pages are static, aligned, in-image storage; five L3
pages cover the worst-case 8 MiB blob plus its initial page offset. The
existing L1 slot 2 must be invalid; publication refuses a collision. All leaf
pages are Normal WB, non-shareable, read-only, execute-never using the existing
MAIR entry 1. Table stores are atomic aligned u64 writes. Populate children
before publishing their parent. Each publication uses DSB SY, TLBI ALLE2,
DSB SY, ISB before access. This is a bounded single-CPU invalid-to-valid
extension, not a remapping service; no break-before-make replacement occurs.

The sole new raw slice boundary borrows an aperture owner; it is necessary
because safe Rust cannot create a firmware-memory slice. It requires checked
coverage, successful descriptor publication, unchanged firmware bytes, no
writers/DMA, and no unmap until restart. The canonical loader provides the
immutable input premise. A repeated initialization or false premise is terminal.
The handle and normalized facts remain owned by the boot sequence while idle;
there is no convenience global for platform facts. An unsafe inventory entry
and independent soundness review are required before merging this boundary.

## Specification corrections and representations

The [DTSpec flattened format](https://devicetree-specification.readthedocs.io/en/stable/flattened-format.html)
places `boot_cpuid_phys` at byte 28, `size_dt_strings` at 32 and
`size_dt_struct` at 36. The previous W01 pseudocode interchanged fields 28
and 36. The strings block has no alignment requirement; structure requires
4-byte alignment and the reservation map 8-byte alignment. Validate disjoint
header/structure/strings/reservation spans including the reservation terminator.
Only a terminated referenced property name is needed; cursor names are bytes,
so non-UTF-8 names remain safely ignorable with an anomaly count.

The [DTSpec base properties](https://devicetree-specification.readthedocs.io/en/stable/devicetree-basics.html)
default to address-cells 2 and size-cells 1; `/cpus` additionally allows size
cells 0. Child cells come from the immediate parent, not arbitrary ancestors.
CPU size-cells must be zero; a nonzero size-cell count is unsupported.
CPU nodes are identified by `device_type = cpu` or `cpu@` names; auxiliary
`cpu-map` nodes are skipped. Invalid individual records remain explicit states.
Lists use `FactState<T>` entries when necessary so malformed reservations and
artifacts cannot disappear from the W03 handoff.

GIC `redistributor-stride` is a 64-bit property. Timer interrupt specifiers use
the resolved `interrupt-parent` phandle, not the structural parent's cells.
Unsupported bus address translation is an explicit unusable fact; nested
physical addresses are never guessed. Singleton duplicates are counted and
first supported candidate policy is deterministic. Singleton status properties are retained as bounded annotations: absent or
`ok`/`okay` permits the recorded fact; `disabled`/`reserved` produces Absent,
and other declared status produces Unusable. Console resolution verifies
the target node exists; only one alias hop is attempted. String capacity errors
are explicit rather than truncated. Equality for determinism is field-wise;
Rust padding bytes are never serialized or compared.

W01 reservations use a capped borrowed iterator over the validated immutable
reservation block instead of copying 1024 entries onto the 64 KiB boot stack.
The count, termination and ranges are certified before publication; ordering
and zero-address preservation are unchanged. Host fixture builders may use
standard-library allocation; production intake/discovery never allocates.

## Validation and completion

Host tests cover zero-read placement rejection, bad/truncated headers, block
overlap, token and reservation caps, malformed strings, non-forgeable cursor
access, cell decoding, per-domain fact states, capacity failures and repeated
normalization. Mapping model tests check rounded containment, image exclusion,
aperture capacity and RO/XN attributes. QEMU smoke validates this new access
boundary and the W01-to-W02 boot chain; W09's full matrix remains its own work.
Existing P1 regression remains required for the affected default image.
