use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Preset {
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    FirstPass,
    ToolTest,
}

impl IntoFFI<i32> for Preset {
    fn into_ffi(self) -> i32 {
        match self {
            Preset::Faster => ffi::vvencPresetMode_VVENC_FASTER,
            Preset::Fast => ffi::vvencPresetMode_VVENC_FAST,
            Preset::Medium => ffi::vvencPresetMode_VVENC_MEDIUM,
            Preset::Slow => ffi::vvencPresetMode_VVENC_SLOW,
            Preset::Slower => ffi::vvencPresetMode_VVENC_SLOWER,
            Preset::FirstPass => ffi::vvencPresetMode_VVENC_FIRSTPASS,
            Preset::ToolTest => ffi::vvencPresetMode_VVENC_TOOLTEST,
        }
    }
}
