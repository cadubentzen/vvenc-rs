use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Profile {
    Auto,
    Main10,
    Main10StillPicture,
    Main10_444,
    Main10_444StillPicture,
    MultilayerMain10,
    MultilayerMain10StillPicture,
    MultilayerMain10_444,
    MultilayerMain10_444StillPicture,
}

impl IntoFFI<u32> for Profile {
    fn into_ffi(self) -> u32 {
        match self {
            Profile::Auto => ffi::vvencProfile_VVENC_PROFILE_AUTO,
            Profile::Main10 => ffi::vvencProfile_VVENC_MAIN_10,
            Profile::Main10StillPicture => ffi::vvencProfile_VVENC_MAIN_10_STILL_PICTURE,
            Profile::Main10_444 => ffi::vvencProfile_VVENC_MAIN_10_444,
            Profile::Main10_444StillPicture => ffi::vvencProfile_VVENC_MAIN_10_444_STILL_PICTURE,
            Profile::MultilayerMain10 => ffi::vvencProfile_VVENC_MULTILAYER_MAIN_10,
            Profile::MultilayerMain10StillPicture => {
                ffi::vvencProfile_VVENC_MULTILAYER_MAIN_10_STILL_PICTURE
            }
            Profile::MultilayerMain10_444 => ffi::vvencProfile_VVENC_MULTILAYER_MAIN_10_444,
            Profile::MultilayerMain10_444StillPicture => {
                ffi::vvencProfile_VVENC_MULTILAYER_MAIN_10_444_STILL_PICTURE
            }
        }
    }
}
