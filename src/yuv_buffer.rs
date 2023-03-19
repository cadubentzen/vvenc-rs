use crate::*;

pub struct YUVBuffer<'a> {
    pub planes: [YUVPlane<'a>; 3],
    pub sequence_number: u64,
    pub composition_timestamp: Option<u64>,
}

pub struct YUVPlane<'a> {
    pub buffer: &'a [i16],
    pub width: i32,
    pub height: i32,
    pub stride: i32,
}

impl<'a> YUVPlane<'a> {
    pub(crate) fn to_ffi(&self) -> ffi::vvencYUVPlane {
        ffi::vvencYUVPlane {
            // this const -> mut cast should be OK if VVenC doesn't modify
            // the buffer internally. It copies the buffer after being pushed.
            ptr: self.buffer.as_ptr() as *mut i16,
            width: self.width,
            height: self.height,
            stride: self.stride,
        }
    }
}

impl<'a> YUVBuffer<'a> {
    pub(crate) fn to_ffi(&self) -> ffi::vvencYUVBuffer {
        ffi::vvencYUVBuffer {
            planes: [
                self.planes[0].to_ffi(),
                self.planes[1].to_ffi(),
                self.planes[2].to_ffi(),
            ],
            sequenceNumber: self.sequence_number,
            cts: self.composition_timestamp.unwrap_or(0),
            ctsValid: self.composition_timestamp.is_some(),
        }
    }
}
