# P2-W01 Code Contracts — Intake Boundary

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W01 detailed design](README.md).  
**Contract notation:** per the implementation-design checklist §3 template.
Pseudocode is an algorithm outline for review, not runnable production code.
Type and function names are stage-local design freedom owned by this design
([README, decisions](README.md)); the P0 base-crate address/error primitives
it consumes are assumed contracts ([01 §2](01-intake-boundary.md)).

## 1. Data types

### 1.1 `DtbPhysicalRange` — untrusted input newtype

```text
Name and stability: DtbPhysicalRange, internal to the intake module set;
consumed by W03 via the handle. Stability: stage-local; not a public ABI.
Purpose and caller: carries the A1 boot-supplied (physical address, length)
pair so it cannot be confused with a validated range; constructed once by the
boot sequence.
Inputs / outputs: PhysAddr, ByteLen (P0 base-crate newtypes; assumed contract).
Preconditions / postconditions: none trusted — the whole point is that the
  contents are untrusted until intake validates them.
State and ownership: value type; no owner transition.
Concurrency/allocation context: constructed pre-allocation; Copy-shaped.
Errors and failure guarantee: none (a plain carrier).
Security/authorization checks: none here; all checks downstream.
Logic: `struct { phys: PhysAddr, len: ByteLen }`.
Validation: review that no code path converts it to a slice without passing
  through `intake::validate`.
```

### 1.2 `IntakeConfig` — validation limits

```text
Name and stability: IntakeConfig, internal. Holds the caps of
  [02 §4](02-architecture-and-state.md): max_dtb_size = 8 MiB, max_depth = 32,
  max_nodes = 4096, max_properties_total = 16384, max_reservations = 1024,
  max_name_len = 256.
Purpose and caller: single reviewable place for DoS bounds; read by all
  validators.
Inputs / outputs: constants only; no runtime input in P2.
Preconditions / postconditions: values fixed at compile time in P2; making
  them runtime-configurable is a later-stage change requiring design.
Errors: none.
Security checks: the caps are the iteration bounds — reviewers must treat a
  cap change as a security-relevant change.
Logic: plain const-bearing struct; no logic.
Validation: W01-DV06 asserts caps are actually enforced.
```

### 1.3 `IntakeDiagnostic` — failure taxonomy

```text
Name and stability: IntakeDiagnostic (enum), internal; logged by the boot
  failure path; asserted by W01 host tests and reused as fixture expectations
  by W08.
Purpose and caller: the ONLY error type crossing the intake boundary;
  classes fixed in [01 §3](01-intake-boundary.md).
Inputs / outputs: variants DtbAbsent, DtbMisaligned{where}, DtbUnreachable,
  DtbSizeInvalid{detail}, DtbImageOverlap, DtbHeaderInvalid{field},
  DtbStructureInvalid{detail}, DtbReservationInvalid{detail},
  StructureAnomaly{detail} (informative; never fatal alone).
Preconditions / postconditions: construction implies the failing stage is
  recorded; detail carries a field name or offset class, never blob content.
State and ownership: immutable value; no state change accompanies it.
Concurrency/allocation context: formed without allocation (static detail
  strings or small integers).
Errors: n/a (this is the error type).
Security/authorization checks: diagnostics must not echo untrusted bytes
  into logs (reflection surface); only classes, offsets, field names.
Logic: plain enum; a single `log()` formatting function maps it to one
  boot-log line via the P0-W12 diagnostic channel.
Validation: W01-DV09 (one input defect -> exactly one expected class).
```

### 1.4 `ValidatedHeader`

```text
Name and stability: ValidatedHeader, internal; composed into the handle.
Purpose and caller: decoded, range-checked header fields; consumers index
  blocks only through its pre-validated spans.
Inputs / outputs: produced by fdt_header::validate; fields: total_size,
  off_dt_struct, size_dt_struct, off_dt_strings, size_dt_strings,
  off_mem_rsvmap, version, last_comp_version, boot_cpuid_phys (raw, for W02).
Preconditions / postconditions: every span satisfies
  offset.checked_add(size) <= total_size and the alignment rules; version
  policy applied. Post: all block spans disjoint per format rules.
State and ownership: immutable; owned by the handle.
Errors: none post-construction.
Security checks: all arithmetic was checked at construction.
Logic: plain record.
Validation: covered by header tests (W01-DV04).
```

