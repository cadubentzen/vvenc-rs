use crate::ffi;

pub enum Preset {
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    FirstPass,
    ToolTest,
}

impl From<Preset> for i32 {
    fn from(value: Preset) -> Self {
        match value {
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
