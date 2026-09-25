//! P1 address arithmetic. Identity translation is a temporary W08 operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteSize(pub u64);

impl PhysAddr {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    /// Raw output is used only at descriptor/register and audited pointer boundaries.
    pub const fn raw(self) -> u64 {
        self.0
    }
    pub fn checked_add(self, bytes: ByteSize) -> Option<Self> {
        self.0.checked_add(bytes.0).map(Self)
    }
}
impl VirtAddr {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub const fn raw(self) -> u64 {
        self.0
    }
    pub fn checked_add(self, bytes: ByteSize) -> Option<Self> {
        self.0.checked_add(bytes.0).map(Self)
    }
    pub fn distance_from(self, base: Self) -> Option<ByteSize> {
        self.0.checked_sub(base.0).map(ByteSize)
    }
    pub const fn is_page_aligned(self) -> bool {
        self.0 & 4095 == 0
    }
    pub const fn page_base(self) -> Self {
        Self(self.0 & !4095)
    }
    /// Named P1-only translation: caller's region inventory establishes VA=PA.
    pub const fn p1_identity_physical(self) -> PhysAddr {
        PhysAddr::new(self.0)
    }
}
