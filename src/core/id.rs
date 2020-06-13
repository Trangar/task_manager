#[derive(Eq, Copy, Clone)]
pub struct ID(u32);

const GENERATION_SHIFT: u32 = 24;
const GENERATION_MASK: u32 = (u8::max_value() as u32) << GENERATION_SHIFT;
const MAX_INDEX: u32 = (1 << GENERATION_SHIFT) - 1;

impl ID {
    pub fn new(index: u32, generation: u8) -> ID {
        if index > MAX_INDEX {
            panic!(
                "Tried to create an index that was out of range (max {})",
                MAX_INDEX
            );
        }
        unsafe { ID::new_unchecked(index, generation) }
    }

    pub(crate) unsafe fn new_unchecked(index: u32, generation: u8) -> ID {
        ID((u32::from(generation) << GENERATION_SHIFT) | index)
    }

    pub fn index(self) -> usize {
        (self.0 & MAX_INDEX) as usize
    }

    pub fn generation(self) -> u8 {
        (self.0 >> GENERATION_SHIFT) as u8
    }

    pub fn is_same_generation(self, other: Self) -> bool {
        (self.0 & GENERATION_MASK) == (other.0 & GENERATION_MASK)
    }

    pub(crate) unsafe fn from_u64(v: u64) -> ID {
        ID(v as u32)
    }

    pub(crate) fn to_u64(self) -> u64 {
        u64::from(self.0)
    }

    pub fn next_generation(self) -> ID {
        let index = self.index();
        let generation = self.generation().wrapping_add(1);
        // Guaranteed to be as safe as the current ID
        unsafe { ID::new_unchecked(index as u32, generation) }
    }
}

impl std::hash::Hash for ID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl std::cmp::PartialEq<ID> for ID {
    fn eq(&self, other: &ID) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Debug for ID {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} (gen {})", self.index(), self.generation())
    }
}

impl std::fmt::Display for ID {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} (gen {})", self.index(), self.generation())
    }
}
