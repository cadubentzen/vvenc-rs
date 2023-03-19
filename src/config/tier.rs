use crate::*;

#[derive(Debug, PartialEq)]
pub enum Tier {
    Main,
    High,
}

impl IntoFFI<u32> for Tier {
    fn into_ffi(self) -> u32 {
        match self {
            Tier::Main => ffi::vvencTier_VVENC_TIER_MAIN,
            Tier::High => ffi::vvencTier_VVENC_TIER_HIGH,
        }
    }
}
