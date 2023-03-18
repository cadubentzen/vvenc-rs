use crate::*;

#[derive(Debug)]
pub enum SliceType {
    B,
    P,
    I,
}

impl TryFrom<u32> for SliceType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            ffi::vvencSliceType_VVENC_B_SLICE => Ok(Self::B),
            ffi::vvencSliceType_VVENC_P_SLICE => Ok(Self::P),
            ffi::vvencSliceType_VVENC_I_SLICE => Ok(Self::I),
            _ => Err(value),
        }
    }
}
