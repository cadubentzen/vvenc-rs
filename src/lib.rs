use std::{
    ptr,
    sync::{Arc, Mutex},
};

use vvenc_sys::*;

#[derive(Debug, Clone)]
pub struct Encoder {
    inner: Arc<Mutex<InnerEncoder>>,
}

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

impl Encoder {
    pub fn new(mut config: Config) -> Result<Self, Error> {
        let Some(encoder) = ptr::NonNull::new(unsafe { vvenc_encoder_create() }) else {
            return Err(Error::Initialize);
        };
        let ret = unsafe { vvenc_encoder_open(encoder.as_ptr(), &mut config.inner) };
        match ret {
            ErrorCodes_VVENC_OK => Ok(Self {
                inner: Arc::new(Mutex::new(InnerEncoder { encoder })),
            }),
            _ => Err(Error::new(ret)),
        }
    }

    fn with_config(config: &mut vvenc_config) -> Result<Self, Error> {
        let Some(encoder) = ptr::NonNull::new(unsafe { vvenc_encoder_create() }) else {
            return Err(Error::Initialize);
        };
        let ret = unsafe { vvenc_encoder_open(encoder.as_ptr(), config) };
        match ret {
            ErrorCodes_VVENC_OK => Ok(Self {
                inner: Arc::new(Mutex::new(InnerEncoder { encoder })),
            }),
            _ => Err(Error::new(ret)),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    inner: vvenc_config,
}

impl Config {
    pub fn new(
        width: i32,
        height: i32,
        framerate: i32,
        target_bitrate: i32,
        qp: i32,
        preset: Preset,
    ) -> Self {
        unsafe {
            let mut inner = std::mem::zeroed();
            vvenc_init_default(
                &mut inner,
                width,
                height,
                framerate,
                target_bitrate,
                qp,
                preset.to_ffi(),
            );
            Self { inner }
        }
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
        }
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
