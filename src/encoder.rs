use std::mem::MaybeUninit;

use crate::*;

pub struct Encoder {
    ffi_encoder: *mut ffi::vvencEncoder,
    #[allow(unused)] // It's used indirectly through unsafe
    output_buffer: Vec<u8>,
    ffi_access_unit: ffi::vvencAccessUnit,
}

impl Encoder {
    pub fn with_config(mut config: ConfigBuilder) -> Result<Self, Error> {
        let ffi_encoder = unsafe { ffi::vvenc_encoder_create() };
        unsafe { ffi::vvenc_encoder_open(ffi_encoder, &mut config.ffi_config) }.to_result()?;
        // We should drop the config builder at this point. To be able to access the config,
        // we should query it from the encoder.
        drop(config);

        let config = Self::ffi_get_config(ffi_encoder)?;

        let au_size_scale = get_access_unit_size_scale(&config);
        let output_buffer_size =
            (au_size_scale * config.source_width() * config.source_height() + 1024) as usize;

        let mut output_buffer = vec![0; output_buffer_size];

        let mut ffi_access_unit = MaybeUninit::uninit();
        unsafe {
            ffi::vvenc_accessUnit_default(ffi_access_unit.as_mut_ptr());
        }
        // SAFETY: vvenc_accessUnit_default initialized the access unit
        let mut ffi_access_unit = unsafe { ffi_access_unit.assume_init() };
        ffi_access_unit.payload = output_buffer.as_mut_ptr();
        ffi_access_unit.payloadSize = output_buffer.len() as i32;

        Ok(Self {
            ffi_encoder,
            output_buffer,
            ffi_access_unit,
        })
    }

    pub fn encode<'a>(
        &mut self,
        yuv_buffer: Option<&'a YUVBuffer>,
    ) -> Result<EncoderOutput<'a>, Error> {
        let mut enc_done = false;
        unsafe {
            ffi::vvenc_encode(
                self.ffi_encoder,
                yuv_buffer.map_or(std::ptr::null_mut(), |y| &mut y.to_ffi()),
                &mut self.ffi_access_unit,
                &mut enc_done,
            )
        }
        .to_result()?;

        if self.ffi_access_unit.payloadUsedSize > 0 {
            let data = self.get_encoder_output_data(yuv_buffer);
            Ok(EncoderOutput::Data(data, enc_done))
        } else {
            Ok(EncoderOutput::None)
        }
    }

    fn get_encoder_output_data<'a>(
        &self,
        _yuv_buffer: Option<&'a YUVBuffer>,
    ) -> EncoderOutputData<'a> {
        EncoderOutputData {
            // SAFETY: the lifetime of the output data is tied to the input buffer,
            // therefore we are not able to access it anymore when the next input is pushed
            payload: unsafe {
                std::slice::from_raw_parts(
                    self.ffi_access_unit.payload,
                    self.ffi_access_unit.payloadUsedSize as usize,
                )
            },
            pts: if self.ffi_access_unit.ctsValid {
                Some(self.ffi_access_unit.cts)
            } else {
                None
            },
            dts: if self.ffi_access_unit.dtsValid {
                Some(self.ffi_access_unit.dts)
            } else {
                None
            },
            is_random_access_point: self.ffi_access_unit.rap,
            slice_type: match self
                .ffi_access_unit
                .sliceType
                .try_into() {
                    Ok(slice_type) => slice_type,
                    Err(ffi_slice_type) => panic!("slice type {} from libvvenc not enumerated in this crate. Please open an issue.", ffi_slice_type)
                },
            temporal_layer: self.ffi_access_unit.temporalLayer,
        }
    }

    pub fn get_config(&self) -> Config {
        Self::ffi_get_config(self.ffi_encoder).expect("encoder should have been initialized")
    }

    fn ffi_get_config(ffi_encoder: *mut ffi::vvencEncoder) -> Result<Config, Error> {
        let mut ffi_config = MaybeUninit::uninit();

        unsafe { ffi::vvenc_get_config(ffi_encoder, ffi_config.as_mut_ptr()) }.to_result()?;

        Ok(Config::with_ffi(
            // SAFETY: vvenc_get_config has filled the config
            unsafe { ffi_config.assume_init() },
        ))
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            ffi::vvenc_encoder_close(self.ffi_encoder);
        }
    }
}

fn get_access_unit_size_scale(config: &Config) -> i32 {
    if config.internal_chroma_format().unwrap() <= ChromaFormat::Chroma420 {
        2
    } else {
        3
    }
}
