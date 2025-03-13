use std::{
    ffi::c_void,
    path::Path,
    ptr,
    sync::{Arc, Mutex},
};

use vsprintf::vsprintf;
use vvenc_sys::*;

#[derive(Debug)]
pub struct Encoder<Opaque> {
    inner: Arc<Mutex<InnerEncoder>>,
    _phantom: std::marker::PhantomData<Opaque>,
}

unsafe impl<Opaque> Sync for Encoder<Opaque> {}
unsafe impl<Opaque> Send for Encoder<Opaque> {}

#[derive(Debug)]
struct InnerEncoder {
    encoder: ptr::NonNull<vvencEncoder>,
}

impl Drop for InnerEncoder {
    fn drop(&mut self) {
        unsafe {
            vvenc_encoder_close(self.encoder.as_ptr());
        }
    }
}

pub type EncodeDone = bool;

impl<Opaque: Sized + Sync + Send> Encoder<Opaque> {
    pub fn with_config(mut config: Config) -> Result<Self, Error> {
        let Some(encoder) = ptr::NonNull::new(unsafe { vvenc_encoder_create() }) else {
            return Err(Error::Initialize);
        };
        let ret = unsafe { vvenc_encoder_open(encoder.as_ptr(), &mut config.inner) };
        #[allow(non_upper_case_globals)]
        match ret {
            ErrorCodes_VVENC_OK => Ok(Self {
                inner: Arc::new(Mutex::new(InnerEncoder { encoder })),
                _phantom: std::marker::PhantomData::default(),
            }),
            _ => Err(Error::new(ret)),
        }
    }

    pub fn init_pass(&mut self, pass: i32, stats_file: &Path) -> Result<(), Error> {
        let ret = unsafe {
            vvenc_init_pass(
                self.inner.lock().unwrap().encoder.as_ptr(),
                pass,
                stats_file
                    .as_os_str()
                    .to_str()
                    .ok_or(Error::Parameter)?
                    .as_ptr() as *const i8,
            )
        };
        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }
        Ok(())
    }

    pub fn encode<'b>(
        &mut self,
        yuv_buffer: &mut YUVBuffer<Opaque>,
        out_data: &'b mut [u8],
    ) -> Result<Option<AccessUnit<'b, Opaque>>, Error> {
        let mut au = AccessUnit::new(out_data);
        let mut encode_done = false;
        let ret = unsafe {
            vvenc_encode(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut yuv_buffer.inner,
                &mut au.inner,
                &mut encode_done,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok((!au.payload().is_empty()).then_some(au))
    }

    pub fn flush<'b>(
        &mut self,
        out_data: &'b mut [u8],
    ) -> Result<Option<(AccessUnit<'b, Opaque>, EncodeDone)>, Error> {
        let mut au = AccessUnit::new(out_data);
        let mut encode_done = false;
        let ret = unsafe {
            vvenc_encode(
                self.inner.lock().unwrap().encoder.as_ptr(),
                std::ptr::null_mut(),
                &mut au.inner,
                &mut encode_done,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok((!au.payload().is_empty()).then_some((au, encode_done)))
    }

    pub fn config(&self) -> Config {
        let mut config = Config::default();
        unsafe {
            vvenc_get_config(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut config.inner,
            )
        };
        config
    }
}

#[derive(Debug)]
pub struct Rational {
    pub num: i32,
    pub den: i32,
}

#[derive(Debug)]
pub struct Qp(pub u8);

impl Qp {
    pub fn new(value: u8) -> Result<Self, Error> {
        if value > 63 {
            return Err(Error::Parameter);
        }
        Ok(Self(value))
    }

    fn from_ffi(value: i32) -> Result<Self, Error> {
        if value > 63 {
            return Err(Error::Parameter);
        }
        Ok(Self(value as u8))
    }

    fn to_ffi(self) -> i32 {
        self.0 as i32
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Silent = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Notice = 4,
    Verbose = 5,
    Details = 6,
}

impl LogLevel {
    fn from_ffi(value: u32) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencMsgLevel_VVENC_SILENT => Self::Silent,
            vvencMsgLevel_VVENC_ERROR => Self::Error,
            vvencMsgLevel_VVENC_WARNING => Self::Warning,
            vvencMsgLevel_VVENC_INFO => Self::Info,
            vvencMsgLevel_VVENC_NOTICE => Self::Notice,
            vvencMsgLevel_VVENC_VERBOSE => Self::Verbose,
            vvencMsgLevel_VVENC_DETAILS => Self::Details,
            _ => unreachable!(),
        }
    }

    fn to_ffi(self) -> u32 {
        match self {
            Self::Silent => vvencMsgLevel_VVENC_SILENT,
            Self::Error => vvencMsgLevel_VVENC_ERROR,
            Self::Warning => vvencMsgLevel_VVENC_WARNING,
            Self::Info => vvencMsgLevel_VVENC_INFO,
            Self::Notice => vvencMsgLevel_VVENC_NOTICE,
            Self::Verbose => vvencMsgLevel_VVENC_VERBOSE,
            Self::Details => vvencMsgLevel_VVENC_DETAILS,
        }
    }
}

