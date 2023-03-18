use crate::*;

pub enum EncoderOutput<'a> {
    None,
    Data(EncoderOutputData<'a>),
    EncodingDone(EncoderOutputData<'a>),
}

#[derive(Debug)]
pub struct EncoderOutputData<'a> {
    pub payload: &'a [u8],
    pub pts: Option<u64>,
    pub dts: Option<u64>,
    pub is_random_access_point: bool,
    pub slice_type: SliceType,
    pub temporal_layer: i32,
}
