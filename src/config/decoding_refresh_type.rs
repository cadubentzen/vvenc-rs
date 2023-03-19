use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecodingRefreshType {
    None,
    Cra,
    Idr,
    RecoveryPointSEI,
    Idr2,
    CraCre,
}

impl IntoFFI<u32> for DecodingRefreshType {
    fn into_ffi(self) -> u32 {
        match self {
            DecodingRefreshType::None => ffi::vvencDecodingRefreshType_VVENC_DRT_NONE,
            DecodingRefreshType::Cra => ffi::vvencDecodingRefreshType_VVENC_DRT_CRA,
            DecodingRefreshType::Idr => ffi::vvencDecodingRefreshType_VVENC_DRT_IDR,
            DecodingRefreshType::RecoveryPointSEI => {
                ffi::vvencDecodingRefreshType_VVENC_DRT_RECOVERY_POINT_SEI
            }
            DecodingRefreshType::Idr2 => ffi::vvencDecodingRefreshType_VVENC_DRT_IDR2,
            DecodingRefreshType::CraCre => ffi::vvencDecodingRefreshType_VVENC_DRT_CRA_CRE,
        }
    }
}
