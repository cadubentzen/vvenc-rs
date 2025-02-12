use std::{
    ptr,
    sync::{Arc, Mutex},
};

use vvenc_sys::*;

#[derive(Debug)]
pub struct Encoder {
    inner: Arc<Mutex<InnerEncoder>>,
    au: AccessUnit,
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
        #[allow(non_upper_case_globals)]
        match ret {
            ErrorCodes_VVENC_OK => Ok(Self {
                inner: Arc::new(Mutex::new(InnerEncoder { encoder })),
                au: AccessUnit::new(&config),
            }),
            _ => Err(Error::new(ret)),
        }
    }

    pub fn encode<'a>(&mut self, frame: Frame<'a>) -> Result<Option<&AccessUnit>, Error> {
        let mut yuv_buffer = vvencYUVBuffer {
            planes: [
                vvencYUVPlane {
                    ptr: frame.planes[0].data.as_ptr() as *mut i16,
                    width: frame.planes[0].width,
                    height: frame.planes[0].height,
                    stride: frame.planes[0].stride,
                },
                vvencYUVPlane {
                    ptr: frame.planes[1].data.as_ptr() as *mut i16,
                    width: frame.planes[1].width,
                    height: frame.planes[1].height,
                    stride: frame.planes[1].stride,
                },
                vvencYUVPlane {
                    ptr: frame.planes[2].data.as_ptr() as *mut i16,
                    width: frame.planes[2].width,
                    height: frame.planes[2].height,
                    stride: frame.planes[2].stride,
                },
            ],
            sequenceNumber: frame.sequence_number,
            cts: frame.cts.unwrap_or(0),
            ctsValid: frame.cts.is_some(),
        };

        let mut encode_done = false;
        let ret = unsafe {
            vvenc_encode(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut yuv_buffer,
                &mut self.au.inner,
                &mut encode_done,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        // TODO: double check if encode_done handling is correct or if we lose a frame like that
        if self.au.inner.payloadUsedSize == 0 {
            return Ok(None);
        }

        Ok(Some(&self.au))
    }

    pub fn flush(&mut self) -> Result<Option<&AccessUnit>, Error> {
        let mut encode_done = false;
        let ret = unsafe {
            vvenc_encode(
                self.inner.lock().unwrap().encoder.as_ptr(),
                std::ptr::null_mut(),
                &mut self.au.inner,
                &mut encode_done,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        if self.au.inner.payloadUsedSize == 0 {
            return Ok(None);
        }

        Ok(Some(&self.au))
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

#[derive(Debug)]
pub struct Frame<'a> {
    pub planes: [Plane<'a>; 3],
    pub sequence_number: u64,
    pub cts: Option<u64>,
}

#[derive(Debug)]
pub struct Plane<'a> {
    pub data: &'a [i16],
    pub width: i32,
    pub height: i32,
    pub stride: i32,
}

#[derive(Debug)]
pub struct AccessUnit {
    // FIXME: make inner private
    pub inner: vvencAccessUnit,
}

impl AccessUnit {
    fn new(config: &Config) -> Self {
        // Allocate enough space for the AU payloads from the given config. Same as in EncApp.cpp from libvvenc
        #[allow(non_upper_case_globals)]
        let au_size_scale = match config.inner.m_internChromaFormat {
            vvencChromaFormat_VVENC_CHROMA_400 | vvencChromaFormat_VVENC_CHROMA_420 => 2,
            _ => 3,
        };
        let payload_size =
            au_size_scale * config.inner.m_SourceHeight * config.inner.m_SourceWidth + 1024;
        let inner = unsafe {
            let mut inner = std::mem::zeroed();
            vvenc_accessUnit_default(&mut inner);
            vvenc_accessUnit_alloc_payload(&mut inner, payload_size);
            inner
        };
        Self { inner }
    }
}

impl Drop for AccessUnit {
    fn drop(&mut self) {
        unsafe {
            vvenc_accessUnit_free_payload(&mut self.inner);
        }
    }
}