pub trait Logger {
    fn log(&self, level: LogLevel, message: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Auto,
    Main10,
    Main10StillPicture,
    Main10444,
    Main10444StillPicture,
    MultilayerMain10,
    MultilayerMain10StillPicture,
    MultilayerMain10444,
    MultilayerMain10444StillPicture,
}

impl Profile {
    #[inline]
    fn to_ffi(self) -> vvencProfile {
        match self {
            Self::Auto => vvencProfile_VVENC_PROFILE_AUTO,
            Self::Main10 => vvencProfile_VVENC_MAIN_10,
            Self::Main10StillPicture => vvencProfile_VVENC_MAIN_10_STILL_PICTURE,
            Self::Main10444 => vvencProfile_VVENC_MAIN_10_444,
            Self::Main10444StillPicture => vvencProfile_VVENC_MAIN_10_444_STILL_PICTURE,
            Self::MultilayerMain10 => vvencProfile_VVENC_MULTILAYER_MAIN_10,
            Self::MultilayerMain10StillPicture => {
                vvencProfile_VVENC_MULTILAYER_MAIN_10_STILL_PICTURE
            }
            Self::MultilayerMain10444 => vvencProfile_VVENC_MULTILAYER_MAIN_10_444,
            Self::MultilayerMain10444StillPicture => {
                vvencProfile_VVENC_MULTILAYER_MAIN_10_444_STILL_PICTURE
            }
        }
    }

    #[inline]
    fn from_ffi(value: vvencProfile) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencProfile_VVENC_PROFILE_AUTO => Self::Auto,
            vvencProfile_VVENC_MAIN_10 => Self::Main10,
            vvencProfile_VVENC_MAIN_10_STILL_PICTURE => Self::Main10StillPicture,
            vvencProfile_VVENC_MAIN_10_444 => Self::Main10444,
            vvencProfile_VVENC_MAIN_10_444_STILL_PICTURE => Self::Main10444StillPicture,
            vvencProfile_VVENC_MULTILAYER_MAIN_10 => Self::MultilayerMain10,
            vvencProfile_VVENC_MULTILAYER_MAIN_10_STILL_PICTURE => {
                Self::MultilayerMain10StillPicture
            }
            vvencProfile_VVENC_MULTILAYER_MAIN_10_444 => Self::MultilayerMain10444,
            vvencProfile_VVENC_MULTILAYER_MAIN_10_444_STILL_PICTURE => {
                Self::MultilayerMain10444StillPicture
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Main,
    High,
}

impl Tier {
    #[inline]
    fn to_ffi(self) -> vvencTier {
        match self {
            Self::Main => vvencTier_VVENC_TIER_MAIN,
            Self::High => vvencTier_VVENC_TIER_HIGH,
        }
    }

