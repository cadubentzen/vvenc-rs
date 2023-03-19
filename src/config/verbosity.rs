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

impl From<Verbosity> for u32 {
    fn from(value: Verbosity) -> Self {
        match value {
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
