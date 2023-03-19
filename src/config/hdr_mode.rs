use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HdrMode {
    Off,
    Pq,
    Hlg,
    PqBt2020,
    HlgBt2020,
    UserDefined,
    Bt709,
    Bt2020,
    Bt470Bg,
}

impl IntoFFI<u32> for HdrMode {
    fn into_ffi(self) -> u32 {
        match self {
            HdrMode::Off => ffi::vvencHDRMode_VVENC_HDR_OFF,
            HdrMode::Pq => ffi::vvencHDRMode_VVENC_HDR_PQ,
            HdrMode::Hlg => ffi::vvencHDRMode_VVENC_HDR_HLG,
            HdrMode::PqBt2020 => ffi::vvencHDRMode_VVENC_HDR_PQ_BT2020,
            HdrMode::HlgBt2020 => ffi::vvencHDRMode_VVENC_HDR_HLG_BT2020,
            HdrMode::UserDefined => ffi::vvencHDRMode_VVENC_HDR_USER_DEFINED,
            HdrMode::Bt709 => ffi::vvencHDRMode_VVENC_SDR_BT709,
            HdrMode::Bt2020 => ffi::vvencHDRMode_VVENC_SDR_BT2020,
            HdrMode::Bt470Bg => ffi::vvencHDRMode_VVENC_SDR_BT470BG,
        }
    }
}
