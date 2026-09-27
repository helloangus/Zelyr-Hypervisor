//! Bounded FDT validation. No physical reads, allocation, or device semantics.
use crate::boot::address::{ByteSize, PhysAddr};

pub const MAX_SIZE: usize = 8 * 1024 * 1024;
const MAX_DEPTH: usize = 32;
const MAX_NODES: usize = 4096;
const MAX_PROPS: usize = 16384;
const MAX_NAME: usize = 256;
const MAX_RESERVATIONS: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub base: PhysAddr,
    pub len: ByteSize,
}
impl Span {
    pub fn end(self) -> Option<PhysAddr> {
        self.base.checked_add(self.len)
    }
    pub fn contains(self, other: Self) -> bool {
        match (self.end(), other.end()) {
            (Some(end), Some(other_end)) => self.base <= other.base && other_end <= end,
            _ => false,
        }
    }
    pub fn overlaps(self, other: Self) -> bool {
        match (self.end(), other.end()) {
            (Some(end), Some(other_end)) => self.base < other_end && other.base < end,
            _ => true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Absent,
    Alignment,
    Unreachable,
    Access(&'static str),
    Size,
    ImageOverlap,
    Header(&'static str),
    Structure(&'static str),
    Reservation(&'static str),
}
impl Error {
    pub fn detail(self) -> &'static str {
        match self {
            Self::Header(s) | Self::Structure(s) | Self::Reservation(s) | Self::Access(s) => s,
            _ => self.class(),
        }
    }
    pub fn class(self) -> &'static str {
        match self {
            Self::Absent => "DtbAbsent",
            Self::Alignment => "DtbMisaligned",
            Self::Unreachable | Self::Access(_) => "DtbUnreachable",
            Self::Size => "DtbSizeInvalid",
            Self::ImageOverlap => "DtbImageOverlap",
            Self::Header(_) => "DtbHeaderInvalid",
            Self::Structure(_) => "DtbStructureInvalid",
            Self::Reservation(_) => "DtbReservationInvalid",
        }
    }
}
pub fn placement(span: Span, coverage: Span, image: Span) -> Result<(), Error> {
    if span.base.raw() == 0 || span.len.0 == 0 {
        return Err(Error::Absent);
    }
    if span.len.0 < 40 || span.len.0 > MAX_SIZE as u64 {
        return Err(Error::Size);
    }
    if span.base.raw() & 7 != 0 {
        return Err(Error::Alignment);
    }
    if !coverage.contains(span) {
        return Err(Error::Unreachable);
    }
    if span.overlaps(image) {
        return Err(Error::ImageOverlap);
    }
    Ok(())
}
fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_be_bytes(bytes.get(offset..end)?.try_into().ok()?))
}
fn u64_at(bytes: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(8)?;
    Some(u64::from_be_bytes(bytes.get(offset..end)?.try_into().ok()?))
}
fn aligned(value: usize) -> Option<usize> {
    value.checked_add(3).map(|v| v & !3)
}
fn cstring(bytes: &[u8], offset: usize) -> Option<&[u8]> {
    let tail = bytes.get(offset..)?;
    let end = tail.iter().take(MAX_NAME + 1).position(|b| *b == 0)?;
    Some(&tail[..end])
}
/// Decode only a bounded header view; caller still validates the full extent.
pub fn declared_size(header: &[u8]) -> Result<usize, Error> {
    if header.len() < 40 {
        return Err(Error::Size);
    }
    if u32_at(header, 0) != Some(0xd00dfeed) {
        return Err(Error::Header("magic"));
    }
    let size = u32_at(header, 4).ok_or(Error::Size)? as usize;
    if !(40..=MAX_SIZE).contains(&size) {
        return Err(Error::Size);
    }
    if u32_at(header, 20).is_none_or(|v| v < 17) || u32_at(header, 24).is_none_or(|v| v > 17) {
        return Err(Error::Header("version"));
    }
    Ok(size)
}
pub struct ValidatedBootDtb<'a> {
    bytes: &'a [u8],
    structure: &'a [u8],
    strings: &'a [u8],
    reservation_start: usize,
    reservation_count: usize,
    range: Span,
    boot_cpu: u32,
    version: u32,
    properties: usize,
    anomalies: usize,
    node_count: usize,
}
fn block(total: usize, offset: u32, size: u32) -> Result<(usize, usize), Error> {
    let start = offset as usize;
    let end = start
        .checked_add(size as usize)
        .ok_or(Error::Header("span"))?;
    if start < 40 || end > total {
        return Err(Error::Header("span"));
    }
    Ok((start, end))
}
fn intersect(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.1 && b.0 < a.1
}
pub fn validate(
    bytes: &[u8],
    physical: PhysAddr,
    coverage: Span,
    image: Span,
) -> Result<ValidatedBootDtb<'_>, Error> {
    let range = Span {
        base: physical,
        len: ByteSize(bytes.len() as u64),
    };
    placement(range, coverage, image)?;
    let size = declared_size(bytes)?;
    if size != bytes.len() {
        return Err(Error::Size);
    }
    let field = |off| u32_at(bytes, off).ok_or(Error::Header("truncated"));
    let structure = block(size, field(8)?, field(36)?)?;
    let strings = block(size, field(12)?, field(32)?)?;
    let rsv = field(16)? as usize;
    if structure.0 & 3 != 0 || rsv & 7 != 0 {
        return Err(Error::Alignment);
    }
    if rsv < 40 || rsv.checked_add(16).is_none_or(|v| v > size) {
        return Err(Error::Header("reservation-span"));
    }
    if intersect(structure, strings) {
        return Err(Error::Header("overlap"));
    }
    let mut handle = ValidatedBootDtb {
        bytes,
        structure: &bytes[structure.0..structure.1],
        strings: &bytes[strings.0..strings.1],
        reservation_start: rsv,
        reservation_count: 0,
        range,
        boot_cpu: field(28)?,
        version: field(20)?,
        properties: 0,
        anomalies: 0,
        node_count: 0,
    };
    validate_structure(&mut handle)?;
    let mut pos = rsv;
    loop {
        let end = pos.checked_add(16).ok_or(Error::Reservation("span"))?;
        if end > size || intersect((pos, end), structure) || intersect((pos, end), strings) {
            return Err(Error::Reservation("unterminated-or-overlap"));
        }
        let addr = u64_at(bytes, pos).ok_or(Error::Reservation("address"))?;
        let len = u64_at(bytes, pos + 8).ok_or(Error::Reservation("size"))?;
        if addr == 0 && len == 0 {
            break;
        }
        if handle.reservation_count == MAX_RESERVATIONS {
            return Err(Error::Reservation("cap"));
        }
        if addr.checked_add(len).is_none() {
            return Err(Error::Reservation("span"));
        }
        handle.reservation_count += 1;
        if addr == 0 {
            handle.anomalies += 1;
        }
        pos = end;
    }
    Ok(handle)
}
#[derive(Clone, Copy)]
enum Event<'a> {
    Begin(&'a [u8]),
    EndNode,
    Prop(&'a [u8], &'a [u8]),
    Nop,
    End,
}
fn event<'a>(structure: &'a [u8], strings: &'a [u8], pos: &mut usize) -> Option<Event<'a>> {
    let token = u32_at(structure, *pos)?;
    *pos = pos.checked_add(4)?;
    let result = match token {
        1 => {
            let name = cstring(structure, *pos)?;
            *pos = aligned(pos.checked_add(name.len() + 1)?)?;
            Event::Begin(name)
        }
        2 => Event::EndNode,
        3 => {
            let len = u32_at(structure, *pos)? as usize;
            let offset = u32_at(structure, pos.checked_add(4)?)? as usize;
            let start = pos.checked_add(8)?;
            let end = start.checked_add(len)?;
            let value = structure.get(start..end)?;
            let name = cstring(strings, offset)?;
            *pos = aligned(end)?;
            Event::Prop(name, value)
        }
        4 => Event::Nop,
        9 => Event::End,
        _ => return None,
    };
    (*pos <= structure.len()).then_some(result)
}
fn validate_structure(handle: &mut ValidatedBootDtb<'_>) -> Result<(), Error> {
    let mut pos = 0;
    let mut depth = 0;
    let mut properties = 0;
    let mut child_seen = [false; MAX_DEPTH];
    let mut content_seen = [false; MAX_DEPTH];
    let mut closed_root = false;
    let mut ended = false;
    while pos < handle.structure.len() {
        let token = event(handle.structure, handle.strings, &mut pos)
            .ok_or(Error::Structure("token-or-bounds"))?;
        if ended {
            if !matches!(token, Event::Nop) {
                return Err(Error::Structure("trailing"));
            }
            continue;
        }
        match token {
            Event::Begin(name) => {
                if closed_root || depth == MAX_DEPTH || handle.node_count == MAX_NODES {
                    return Err(Error::Structure("root-or-cap"));
                }
                if (depth == 0) != name.is_empty() {
                    return Err(Error::Structure("node-name"));
                }
                if depth != 0 {
                    child_seen[depth - 1] = true;
                    content_seen[depth - 1] = true;
                }
                child_seen[depth] = false;
                content_seen[depth] = false;
                depth += 1;
                handle.node_count += 1;
                if name.iter().any(|b| !b.is_ascii_graphic()) {
                    handle.anomalies += 1;
                }
                if depth == MAX_DEPTH {
                    handle.anomalies += 1;
                }
            }
            Event::EndNode => {
                if depth == 0 {
                    return Err(Error::Structure("balance"));
                }
                depth -= 1;
                if depth == 0 {
                    closed_root = true;
                } else if !content_seen[depth] {
                    handle.anomalies += 1;
                }
            }
            Event::Prop(name, _) => {
                if depth == 0 || child_seen[depth - 1] || name.is_empty() {
                    return Err(Error::Structure("property-order-or-name"));
                }
                properties += 1;
                if properties > MAX_PROPS {
                    return Err(Error::Structure("properties-cap"));
                }
                content_seen[depth - 1] = true;
                if name.iter().any(|b| !b.is_ascii_graphic()) {
                    handle.anomalies += 1;
                }
            }
            Event::End => {
                if !closed_root || depth != 0 {
                    return Err(Error::Structure("balance"));
                }
                ended = true;
            }
            Event::Nop => {}
        }
    }
    handle.properties = properties;
    if !ended {
        return Err(Error::Structure("missing-end"));
    }
    Ok(())
}
impl core::fmt::Debug for ValidatedBootDtb<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ValidatedBootDtb")
            .field("range", &self.range)
            .field("nodes", &self.node_count)
            .finish_non_exhaustive()
    }
}
impl<'a> ValidatedBootDtb<'a> {
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn properties(&self) -> usize {
        self.properties
    }
    pub fn range(&self) -> Span {
        self.range
    }
    pub fn boot_cpu(&self) -> u32 {
        self.boot_cpu
    }
    pub fn anomalies(&self) -> usize {
        self.anomalies
    }
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    pub fn nodes(&self) -> Nodes<'_> {
        Nodes {
            handle: self,
            pos: 0,
            depth: 0,
            stack: [0; MAX_DEPTH],
        }
    }
    pub fn find(&self, path: &[u8]) -> Option<Node<'_>> {
        if path.first() != Some(&b'/') {
            return None;
        }
        let mut current = self.nodes().next()?;
        for component in path[1..].split(|b| *b == b'/').filter(|v| !v.is_empty()) {
            current = self
                .nodes()
                .find(|n| n.parent == Some(current.offset) && n.name == component)?;
        }
        Some(current)
    }
    pub fn reservations(&self) -> impl Iterator<Item = Span> + '_ {
        (0..self.reservation_count).filter_map(|i| {
            let pos = self.reservation_start + i * 16;
            Some(Span {
                base: PhysAddr::new(u64_at(self.bytes, pos)?),
                len: ByteSize(u64_at(self.bytes, pos + 8)?),
            })
        })
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NodeId(usize);

