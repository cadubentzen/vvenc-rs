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
    ) -> Result<Self> {
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

    pub fn with_ticks_per_second(mut self, ticks_per_second: i32) -> Self {
        self.ffi_config.m_TicksPerSecond = ticks_per_second;
        self
    }

    pub fn with_input_bit_depth(mut self, input_bit_depth: [i32; 2]) -> Self {
        self.ffi_config.m_inputBitDepth[0] = input_bit_depth[0];
        self.ffi_config.m_inputBitDepth[1] = input_bit_depth[1];
        self
    }

    pub fn with_frames_to_be_encoded(mut self, num_frames: i32) -> Self {
        self.ffi_config.m_framesToBeEncoded = num_frames;
        self
    }

    pub fn with_num_threads(mut self, num_threads: i32) -> Self {
        self.ffi_config.m_numThreads = num_threads;
        self
    }

    pub fn with_num_tiles(mut self, num_columns: u32, num_rows: u32) -> Self {
        self.ffi_config.m_numTileCols = num_columns;
        self.ffi_config.m_numTileRows = num_rows;
        self
    }

    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.ffi_config.m_verbosity = verbosity.into();
        self
    }

    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.ffi_config.m_profile = profile.into();
        self
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

    pub fn internal_chroma_format(&self) -> ChromaFormat {
        match self.ffi_config.m_internChromaFormat.try_into() {
            Ok(chroma_format) => chroma_format,
            Err(ffi_chroma_format) => panic!("chroma format {} from libvvenc not enumerated by this crate. Please file an issue.", ffi_chroma_format),
        }
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
