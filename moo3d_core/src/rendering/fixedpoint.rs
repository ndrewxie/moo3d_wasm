#[derive(Clone, Copy, Debug)]
pub struct Fixed32(i32);

const FIXED32_SCALE: i32 = 16;
const FIXED32_POW: i32 = 2i32.pow(FIXED32_SCALE as u32);
const FIXED32_FRACTION_MASK: i32 = 0x0000ffff;
const FIXED32_WHOLE_MASK: i32 = -1 ^ FIXED32_FRACTION_MASK;

impl Fixed32 {
    #[inline(always)]
    pub fn from_f32(input: f32) -> Self {
        Self((FIXED32_POW as f32 * input) as i32)
    }
    #[inline(always)]
    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / (FIXED32_POW as f32)
    }
    #[inline(always)]
    pub fn from_i32(input: i32) -> Self {
        Self(input << FIXED32_SCALE)
    }
    #[inline(always)]
    pub fn to_i32(self) -> i32 {
        self.0 >> FIXED32_SCALE
    }
    #[inline(always)]
    pub fn fraction_part(self) -> Self {
        Self(self.0 & FIXED32_FRACTION_MASK)
    }
    #[inline(always)]
    pub fn whole_part(self) -> Self {
        Self(self.0 & FIXED32_WHOLE_MASK)
    }
    #[inline(always)]
    pub fn plus(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
    #[inline(always)]
    pub fn minus(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
    #[inline(always)]
    pub fn divide(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl std::fmt::Display for Fixed32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}