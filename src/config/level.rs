use crate::*;

#[derive(Debug, PartialEq)]
pub enum Level {
    Auto,
    Level1,
    Level2,
    Level2_1,
    Level3,
    Level3_1,
    Level4,
    Level4_1,
    Level5,
    Level5_1,
    Level5_2,
    Level6,
    Level6_1,
    Level6_2,
    Level6_3,
    Level15_5,
}

impl From<Level> for u32 {
    fn from(value: Level) -> Self {
        match value {
            Level::Auto => ffi::vvencLevel_VVENC_LEVEL_AUTO,
            Level::Level1 => ffi::vvencLevel_VVENC_LEVEL1,
            Level::Level2 => ffi::vvencLevel_VVENC_LEVEL2,
            Level::Level2_1 => ffi::vvencLevel_VVENC_LEVEL2_1,
            Level::Level3 => ffi::vvencLevel_VVENC_LEVEL3,
            Level::Level3_1 => ffi::vvencLevel_VVENC_LEVEL3_1,
            Level::Level4 => ffi::vvencLevel_VVENC_LEVEL4,
            Level::Level4_1 => ffi::vvencLevel_VVENC_LEVEL4_1,
            Level::Level5 => ffi::vvencLevel_VVENC_LEVEL5,
            Level::Level5_1 => ffi::vvencLevel_VVENC_LEVEL5_1,
            Level::Level5_2 => ffi::vvencLevel_VVENC_LEVEL5_2,
            Level::Level6 => ffi::vvencLevel_VVENC_LEVEL6,
            Level::Level6_1 => ffi::vvencLevel_VVENC_LEVEL6_1,
            Level::Level6_2 => ffi::vvencLevel_VVENC_LEVEL6_2,
            Level::Level6_3 => ffi::vvencLevel_VVENC_LEVEL6_3,
            Level::Level15_5 => ffi::vvencLevel_VVENC_LEVEL15_5,
        }
    }
}