    #[inline]
    fn from_ffi(value: vvencTier) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencTier_VVENC_TIER_MAIN => Self::Main,
            vvencTier_VVENC_TIER_HIGH => Self::High,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Level {
    #[inline]
    fn to_ffi(self) -> vvencLevel {
        match self {
            Self::Auto => vvencLevel_VVENC_LEVEL_AUTO,
            Self::Level1 => vvencLevel_VVENC_LEVEL1,
            Self::Level2 => vvencLevel_VVENC_LEVEL2,
            Self::Level2_1 => vvencLevel_VVENC_LEVEL2_1,
            Self::Level3 => vvencLevel_VVENC_LEVEL3,
            Self::Level3_1 => vvencLevel_VVENC_LEVEL3_1,
            Self::Level4 => vvencLevel_VVENC_LEVEL4,
            Self::Level4_1 => vvencLevel_VVENC_LEVEL4_1,
            Self::Level5 => vvencLevel_VVENC_LEVEL5,
            Self::Level5_1 => vvencLevel_VVENC_LEVEL5_1,
            Self::Level5_2 => vvencLevel_VVENC_LEVEL5_2,
            Self::Level6 => vvencLevel_VVENC_LEVEL6,
            Self::Level6_1 => vvencLevel_VVENC_LEVEL6_1,
            Self::Level6_2 => vvencLevel_VVENC_LEVEL6_2,
            Self::Level6_3 => vvencLevel_VVENC_LEVEL6_3,
            Self::Level15_5 => vvencLevel_VVENC_LEVEL15_5,
        }
    }

    #[inline]
    fn from_ffi(value: vvencLevel) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencLevel_VVENC_LEVEL_AUTO => Self::Auto,
            vvencLevel_VVENC_LEVEL1 => Self::Level1,
            vvencLevel_VVENC_LEVEL2 => Self::Level2,
            vvencLevel_VVENC_LEVEL2_1 => Self::Level2_1,
            vvencLevel_VVENC_LEVEL3 => Self::Level3,
            vvencLevel_VVENC_LEVEL3_1 => Self::Level3_1,
            vvencLevel_VVENC_LEVEL4 => Self::Level4,
            vvencLevel_VVENC_LEVEL4_1 => Self::Level4_1,
            vvencLevel_VVENC_LEVEL5 => Self::Level5,
            vvencLevel_VVENC_LEVEL5_1 => Self::Level5_1,
            vvencLevel_VVENC_LEVEL5_2 => Self::Level5_2,
            vvencLevel_VVENC_LEVEL6 => Self::Level6,
            vvencLevel_VVENC_LEVEL6_1 => Self::Level6_1,
            vvencLevel_VVENC_LEVEL6_2 => Self::Level6_2,
            vvencLevel_VVENC_LEVEL6_3 => Self::Level6_3,
            vvencLevel_VVENC_LEVEL15_5 => Self::Level15_5,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodingRefreshType {
    None,
    Cra,
    Idr,
    RecoveryPointSei,
    Idr2,
    CraCre,
    IdrNoRadl,
}

impl DecodingRefreshType {
    #[inline]
    fn to_ffi(self) -> vvencDecodingRefreshType {
        match self {
            Self::None => vvencDecodingRefreshType_VVENC_DRT_NONE,
            Self::Cra => vvencDecodingRefreshType_VVENC_DRT_CRA,
            Self::Idr => vvencDecodingRefreshType_VVENC_DRT_IDR,
            Self::RecoveryPointSei => vvencDecodingRefreshType_VVENC_DRT_RECOVERY_POINT_SEI,
            Self::Idr2 => vvencDecodingRefreshType_VVENC_DRT_IDR2,
            Self::CraCre => vvencDecodingRefreshType_VVENC_DRT_CRA_CRE,
            Self::IdrNoRadl => vvencDecodingRefreshType_VVENC_DRT_IDR_NO_RADL,
        }
    }

    #[inline]
    fn from_ffi(value: vvencDecodingRefreshType) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencDecodingRefreshType_VVENC_DRT_NONE => Self::None,
            vvencDecodingRefreshType_VVENC_DRT_CRA => Self::Cra,
            vvencDecodingRefreshType_VVENC_DRT_IDR => Self::Idr,
            vvencDecodingRefreshType_VVENC_DRT_RECOVERY_POINT_SEI => Self::RecoveryPointSei,
            vvencDecodingRefreshType_VVENC_DRT_IDR2 => Self::Idr2,
            vvencDecodingRefreshType_VVENC_DRT_CRA_CRE => Self::CraCre,
            vvencDecodingRefreshType_VVENC_DRT_IDR_NO_RADL => Self::IdrNoRadl,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMode {
    Off,
    First,
    Mid,
    Last,
}

impl SegmentMode {
    #[inline]
    fn to_ffi(self) -> vvencSegmentMode {
        match self {
            Self::Off => vvencSegmentMode_VVENC_SEG_OFF,
            Self::First => vvencSegmentMode_VVENC_SEG_FIRST,
            Self::Mid => vvencSegmentMode_VVENC_SEG_MID,
            Self::Last => vvencSegmentMode_VVENC_SEG_LAST,
        }
    }

    #[inline]
    fn from_ffi(value: vvencSegmentMode) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencSegmentMode_VVENC_SEG_OFF => Self::Off,
            vvencSegmentMode_VVENC_SEG_FIRST => Self::First,
            vvencSegmentMode_VVENC_SEG_MID => Self::Mid,
            vvencSegmentMode_VVENC_SEG_LAST => Self::Last,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrMode {
    Off,
    Pq,
    Hlg,
    PqBt2020,
    HlgBt2020,
    UserDefined,
    SdrBt709,
    SdrBt2020,
    SdrBt470bg,
}

impl HdrMode {
    #[inline]
    fn to_ffi(self) -> vvencHDRMode {
        match self {
            Self::Off => vvencHDRMode_VVENC_HDR_OFF,
            Self::Pq => vvencHDRMode_VVENC_HDR_PQ,
            Self::Hlg => vvencHDRMode_VVENC_HDR_HLG,
            Self::PqBt2020 => vvencHDRMode_VVENC_HDR_PQ_BT2020,
            Self::HlgBt2020 => vvencHDRMode_VVENC_HDR_HLG_BT2020,
            Self::UserDefined => vvencHDRMode_VVENC_HDR_USER_DEFINED,
            Self::SdrBt709 => vvencHDRMode_VVENC_SDR_BT709,
            Self::SdrBt2020 => vvencHDRMode_VVENC_SDR_BT2020,
            Self::SdrBt470bg => vvencHDRMode_VVENC_SDR_BT470BG,
        }
    }

    #[inline]
    fn from_ffi(value: vvencHDRMode) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencHDRMode_VVENC_HDR_OFF => Self::Off,
            vvencHDRMode_VVENC_HDR_PQ => Self::Pq,
            vvencHDRMode_VVENC_HDR_HLG => Self::Hlg,
            vvencHDRMode_VVENC_HDR_PQ_BT2020 => Self::PqBt2020,
            vvencHDRMode_VVENC_HDR_HLG_BT2020 => Self::HlgBt2020,
            vvencHDRMode_VVENC_HDR_USER_DEFINED => Self::UserDefined,
            vvencHDRMode_VVENC_SDR_BT709 => Self::SdrBt709,
            vvencHDRMode_VVENC_SDR_BT2020 => Self::SdrBt2020,
            vvencHDRMode_VVENC_SDR_BT470BG => Self::SdrBt470bg,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    inner: vvenc_config,
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

impl Default for Config {
    fn default() -> Self {
        let mut inner = unsafe { std::mem::zeroed() };
        unsafe {
            vvenc_config_default(&mut inner);
        }
        Self { inner }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(&self) -> i32 {
        self.inner.m_SourceWidth
    }

    pub fn set_width(&mut self, width: i32) -> &mut Self {
        self.inner.m_SourceWidth = width;
        self
    }

    pub fn height(&self) -> i32 {
        self.inner.m_SourceHeight
    }

    pub fn set_height(&mut self, height: i32) -> &mut Self {
        self.inner.m_SourceHeight = height;
        self
    }

    pub fn framerate(&self) -> Rational {
        Rational {
            num: self.inner.m_FrameRate,
            den: self.inner.m_FrameScale,
        }
    }

    pub fn set_framerate(&mut self, framerate: Rational) -> &mut Self {
        self.inner.m_FrameRate = framerate.num;
        self.inner.m_FrameScale = framerate.den;
        self
    }

    pub fn set_preset(&mut self, preset: Preset) -> Result<&mut Self, Error> {
        let ret = unsafe { vvenc_init_preset(&mut self.inner, preset.to_ffi()) };
        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }
        Ok(self)
    }

    pub fn ticks_per_second(&self) -> i32 {
        self.inner.m_TicksPerSecond
    }

    pub fn set_ticks_per_second(&mut self, ticks_per_second: i32) -> &mut Self {
        self.inner.m_TicksPerSecond = ticks_per_second;
        self
    }

    pub fn frames_to_be_encoded(&self) -> i32 {
        self.inner.m_framesToBeEncoded
    }

    pub fn set_frames_to_be_encoded(&mut self, frames_to_be_encoded: i32) -> &mut Self {
        self.inner.m_framesToBeEncoded = frames_to_be_encoded;
        self
    }

    pub fn input_bit_depth(&self) -> [i32; 2] {
        self.inner.m_inputBitDepth
    }

    pub fn set_input_bit_depth(&mut self, input_bit_depth: [i32; 2]) -> &mut Self {
        self.inner.m_inputBitDepth = input_bit_depth;
        self
    }

    pub fn num_threads(&self) -> i32 {
        self.inner.m_numThreads
    }

    pub fn set_num_threads(&mut self, num_threads: i32) -> &mut Self {
        self.inner.m_numThreads = num_threads;
        self
    }

    pub fn qp(&self) -> Qp {
        Qp::from_ffi(self.inner.m_QP).unwrap()
    }

    pub fn set_qp(&mut self, qp: Qp) -> &mut Self {
        self.inner.m_QP = qp.to_ffi();
        self
    }

    pub fn target_bitrate(&self) -> i32 {
        self.inner.m_RCTargetBitrate
    }

    pub fn set_target_bitrate(&mut self, target_bitrate: i32) -> &mut Self {
        self.inner.m_RCTargetBitrate = target_bitrate;
        self
    }

    pub fn profile(&self) -> Profile {
        Profile::from_ffi(self.inner.m_profile)
    }

    pub fn set_profile(&mut self, profile: Profile) -> &mut Self {
        self.inner.m_profile = profile.to_ffi();
        self
    }

    pub fn tier(&self) -> Tier {
        Tier::from_ffi(self.inner.m_levelTier)
    }

    pub fn set_tier(&mut self, tier: Tier) -> &mut Self {
        self.inner.m_levelTier = tier.to_ffi();
        self
    }

    pub fn level(&self) -> Level {
        Level::from_ffi(self.inner.m_level)
    }

    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.inner.m_level = level.to_ffi();
        self
    }

    pub fn intra_period(&self) -> i32 {
        self.inner.m_IntraPeriod
    }

    pub fn set_intra_period(&mut self, intra_period: i32) -> &mut Self {
        self.inner.m_IntraPeriod = intra_period;
        self
    }

    pub fn intra_period_seconds(&self) -> i32 {
        self.inner.m_IntraPeriodSec
    }

    pub fn set_intra_period_seconds(&mut self, intra_period_seconds: i32) -> &mut Self {
        self.inner.m_IntraPeriodSec = intra_period_seconds;
        self
    }

    pub fn decoding_refresh_type(&self) -> DecodingRefreshType {
        DecodingRefreshType::from_ffi(self.inner.m_DecodingRefreshType)
    }

    pub fn set_decoding_refresh_type(
        &mut self,
        decoding_refresh_type: DecodingRefreshType,
    ) -> &mut Self {
        self.inner.m_DecodingRefreshType = decoding_refresh_type.to_ffi();
        self
    }

    pub fn gop_size(&self) -> i32 {
        self.inner.m_GOPSize
    }

    pub fn set_gop_size(&mut self, gop_size: i32) -> &mut Self {
        self.inner.m_GOPSize = gop_size;
        self
    }

    pub fn num_passes(&self) -> i32 {
        self.inner.m_RCNumPasses
    }

    pub fn set_num_passes(&mut self, num_passes: i32) -> &mut Self {
        self.inner.m_RCNumPasses = num_passes;
        self
    }

    pub fn pass(&self) -> i32 {
        self.inner.m_RCPass
    }

    pub fn set_pass(&mut self, pass: i32) -> &mut Self {
        self.inner.m_RCPass = pass;
        self
    }

    pub fn internal_bit_depth(&self) -> [i32; 2] {
        self.inner.m_internalBitDepth
    }

    pub fn set_internal_bit_depth(&mut self, internal_bit_depth: [i32; 2]) -> &mut Self {
        self.inner.m_internalBitDepth = internal_bit_depth;
        self
    }

    pub fn hdr_mode(&self) -> HdrMode {
        HdrMode::from_ffi(self.inner.m_HdrMode)
    }

    pub fn set_hdr_mode(&mut self, hdr_mode: HdrMode) -> &mut Self {
        self.inner.m_HdrMode = hdr_mode.to_ffi();
        self
    }

    pub fn segment_mode(&self) -> SegmentMode {
        SegmentMode::from_ffi(self.inner.m_SegmentMode)
    }

    pub fn set_segment_mode(&mut self, segment_mode: SegmentMode) -> &mut Self {
        self.inner.m_SegmentMode = segment_mode.to_ffi();
        self
    }

    pub fn use_percept_qpa(&self) -> bool {
        self.inner.m_usePerceptQPA
    }

    pub fn set_use_percept_qpa(&mut self, use_percept_qpa: bool) -> &mut Self {
        self.inner.m_usePerceptQPA = use_percept_qpa;
        self
    }

    pub fn num_tile_columns(&self) -> i32 {
        self.inner.m_numTileCols
    }

    pub fn set_num_tile_columns(&mut self, num_tile_columns: i32) -> &mut Self {
        self.inner.m_numTileCols = num_tile_columns;
        self
    }

    pub fn num_tile_rows(&self) -> i32 {
        self.inner.m_numTileRows
    }

    pub fn set_num_tile_rows(&mut self, num_tile_rows: i32) -> &mut Self {
        self.inner.m_numTileRows = num_tile_rows;
        self
    }

    pub fn internal_chroma_format(&self) -> ChromaFormat {
        ChromaFormat::from_ffi(self.inner.m_internChromaFormat)
    }

    pub fn set_internal_chroma_format(
        &mut self,
        internal_chroma_format: ChromaFormat,
    ) -> &mut Self {
        self.inner.m_internChromaFormat = internal_chroma_format.to_ffi();
        self
    }

    pub fn log_level(&self) -> LogLevel {
        LogLevel::from_ffi(self.inner.m_verbosity)
    }

    pub fn set_log_level(&mut self, log_level: LogLevel) -> &mut Self {
        self.inner.m_verbosity = log_level.to_ffi();
        self
    }

    pub fn set_logger(&mut self, logger: Box<dyn Logger>) -> &mut Self {
        let logger = Box::new(logger);
        unsafe {
            vvenc_set_msg_callback(
                &mut self.inner,
                Box::into_raw(logger) as *mut c_void,
                Some(log_callback),
            )
        };
        self
    }
}

unsafe extern "C" fn log_callback(
    ctx: *mut ::std::os::raw::c_void,
    level: ::std::os::raw::c_int,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) {
    let logger = &*(ctx as *mut Box<dyn Logger>);
    let level = LogLevel::from_ffi(level as u32);
    let message = vsprintf(fmt, args).unwrap();
    logger.log(level, &message);
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("unspecified error")]
    Unspecified,
    #[error("failed to initialize encoder")]
    Initialize,
    #[error("failed to allocate resources")]
    Allocate,
    #[error("not enough memory")]
    NotEnoughMemory,
    #[error("invalid parameter")]
    Parameter,
    #[error("operation not supported")]
    NotSupported,
    #[error("restart required")]
    RestartRequired,
    #[error("CPU error")]
    Cpu,
    #[error("unknown error with code {0}")]
    Unknown(i32),
}

impl Error {
    fn new(code: ErrorCodes) -> Self {
        #[allow(non_upper_case_globals)]
        match code {
            ErrorCodes_VVENC_ERR_UNSPECIFIED => Error::Unspecified,
            ErrorCodes_VVENC_ERR_INITIALIZE => Error::Initialize,
            ErrorCodes_VVENC_ERR_ALLOCATE => Error::Allocate,
            ErrorCodes_VVENC_NOT_ENOUGH_MEM => Error::NotEnoughMemory,
            ErrorCodes_VVENC_ERR_PARAMETER => Error::Parameter,
            ErrorCodes_VVENC_ERR_NOT_SUPPORTED => Error::NotSupported,
            ErrorCodes_VVENC_ERR_RESTART_REQUIRED => Error::RestartRequired,
            ErrorCodes_VVENC_ERR_CPU => Error::Cpu,
            code => Error::Unknown(code),
        }
    }
}

#[derive(Debug, Clone)]
pub struct YUVBuffer<Opaque> {
    inner: vvencYUVBuffer,
    _phantom: std::marker::PhantomData<Opaque>,
}

// TODO: check if this is safe wrt to inner
unsafe impl<Opaque> Send for YUVBuffer<Opaque> {}
unsafe impl<Opaque> Sync for YUVBuffer<Opaque> {}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum YUVComponent {
    Y = 0,
    U = 1,
    V = 2,
}

impl<Opaque: Sized + Send + Sync> YUVBuffer<Opaque> {
    pub fn new(width: i32, height: i32, chroma_format: ChromaFormat) -> Self {
        let mut inner = unsafe {
            let mut inner = std::mem::zeroed();
            vvenc_YUVBuffer_alloc_buffer(&mut inner, chroma_format.to_ffi(), width, height);
            inner
        };
        inner.opaque = ptr::null_mut();
        Self {
            inner,
            _phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn plane_mut<'a>(&'a mut self, component: YUVComponent) -> Plane<'a> {
        Plane {
            inner: self.inner.planes[component as usize],
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn sequence_number(&self) -> u64 {
        self.inner.sequenceNumber
    }

    pub fn set_sequence_number(&mut self, sequence_number: u64) {
        self.inner.sequenceNumber = sequence_number;
    }

    pub fn cts(&self) -> Option<u64> {
        self.inner.ctsValid.then_some(self.inner.cts)
    }

    pub fn set_cts(&mut self, cts: u64) {
        self.inner.cts = cts;
        self.inner.ctsValid = true;
    }

    pub fn set_opaque(&mut self, opaque: Opaque) {
        self.inner.opaque = Box::into_raw(Box::new(opaque)) as *mut c_void;
    }
}

impl<Opaque> Drop for YUVBuffer<Opaque> {
    fn drop(&mut self) {
        unsafe {
            vvenc_YUVBuffer_free_buffer(&mut self.inner);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plane<'a> {
    inner: vvencYUVPlane,
    phantom: std::marker::PhantomData<&'a [i16]>,
}

impl<'a> Plane<'a> {
    pub fn data(&mut self) -> &[i16] {
        unsafe {
            std::slice::from_raw_parts(
                self.inner.ptr,
                self.inner.height as usize * self.inner.stride as usize,
            )
        }
    }

    pub fn data_mut(&mut self) -> &mut [i16] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.inner.ptr,
                self.inner.height as usize * self.inner.stride as usize,
            )
        }
    }

    pub fn width(&self) -> i32 {
        self.inner.width
    }

    pub fn height(&self) -> i32 {
        self.inner.height
    }

    pub fn stride(&self) -> i32 {
        self.inner.stride
    }
}

#[derive(Debug)]
pub struct AccessUnit<'a, Opaque> {
    inner: vvencAccessUnit,
    data: &'a [u8],
    _phantom: std::marker::PhantomData<Opaque>,
}

impl<'a, Opaque: Sized + Sync + Send> AccessUnit<'a, Opaque> {
    fn new(data: &'a mut [u8]) -> Self {
        let inner = unsafe {
            let mut inner = std::mem::zeroed();
            vvenc_accessUnit_default(&mut inner);
            inner.payload = data.as_mut_ptr();
            inner.payloadSize = data.len() as i32;
            inner.payloadUsedSize = 0;
            inner
        };
        Self {
            inner,
            data,
            _phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[..self.inner.payloadUsedSize as usize]
    }

    pub fn cts(&self) -> Option<u64> {
        self.inner.ctsValid.then_some(self.inner.cts)
    }

    pub fn dts(&self) -> Option<u64> {
        self.inner.dtsValid.then_some(self.inner.dts)
    }

    pub fn rap(&self) -> bool {
        self.inner.rap
    }

    pub fn slice_type(&self) -> SliceType {
        SliceType::from_ffi(self.inner.sliceType)
    }

    pub fn is_ref_pic(&self) -> bool {
        self.inner.refPic
    }

    pub fn temporal_layer(&self) -> i32 {
        self.inner.temporalLayer
    }

    pub fn poc(&self) -> u64 {
        self.inner.poc
    }

    pub fn take_opaque(&mut self) -> Box<Opaque> {
        let raw = self.inner.opaque;
        self.inner.opaque = ptr::null_mut();
        // SAFETY: AccessUnit is only created from an Encoder with Opaque type, forcing the corresponding
        // input YUV buffer to have Opaque types as well.
        unsafe { Box::from_raw(raw as *mut Opaque) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    MediumLowDecNrg,
    FirstPass,
    ToolTest,
    Unknown(i32),
}

impl Preset {
    #[inline]
    fn to_ffi(self) -> vvencPresetMode {
        match self {
            Self::Faster => vvencPresetMode_VVENC_FASTER,
            Self::Fast => vvencPresetMode_VVENC_FAST,
            Self::Medium => vvencPresetMode_VVENC_MEDIUM,
            Self::Slow => vvencPresetMode_VVENC_SLOW,
            Self::Slower => vvencPresetMode_VVENC_SLOWER,
            Self::MediumLowDecNrg => vvencPresetMode_VVENC_MEDIUM_LOWDECNRG,
            Self::FirstPass => vvencPresetMode_VVENC_FIRSTPASS,
            Self::ToolTest => vvencPresetMode_VVENC_TOOLTEST,
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChromaFormat {
    Chroma400,
    Chroma420,
    Chroma422,
    Chroma444,
    Unknown(u32),
}

impl ChromaFormat {
    #[inline]
    fn to_ffi(self) -> vvencChromaFormat {
        match self {
            Self::Chroma400 => vvencChromaFormat_VVENC_CHROMA_400,
            Self::Chroma420 => vvencChromaFormat_VVENC_CHROMA_420,
            Self::Chroma422 => vvencChromaFormat_VVENC_CHROMA_422,
            Self::Chroma444 => vvencChromaFormat_VVENC_CHROMA_444,
            Self::Unknown(value) => value,
        }
    }

    #[inline]
    fn from_ffi(value: vvencChromaFormat) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencChromaFormat_VVENC_CHROMA_400 => Self::Chroma400,
            vvencChromaFormat_VVENC_CHROMA_420 => Self::Chroma420,
            vvencChromaFormat_VVENC_CHROMA_422 => Self::Chroma422,
            vvencChromaFormat_VVENC_CHROMA_444 => Self::Chroma444,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceType {
    B,
    P,
    I,
    Unknown(u32),
}

impl SliceType {
    #[inline]
    fn from_ffi(value: vvencSliceType) -> Self {
        #[allow(non_upper_case_globals)]
        match value {
            vvencSliceType_VVENC_B_SLICE => Self::B,
            vvencSliceType_VVENC_P_SLICE => Self::P,
            vvencSliceType_VVENC_I_SLICE => Self::I,
            _ => Self::Unknown(value),
        }
    }
}