### 1.5 `ValidatedBootDtb` — the published handle

```text
Name and stability: ValidatedBootDtb, the W01 output contract; consumed by
  W02 (borrow), W03 (range queries), W07 (host re-validation produces the
  same type over a byte fixture). Stage-local name; W10 records it as the
  consumer-facing shape.
Purpose and caller: proves "this blob passed intake"; sole legal gateway to
  DTB bytes for all P2 consumers.
Inputs / outputs: composed of DtbPhysicalRange (validated),
  ValidatedHeader, ValidatedStructure marker, ValidatedReservations,
  anomaly counter, and the borrowed byte view.
Preconditions / postconditions: exists only if every validation stage
  passed; borrow ties byte views to boot-phase state (P2 has no DTB
  release/copy — [01 §7](01-intake-boundary.md)).
State and ownership: immutable after publication; owned by the boot
  sequence; W02 borrows.
Concurrency/allocation context: single-core boot publication; immutable
  afterward, so later concurrent readers need no lock (P3 note).
Errors: none post-publication.
Security/authorization checks: type is the authorization to read; no other
  path to the bytes is permitted in P2 code review.
Logic:
  struct ValidatedBootDtb<'a> {
      range: DtbPhysicalRange,      // now validated
      header: ValidatedHeader,
      reservations: ValidatedReservations,
      anomalies: AnomalyLog,        // counters, fixed capacity
      bytes: &'a [u8],              // through fdt_access only
  }
Validation: W01-DV10 (consumers can do everything they need without raw
  byte access).
```

### 1.6 `ValidatedReservations`

```text
Name and stability: ValidatedReservations, internal; consumed by W03 as
  protected-range input.
Purpose and caller: the terminated reservation entry list plus anomaly
  records.
Inputs / outputs: fixed-capacity array (capacity = max_reservations) of
  ReservationEntry { address: u64, size: u64 } in blob order (unsorted,
  verbatim), terminated list proven; entries with address == 0 && size > 0
  flagged (not dropped) per [01 §5](01-intake-boundary.md).
Preconditions / postconditions: terminator was found in bounds; entry
  spans formed with checked arithmetic (no range claim is made here — W03
  owns overlap/conflict treatment).
State and ownership: immutable; owned by the handle.
Concurrency/allocation context: no allocation; fixed capacity from
  IntakeConfig.
Errors: constructor returns DtbReservationInvalid on any violation.
Security checks: untrusted u64s are carried but never used as addresses in
  W01.
Logic: `struct { entries: [ReservationEntry; MAX], used: usize, zero_based:
  usize }`.
Validation: W01-DV07/DV08.
```

## 2. Function contracts — access boundary

### 2.1 `fdt_access::blob_bytes`

```text
Name and stability: fdt_access::blob_bytes — the module's ONLY unsafe
  function. Internal; audited as the W01 unsafe-inventory entry (P0-W10).
Purpose and caller: turn a validated physical span inside the A2 window
  coverage into a read-only byte slice; called once by intake::validate.
Inputs / outputs: window: &dyn HostPhysicalRead (A2 assumed contract,
  injected); span: (PhysAddr, ByteLen) already passing placement checks
  ([01 §4](01-intake-boundary.md)); output: &[u8] of exactly span length.
Preconditions / postconditions: caller guarantees span is inside window
  coverage and passed the presence/size/alignment/overlap checks; post: the
  returned slice addresses exactly the span; no lifetime beyond the boot
  phase (enforced by borrow of the injected window's mapping).
State and ownership: no state.
Concurrency/allocation context: single-core boot; no allocation.
Errors and failure guarantee: none returning — a precondition violation is a
  programming error; intake sequences the checks so this cannot be reached
  with an unvalidated span (order enforced in §5.1).
Security/authorization checks: this is the trust anchor; SAFETY comment
  must argue (a) window coverage proven by caller, (b) read-only, (c)
  pointer provenance from the P1 mapping, (d) no aliasing writer exists in
  P2 (DTB frozen — [01 §7](01-intake-boundary.md)).
Logic (pseudocode):
    let ptr = window.base_va_for(span.phys)?   // P1-provided; assumed
    let slice = slice_from_parts(ptr, span.len) // unsafe: see SAFETY
    return slice
Validation: audited in review (W01-DV07 documents the inventory entry);
  exercised on QEMU only via W09 (host tests use synthesized slices and do
  not call this function).
```

