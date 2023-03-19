use crate::*;
use std::mem::MaybeUninit;

pub struct Config {
    ffi_config: ffi::vvenc_config,
}

pub struct ConfigBuilder {
    pub(crate) ffi_config: ffi::vvenc_config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        let mut ffi_config = MaybeUninit::uninit();

        unsafe {
            ffi::vvenc_config_default(ffi_config.as_mut_ptr());
        }

        Self {
            // SAFETY: vvenc_init_default should have fully initialized the config
            ffi_config: unsafe { ffi_config.assume_init() },
        }
    }

    pub fn with_default(
        mut self,
        width: i32,
        height: i32,
        framerate: i32,
        bitrate: i32,
        qp: i32,
        preset: Preset,
    ) -> Result<Self, Error> {
        unsafe {
            ffi::vvenc_init_default(
                &mut self.ffi_config,
                width,
                height,
                framerate,
                bitrate,
                qp,
                preset.into(),
            )
        }
        .to_result()?;
        Ok(self)
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub(crate) fn with_ffi(ffi_config: ffi::vvenc_config) -> Self {
        Self { ffi_config }
    }

    // TODO: add getters and setters for at core and basic config params
    pub fn source_width(&self) -> i32 {
        self.ffi_config.m_SourceWidth
    }

    pub fn source_height(&self) -> i32 {
        self.ffi_config.m_SourceHeight
    }

    pub fn internal_chroma_format(&self) -> Result<ChromaFormat, u32> {
        self.ffi_config.m_internChromaFormat.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        Config::builder()
            .with_default(1280, 720, 30, 2_000_000, 23, Preset::Medium)
            .unwrap();
    }
}
