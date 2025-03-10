use std::{
    ffi::c_void,
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
    pub fn with_config(config: &mut Config) -> Result<Self, Error> {
        let Some(encoder) = ptr::NonNull::new(unsafe { vvenc_encoder_create() }) else {
            return Err(Error::Initialize);
        };
        let ret = unsafe { vvenc_encoder_open(encoder.as_ptr(), &mut config.to_ffi()?) };
        #[allow(non_upper_case_globals)]
        match ret {
            ErrorCodes_VVENC_OK => Ok(Self {
                inner: Arc::new(Mutex::new(InnerEncoder { encoder })),
                _phantom: std::marker::PhantomData::default(),
            }),
            _ => Err(Error::new(ret)),
        }
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

    pub fn reconfigure(&mut self, config: &mut Config) -> Result<(), Error> {
        let ret = unsafe {
            vvenc_reconfig(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut config.to_ffi()?,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Rational {
    pub num: i32,
    pub den: i32,
}

#[derive(Debug)]
pub struct Qp(u8);

impl Qp {
    pub fn new(value: u8) -> Result<Self, Error> {
        if value > 63 {
            return Err(Error::Parameter);
        }
        Ok(Self(value))
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
    fn log_level(&self) -> LogLevel;
    fn log(&self, level: LogLevel, message: &str);
}

pub struct Config {
    pub width: i32,
    pub height: i32,
    pub framerate: Rational,
    pub qp: Qp,
    pub chroma_format: ChromaFormat,
    pub preset: Preset,
    pub logger: Option<Box<dyn Logger>>,
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

impl Config {
    fn to_ffi(&mut self) -> Result<vvenc_config, Error> {
        let mut vvenc_cfg = unsafe { std::mem::zeroed() };

        let ret = unsafe {
            vvenc_init_default(
                &mut vvenc_cfg,
                self.width,
                self.height,
                // vvenc_init_default handles fractional framerates by manually
                // checking for values of 23, 29, 59 and 119... sigh
                (self.framerate.num / self.framerate.den) as i32,
                VVENC_RC_OFF as i32,
                self.qp.0 as i32,
                self.preset.to_ffi(),
            )
        };
        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }
        vvenc_cfg.m_internChromaFormat = self.chroma_format.to_ffi();
        if let Some(logger) = self.logger.take() {
            vvenc_cfg.m_verbosity = logger.log_level().to_ffi();
            let logger = Box::new(logger);
            unsafe {
                vvenc_set_msg_callback(
                    &mut vvenc_cfg,
                    Box::into_raw(logger) as *mut c_void,
                    Some(log_callback),
                )
            };
        }

        Ok(vvenc_cfg)
    }
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