## 3. Function contracts — header validation

### 3.1 `fdt_header::validate`

```text
Name and stability: fdt_header::validate. Internal; called by
  intake::validate and by W07 offline checking.
Purpose and caller: decode and range-check the 10 u32 big-endian header
  fields; produce ValidatedHeader or a diagnostic.
Inputs / outputs: bytes: &[u8] (len >= DTB_HEADER_LEN guaranteed by
  placement stage), supplied_len: ByteLen (A1), config: &IntakeConfig.
Output: Result<ValidatedHeader, IntakeDiagnostic>.
Preconditions / postconditions: pre — bytes length == supplied_len;
  post — on Ok, every block span is within supplied_len, 4-byte aligned,
  with checked-add ranges, and version policy holds; on Err, nothing was
  mutated (bytes borrowed only).
State and ownership: none.
Concurrency/allocation context: no allocation; pure function.
Errors and failure guarantee: DtbSizeInvalid (total_size mismatch or cap),
  DtbHeaderInvalid{magic|version|alignment|span} with the failing field;
  input bytes untouched.
Security/authorization checks: all field reads bounds-checked (bytes.len()
  >= 40 established by placement); big-endian decode via explicit byte
  composition, no pointer casts.
Logic (pseudocode):
    if bytes.len() < 40: unreachable here (placement guarantees)
    magic   = be32(bytes, 0);  if magic != 0xd00dfeed: Err(Header{magic})
    total   = be32(bytes, 4) as u64
    if total != supplied_len: Err(Size{total_mismatch})
    if total > config.max_dtb_size: Err(Size{cap})
    off_struct = be32(bytes, 8);  len_struct = be32(bytes, 28)   # size_dt_struct (v17)
    off_strings= be32(bytes,12);  len_strings= be32(bytes,32)
    off_rsv    = be32(bytes,16)
    version    = be32(bytes,20); last_comp = be32(bytes,24)
    boot_cpu   = be32(bytes,36)
    if version < 17 or last_comp > 17: Err(Header{version})
    for each (off, len) in [(off_struct,len_struct),(off_strings,len_strings)]:
        if off % 4 != 0: Err(Header{alignment})
        if off.checked_add(len).is_none() or off+len > total: Err(Header{span})
    if off_rsv % 4 != 0: Err(Header{alignment})          # list itself is u32-offset
    if (blob_phys + off_rsv) % 8 != 0: Err(Header{alignment})  # u64 entries
    if off_rsv.checked_add(16) > total: Err(Header{span})      # room for terminator
    return Ok(ValidatedHeader{ ... })
Validation: W01-DV04 (each header failure mode has a fixture asserting its
  class).
```

## 4. Function contracts — structure validation and cursor

### 4.1 `fdt_structure::validate`

