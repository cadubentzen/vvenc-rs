use std::{
    ptr,
    sync::{Arc, Mutex},
};

use vvenc_sys::*;

#[derive(Debug)]
pub struct Encoder {
    inner: Arc<Mutex<InnerEncoder>>,
}

unsafe impl Sync for Encoder {}
unsafe impl Send for Encoder {}

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
            }),
            _ => Err(Error::new(ret)),
        }
    }

    pub fn encode<'a, 'b>(
        &mut self,
        frame: Frame<'a>,
        out_data: &'b mut [u8],
    ) -> Result<Option<AccessUnit<'b>>, Error> {
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

        let mut au = AccessUnit::new(out_data);
        let mut encode_done = false;
        let ret = unsafe {
            vvenc_encode(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut yuv_buffer,
                &mut au.inner,
                &mut encode_done,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok((!au.payload().is_empty()).then_some(au))
    }

    pub fn flush<'b>(&mut self, out_data: &'b mut [u8]) -> Result<Option<AccessUnit<'b>>, Error> {
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

        Ok((!au.payload().is_empty()).then_some(au))
    }

    pub fn config(&self) -> Result<Config, Error> {
        let mut inner = unsafe { std::mem::zeroed() };
        let ret =
            unsafe { vvenc_get_config(self.inner.lock().unwrap().encoder.as_ptr(), &mut inner) };
        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }
        Ok(Config { inner })
    }

    pub fn reconfigure(&mut self, mut config: Config) -> Result<(), Error> {
        let ret = unsafe {
            vvenc_reconfig(
                self.inner.lock().unwrap().encoder.as_ptr(),
                &mut config.inner,
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok(())
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
    ) -> Result<Self, Error> {
        let mut inner = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            vvenc_init_default(
                &mut inner,
                width,
                height,
                framerate,
                target_bitrate,
                qp,
                preset.to_ffi(),
            )
        };

        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }

        Ok(Self { inner })
    }

    pub fn source_width(&self) -> i32 {
        self.inner.m_SourceWidth
    }

    pub fn source_height(&self) -> i32 {
        self.inner.m_SourceHeight
    }

    pub fn chroma_format(&self) -> ChromaFormat {
        ChromaFormat::from_ffi(self.inner.m_internChromaFormat)
    }

    pub fn set_chroma_format(&mut self, chroma_format: ChromaFormat) {
        self.inner.m_internChromaFormat = chroma_format.to_ffi();
    }

    pub fn set_preset(&mut self, preset: Preset) -> Result<(), Error> {
        let ret = unsafe { vvenc_init_preset(&mut self.inner, preset.to_ffi()) };
        if ret != ErrorCodes_VVENC_OK {
            return Err(Error::new(ret));
        }
        Ok(())
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
pub struct Frame<'a> {
    pub planes: [Plane<'a>; 3],
    pub sequence_number: u64,
    pub cts: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Plane<'a> {
    pub data: &'a [i16],
    pub width: i32,
    pub height: i32,
    pub stride: i32,
}

#[derive(Debug)]
pub struct AccessUnit<'a> {
    inner: vvencAccessUnit,
    data: &'a [u8],
}

impl<'a> AccessUnit<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        let inner = unsafe {
            let mut inner = std::mem::zeroed();
            vvenc_accessUnit_default(&mut inner);
            inner.payload = data.as_mut_ptr();
            inner.payloadSize = data.len() as i32;
            inner.payloadUsedSize = 0;
            inner
        };
        Self { inner, data }
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