#[derive(Clone, Copy)]
pub struct Node<'a> {
    handle: &'a ValidatedBootDtb<'a>,
    offset: usize,
    pub name: &'a [u8],
    parent: Option<usize>,
}
impl<'a> Node<'a> {
    pub fn properties(self) -> Properties<'a> {
        let mut pos = self.offset;
        let _ = event(self.handle.structure, self.handle.strings, &mut pos);
        Properties {
            handle: self.handle,
            pos,
            ended: false,
        }
    }
    pub fn property(self, wanted: &[u8]) -> Option<&'a [u8]> {
        self.properties()
            .find(|p| p.name() == wanted)
            .map(|p| p.value())
    }
    pub fn parent(self) -> Option<Self> {
        self.handle.nodes().find(|n| Some(n.offset) == self.parent)
    }
    pub fn children(self) -> impl Iterator<Item = Self> {
        self.handle
            .nodes()
            .filter(move |n| n.parent == Some(self.offset))
    }
    pub fn is_root(self) -> bool {
        self.parent.is_none()
    }
    pub fn id(self) -> NodeId {
        NodeId(self.offset)
    }
    pub fn parent_id(self) -> Option<NodeId> {
        self.parent.map(NodeId)
    }
}
pub struct Property<'a> {
    name: &'a [u8],
    value: &'a [u8],
}
impl<'a> Property<'a> {
    pub fn name(&self) -> &'a [u8] {
        self.name
    }
    pub fn value(&self) -> &'a [u8] {
        self.value
    }
}
pub struct Properties<'a> {
    handle: &'a ValidatedBootDtb<'a>,
    pos: usize,
    ended: bool,
}
impl<'a> Iterator for Properties<'a> {
    type Item = Property<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        loop {
            match event(self.handle.structure, self.handle.strings, &mut self.pos) {
                Some(Event::Prop(name, value)) => return Some(Property { name, value }),
                Some(Event::Nop) => {}
                _ => {
                    self.ended = true;
                    return None;
                }
            }
        }
    }
}
pub struct Nodes<'a> {
    handle: &'a ValidatedBootDtb<'a>,
    pos: usize,
    depth: usize,
    stack: [usize; MAX_DEPTH],
}
impl Nodes<'_> {
    /// Skip the remainder of the currently open node, retaining its siblings.
    pub fn skip_subtree(&mut self) {
        let initial = self.depth;
        if initial == 0 {
            return;
        }
        while let Some(token) = event(self.handle.structure, self.handle.strings, &mut self.pos) {
            match token {
                Event::Begin(_) => self.depth += 1,
                Event::EndNode => {
                    self.depth -= 1;
                    if self.depth < initial {
                        return;
                    }
                }
                Event::End => return,
                _ => {}
            }
        }
    }
}
impl<'a> Iterator for Nodes<'a> {
    type Item = Node<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let offset = self.pos;
            match event(self.handle.structure, self.handle.strings, &mut self.pos)? {
                Event::Begin(name) => {
                    let parent = self.depth.checked_sub(1).map(|i| self.stack[i]);
                    self.stack[self.depth] = offset;
                    self.depth += 1;
                    return Some(Node {
                        handle: self.handle,
                        offset,
                        name,
                        parent,
                    });
                }
                Event::EndNode => self.depth = self.depth.checked_sub(1)?,
                Event::End => return None,
                _ => {}
            }
        }
    }
}
