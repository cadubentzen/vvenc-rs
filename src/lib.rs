#[cfg(test)]
mod tests {
    use libvvenc_sys as ffi;
    use std::mem::MaybeUninit;

    #[test]
    fn it_works() {
        let mut params = MaybeUninit::uninit();

        const WIDTH: i32 = 1280;
        const HEIGHT: i32 = 720;
        let framerate = 30;
        let bitrate = 2_000_000;
        let qp = 23;
        let preset = ffi::vvencPresetMode_VVENC_MEDIUM;

        let res = unsafe {
            ffi::vvenc_init_default(
                params.as_mut_ptr(),
                WIDTH,
                HEIGHT,
                framerate,
                bitrate,
                qp,
                preset,
            )
        };
        assert_eq!(res, ffi::ErrorCodes_VVENC_OK);

        // SAFETY: vvenc_init_default initialized params
        let mut params = unsafe { params.assume_init() };

        let encoder = unsafe { ffi::vvenc_encoder_create() };

        let res = unsafe { ffi::vvenc_encoder_open(encoder, &mut params as *mut _) };
        assert_eq!(res, ffi::ErrorCodes_VVENC_OK);

        let mut y_plane = Vec::with_capacity((WIDTH * HEIGHT) as usize);
        let mut u_plane = Vec::with_capacity((WIDTH * HEIGHT) as usize >> 1);
        let mut v_plane = Vec::with_capacity((WIDTH * HEIGHT) as usize >> 1);

        y_plane.resize((WIDTH * HEIGHT) as usize, 0);
        u_plane.resize((WIDTH * HEIGHT) as usize >> 1, 0);
        v_plane.resize((WIDTH * HEIGHT) as usize >> 1, 0);

        let mut yuvbuf = ffi::vvencYUVBuffer {
            planes: [
                ffi::vvencYUVPlane {
                    ptr: y_plane.as_mut_ptr(),
                    width: WIDTH,
                    height: HEIGHT,
                    stride: WIDTH,
                },
                ffi::vvencYUVPlane {
                    ptr: u_plane.as_mut_ptr(),
                    width: WIDTH >> 1,
                    height: HEIGHT >> 1,
                    stride: WIDTH >> 1,
                },
                ffi::vvencYUVPlane {
                    ptr: v_plane.as_mut_ptr(),
                    width: WIDTH >> 1,
                    height: HEIGHT >> 1,
                    stride: WIDTH >> 1,
                },
            ],
            sequenceNumber: 0,
            cts: 0,
            ctsValid: true,
        };

        let au = unsafe { ffi::vvenc_accessUnit_alloc() };

        let au_size_scale =
            if params.m_internChromaFormat <= ffi::vvencChromaFormat_VVENC_CHROMA_420 {
                2
            } else {
                3
            };

        unsafe {
            ffi::vvenc_accessUnit_alloc_payload(
                au,
                au_size_scale * params.m_SourceWidth * params.m_SourceHeight + 1024,
            );
        }

        let mut enc_done = false;
        let res = unsafe { ffi::vvenc_encode(encoder, &mut yuvbuf, au, &mut enc_done) };
        assert_eq!(res, ffi::ErrorCodes_VVENC_OK);

        while !enc_done {
            let res =
                unsafe { ffi::vvenc_encode(encoder, std::ptr::null_mut(), au, &mut enc_done) };
            assert_eq!(res, ffi::ErrorCodes_VVENC_OK);
            let au = &mut unsafe { *au };
            if au.payloadUsedSize > 0 {
                println!("yay, got frame!");
                println!(
                    "{}",
                    unsafe { std::ffi::CStr::from_ptr(au.infoString.as_ptr()) }
                        .to_str()
                        .unwrap()
                )
            }
        }

        unsafe {
            ffi::vvenc_encoder_close(encoder);
            ffi::vvenc_accessUnit_free(au, true);
        };
    }
}