```text
Name and stability: fdt_structure::validate. Internal; called by
  intake::validate and W07.
Purpose and caller: walk the token stream once, enforcing well-formedness
  and caps; on success the block is certified and a cursor can be created.
Inputs / outputs: bytes: &[u8]; header: &ValidatedHeader; config:
  &IntakeConfig. Output: Result<ValidatedStructure, IntakeDiagnostic>
  (ValidatedStructure carries node/property counts and the anomaly list).
Preconditions / postconditions: pre — header validated; post — token
  stream certified: balanced nesting, exactly one FDT_END terminal, all
  property offsets/lengths/strings in bounds, caps respected; bytes
  untouched.
State and ownership: none (iteration state is local; results recorded into
  the returned record).
Concurrency/allocation context: no allocation; anomaly list uses a
  fixed-capacity array; iterative walk with an explicit depth counter —
  recursion is prohibited (untrusted-input depth).
Errors and failure guarantee: DtbStructureInvalid{detail} for every
  malformed case (truncated token, unknown token, unbalanced END_NODE,
  missing/late FDT_END, trailing non-NOP token, string offset out of
  bounds, property value out of bounds, name missing NUL, cap exceeded);
  no partial results escape.
Security/authorization checks: every token index advance is
  bounds-checked; every property len/nameoff checked before reading value
  or name; see [01 §5](01-intake-boundary.md).
Logic (pseudocode):
    pos = header.off_dt_struct; end = off_struct + len_struct
    depth = 0; nodes = 0; props = 0; root_seen = false; closed = false
    loop:
        if pos + 4 > end: Err(Structure{truncated})
        token = be32(bytes, pos); pos += 4
        match token:
          FDT_BEGIN_NODE:
            depth += 1; if depth > config.max_depth: Err(Structure{depth})
            name_end = find_nul(bytes, pos, end) else Err(Structure{name})
            name_len = name_end - pos
            if nodes > 0 and name_len == 0: Err(Structure{name})  # only root empty
            if name_len > config.max_name_len: Err(Structure{name})
            pos = align4(name_end + 1); nodes += 1
          FDT_END_NODE:
            if depth == 0: Err(Structure{balance}); depth -= 1
          FDT_PROP:
            if depth == 0: Err(Structure{prop_at_root})   # props only inside nodes
            if pos + 8 > end: Err(Structure{truncated})
            plen = be32(bytes, pos); nameoff = be32(bytes, pos+4)
            if nameoff >= header.size_dt_strings: Err(Structure{stroff})
            if !nul_terminated_in(strings_block, nameoff): Err(Structure{name})
            if pos+8 .checked_add(plen) beyond end: Err(Structure{proplen})
            pos = align4(pos + 8 + plen); props += 1
            if props > config.max_properties_total: Err(Structure{cap})
          FDT_NOP: pass
          FDT_END:
            if depth != 0: Err(Structure{balance})
            closed = true; break
          _: Err(Structure{token})
    if not closed: Err(Structure{no_end})
    while pos + 4 <= end and be32(bytes,pos) == FDT_NOP: pos += 4
    if pos != end: Err(Structure{trailing})        # only NOPs allowed after FDT_END
    if nodes < 1: Err(Structure{empty})            # root must exist
    return Ok(ValidatedStructure{nodes, props, anomalies})
Validation: W01-DV05/DV06 (malformed fixtures per failure mode; cap
  enforcement).
```

### 4.2 `StructureCursor` and accessors

```text
Name and stability: StructureCursor { next_node, skip_node, next_property,
  property_name, property_value }, internal; consumed by W02's walkers.
Purpose and caller: bounds-guaranteed read-only iteration over the
  validated structure block so semantic code cannot construct an
  out-of-bounds access.
Inputs / outputs: constructed from &ValidatedBootDtb; next_node ->
  Option<NodeRef>, next_property(node) -> Option<PropRef>;
  property_name(PropRef) -> &str (NUL already proven);
  property_value(PropRef) -> &[u8] (length already proven).
Preconditions / postconditions: cursor valid only while the handle is
  borrowed; iteration order is DT order (deterministic — W02 relies on
  this); skipping a subtree is supported so W02 can ignore unknown nodes
  cheaply.
State and ownership: cursor position is the only mutable state, owned by
  the cursor instance; the underlying bytes are immutably borrowed.
Concurrency/allocation context: no allocation; single reader assumed in
  P2 (multiple read-only cursors are safe by immutability).
Errors and failure guarantee: no error paths — every accessor returns
  validated data or None; malformed states are unreachable because
  validate() certified the stream.
Security/authorization checks: the type system is the check: PropRef/
  NodeRef values cannot be forged outside the module (constructors private).
Logic: cursor holds (pos, depth) within the certified span; accessors
  re-walk tokens with the same bounds rules as validate() but without
  re-validating (already certified); assertions (host-only) re-check
  invariants cheaply.
Validation: W01-DV07 (cursor cannot express an out-of-bounds read — host
  property test over random valid/invalid blobs).
```

