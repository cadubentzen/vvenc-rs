use crate::*;

#[derive(Debug, PartialEq)]
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
            Verbosity::Silent => ffi::vvencMsgLevel_VVENC_SILENT,
            Verbosity::Error => ffi::vvencMsgLevel_VVENC_ERROR,
            Verbosity::Warning => ffi::vvencMsgLevel_VVENC_WARNING,
            Verbosity::Info => ffi::vvencMsgLevel_VVENC_INFO,
            Verbosity::Notice => ffi::vvencMsgLevel_VVENC_NOTICE,
            Verbosity::Verbose => ffi::vvencMsgLevel_VVENC_VERBOSE,
            Verbosity::Details => ffi::vvencMsgLevel_VVENC_DETAILS,
        }
    }
}
