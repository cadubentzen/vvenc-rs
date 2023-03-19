use crate::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ChromaFormat {
    Chroma400,
    Chroma420,
    Chroma422,
    Chroma444,
}

impl TryFrom<u32> for ChromaFormat {
    type Error = u32;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            ffi::vvencChromaFormat_VVENC_CHROMA_400 => Ok(ChromaFormat::Chroma400),
            ffi::vvencChromaFormat_VVENC_CHROMA_420 => Ok(ChromaFormat::Chroma420),
            ffi::vvencChromaFormat_VVENC_CHROMA_422 => Ok(ChromaFormat::Chroma422),
            ffi::vvencChromaFormat_VVENC_CHROMA_444 => Ok(ChromaFormat::Chroma444),
            _ => Err(value),
        }
    }
}
