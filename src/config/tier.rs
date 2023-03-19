use crate::*;

#[derive(Debug, PartialEq)]
pub enum Tier {
    Main,
    High,
}

impl From<Tier> for u32 {
    fn from(value: Tier) -> Self {
        match value {
            Tier::Main => ffi::vvencTier_VVENC_TIER_MAIN,
            Tier::High => ffi::vvencTier_VVENC_TIER_HIGH,
        }
    }
}
