use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verbosity {
    Silent,
    Error,
    Warning,
    Info,
    Notice,
    Verbose,
    Details,
}

impl IntoFFI<u32> for Verbosity {
    fn into_ffi(self) -> u32 {
        match self {
            Self::Silent => ffi::vvencMsgLevel_VVENC_SILENT,
            Self::Error => ffi::vvencMsgLevel_VVENC_ERROR,
            Self::Warning => ffi::vvencMsgLevel_VVENC_WARNING,
            Self::Info => ffi::vvencMsgLevel_VVENC_INFO,
            Self::Notice => ffi::vvencMsgLevel_VVENC_NOTICE,
            Self::Verbose => ffi::vvencMsgLevel_VVENC_VERBOSE,
            Self::Details => ffi::vvencMsgLevel_VVENC_DETAILS,
        }
    }
}

impl TryFromFFI<i32> for Verbosity {
    fn try_from_ffi(value: i32) -> std::result::Result<Self, i32> {
        match value as u32 {
            ffi::vvencMsgLevel_VVENC_SILENT => Ok(Self::Silent),
            ffi::vvencMsgLevel_VVENC_ERROR => Ok(Self::Error),
            ffi::vvencMsgLevel_VVENC_WARNING => Ok(Self::Warning),
            ffi::vvencMsgLevel_VVENC_INFO => Ok(Self::Info),
            ffi::vvencMsgLevel_VVENC_NOTICE => Ok(Self::Notice),
            ffi::vvencMsgLevel_VVENC_VERBOSE => Ok(Self::Verbose),
            ffi::vvencMsgLevel_VVENC_DETAILS => Ok(Self::Details),
            _ => Err(value),
        }
    }
}