## 5. Function contracts — reservation validation and orchestration

### 5.1 `fdt_reservation::validate`

```text
Name and stability: fdt_reservation::validate. Internal; called by
  intake::validate and W07.
Purpose and caller: validate the reservation list terminates in bounds and
  collect entries verbatim.
Inputs / outputs: bytes: &[u8]; header: &ValidatedHeader; config.
Output: Result<ValidatedReservations, IntakeDiagnostic>.
Preconditions / postconditions: pre — header validated (terminator room
  checked); post — either a terminated in-bounds list or a diagnostic;
  entries unsorted, unmodified; zero-address non-zero-size entries flagged,
  not dropped ([01 §5](01-intake-boundary.md)).
State and ownership: none.
Concurrency/allocation context: no allocation; fixed-capacity array.
Errors: DtbReservationInvalid{unterminated|cap|span}.
Security checks: loop bounded by both the entry cap and the blob span;
  checked_add on every entry span.
Logic (pseudocode):
    pos = header.off_mem_rsvmap; count = 0
    loop:
        if pos + 16 > total_size: Err(Reservation{unterminated})
        addr = be64(bytes, pos); size = be64(bytes, pos+8)
        if addr == 0 and size == 0: break            # terminator
        if count == config.max_reservations: Err(Reservation{cap})
        if addr.checked_add(size) is None: Err(Reservation{span})
        record entry (flag zero-address if addr == 0); count += 1
        pos += 16
    return Ok(...)
Validation: W01-DV07/DV08.
```

### 5.2 `intake::validate` — orchestrator

```text
Name and stability: intake::validate. The single entry point of W01;
  called once per boot by the boot sequence; called fresh per blob by W07.
Purpose and caller: sequence placement → header → structure → reservation
  and publish the handle.
Inputs / outputs: input: DtbPhysicalRange (A1), image_range:
  Option<PhysSpan> (A3; None means P1 did not supply it — treated as a
  blocked prerequisite, not an optional check, see failure boundary),
  window: &dyn HostPhysicalRead (A2), config: &IntakeConfig. Output:
  Result<ValidatedBootDtb, IntakeDiagnostic>.
Preconditions / postconditions: pre — A1/A3 supplied per contract; post —
  all-or-nothing publication ([02 §3](02-architecture-and-state.md)).
State and ownership: publication is the only state transition; the handle
  is owned by the caller (boot sequence).
Concurrency/allocation context: single-core boot; no allocation; no locks.
Errors and failure guarantee: first failing stage's diagnostic; nothing
  published on Err; blob and globals untouched.
Security/authorization checks: runs the placement rules
  ([01 §4](01-intake-boundary.md)) before any byte access; stage order is
  load-bearing for the unsafe boundary's precondition (§2.1).
Logic (pseudocode):
    check_presence(input) -> DtbAbsent?
    check_size_sanity(input, config) -> DtbSizeInvalid?
    check_alignment(input) -> DtbMisaligned?
    check_reachability(input, window.coverage()) -> DtbUnreachable?
    check_image_overlap(input, image_range) -> DtbImageOverlap?
        # image_range must be Some; None is an upstream-defect stop BEFORE
        # any read, with its own diagnostic path (blocked, not DtbAbsent)
    bytes = fdt_access::blob_bytes(window, input)      # sole unsafe step
    header = fdt_header::validate(bytes, input.len, config)?
    structure = fdt_structure::validate(bytes, &header, config)?
    reservations = fdt_reservation::validate(bytes, &header, config)?
    return Ok(ValidatedBootDtb { ... })
Validation: W01-DV01–DV03 (placement), DV09 (diagnostic locality: earliest
  independent failure reported), DV10 (consumer walkthrough).
```

## 6. Explicitly unauthorized interfaces

No serialization of the validated handle (never persist raw structs —
versioned wire formats are a later-stage contract if one is ever needed); no
mutation of blob bytes; no semantic property interpretation; no
`Display`/logging of property values; no public re-export of the cursor
internals. W07 re-runs the validators; it must not add a parallel validation
implementation.
