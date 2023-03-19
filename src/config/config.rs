use crate::*;
use std::mem::MaybeUninit;

pub struct Config {
    ffi_config: ffi::vvenc_config,
}

pub struct ConfigBuilder {
    pub(crate) ffi_config: ffi::vvenc_config,
}

impl ConfigBuilder {
    pub fn with_default(
        width: i32,
        height: i32,
        framerate: i32,
        bitrate: i32,
        qp: i32,
        preset: Preset,
    ) -> Result<Self> {
        let mut ffi_config = MaybeUninit::uninit();

        unsafe {
            ffi::vvenc_init_default(
                ffi_config.as_mut_ptr(),
                width,
                height,
                framerate,
                bitrate,
                qp,
                preset.into_ffi(),
            )
        }
        .to_result()?;
        Ok(Self {
            // SAFETY: vvenc_init_default should have fully initialized the config
            ffi_config: unsafe { ffi_config.assume_init() },
        })
    }

    pub fn with_logging_handler<L: LoggingHandler>(mut self, handler: L) -> Self {
        let callback = Box::new(LoggingCallback::new(handler));
        unsafe {
            ffi::vvenc_set_msg_callback(
                &mut self.ffi_config,
                Box::into_raw(callback) as *mut core::ffi::c_void,
                Some(LoggingCallback::<L>::c_callback),
            )
        }
        self
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
        self.ffi_config.m_verbosity = verbosity.into_ffi();
        self
    }

    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.ffi_config.m_profile = profile.into_ffi();
        self
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.ffi_config.m_level = level.into_ffi();
        self
    }

    pub fn with_tier(mut self, tier: Tier) -> Self {
        self.ffi_config.m_levelTier = tier.into_ffi();
        self
    }

    pub fn with_intra_period(mut self, intra_period: i32) -> Self {
        self.ffi_config.m_IntraPeriod = intra_period;
        self
    }

    pub fn with_decoding_refresh_type(
        mut self,
        decoding_refresh_type: DecodingRefreshType,
    ) -> Self {
        self.ffi_config.m_DecodingRefreshType = decoding_refresh_type.into_ffi();
        self
    }

    pub fn with_gop_size(mut self, gop_size: i32) -> Self {
        self.ffi_config.m_GOPSize = gop_size;
        self
    }

    pub fn with_perceptual_qp_adaption(mut self, use_qpa: bool) -> Self {
        self.ffi_config.m_usePerceptQPA = use_qpa;
        self
    }

    pub fn with_num_passes(mut self, num_passes: i32) -> Self {
        self.ffi_config.m_RCNumPasses = num_passes;
        self
    }

    pub fn with_current_pass(mut self, current_pass: i32) -> Self {
        self.ffi_config.m_RCPass = current_pass;
        self
    }

    pub fn with_internal_bit_depth(mut self, internal_bit_depth: [i32; 2]) -> Self {
        self.ffi_config.m_internalBitDepth[0] = internal_bit_depth[0];
        self.ffi_config.m_internalBitDepth[1] = internal_bit_depth[1];
        self
    }

    pub fn with_hdr_mode(mut self, hdr_mode: HdrMode) -> Self {
        self.ffi_config.m_HdrMode = hdr_mode.into_ffi();
        self
    }

    pub fn with_segment_mode(mut self, segment_mode: SegmentMode) -> Self {
        self.ffi_config.m_SegmentMode = segment_mode.into_ffi();
        self
    }
}

impl Config {
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
        ConfigBuilder::with_default(1280, 720, 30, 2_000_000, 23, Preset::Medium)
            .unwrap()
            .with_gop_size(120)
            .with_profile(Profile::Main10)
            .with_tier(Tier::Main);
    }
}
