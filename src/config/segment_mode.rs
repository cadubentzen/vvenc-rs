use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegmentMode {
    Off,
    First,
    Mid,
    Last,
}

impl IntoFFI<u32> for SegmentMode {
    fn into_ffi(self) -> u32 {
        match self {
            SegmentMode::Off => ffi::vvencSegmentMode_VVENC_SEG_OFF,
            SegmentMode::First => ffi::vvencSegmentMode_VVENC_SEG_FIRST,
            SegmentMode::Mid => ffi::vvencSegmentMode_VVENC_SEG_MID,
            SegmentMode::Last => ffi::vvencSegmentMode_VVENC_SEG_LAST,
        }
    }
}
