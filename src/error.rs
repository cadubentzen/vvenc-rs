use crate::*;
use thiserror::Error;

pub(crate) const RETURN_OK: i32 = ffi::ErrorCodes_VVENC_OK;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("unspecified malfunction")]
    Unspecified,
    #[error("encoder not initialized or tried to initialize multiple times")]
    Initialize,
    #[error("internal allocation error")]
    Allocate,
    #[error("allocated memory to small to receive encoded data. After allocating sufficient memory the failed call can be repeated.")]
    NotEnoughMem,
    #[error("inconsistent or invalid parameters")]
    Parameter,
    #[error("unsupported request")]
    NotSupported,
    #[error("encoder requires restart")]
    RestartRequired,
    #[error("unsupported CPU SSE 4.1 needed")]
    Cpu,
}

impl TryFrom<i32> for Error {
    type Error = i32;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            ffi::ErrorCodes_VVENC_ERR_UNSPECIFIED => Ok(Self::Unspecified),
            ffi::ErrorCodes_VVENC_ERR_INITIALIZE => Ok(Self::Initialize),
            ffi::ErrorCodes_VVENC_ERR_ALLOCATE => Ok(Self::Allocate),
            ffi::ErrorCodes_VVENC_NOT_ENOUGH_MEM => Ok(Self::NotEnoughMem),
            ffi::ErrorCodes_VVENC_ERR_PARAMETER => Ok(Self::Parameter),
            ffi::ErrorCodes_VVENC_ERR_NOT_SUPPORTED => Ok(Self::NotSupported),
            ffi::ErrorCodes_VVENC_ERR_RESTART_REQUIRED => Ok(Self::RestartRequired),
            ffi::ErrorCodes_VVENC_ERR_CPU => Ok(Self::Cpu),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from() {
        assert_eq!(
            Error::Unspecified,
            ffi::ErrorCodes_VVENC_ERR_UNSPECIFIED.try_into().unwrap()
        );

        assert_eq!(Error::try_from(1).unwrap_err(), 1);
    }

    #[test]
    fn decorate() {
        assert!(RETURN_OK.to_result().is_ok());
        assert_eq!(
            ffi::ErrorCodes_VVENC_ERR_UNSPECIFIED.to_result(),
            Err(Error::Unspecified)
        );
    }
}
