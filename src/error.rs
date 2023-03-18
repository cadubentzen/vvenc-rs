use crate::*;

pub(crate) const RETURN_OK: i32 = ffi::ErrorCodes_VVENC_OK;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// unspecified malfunction
    Unspecified,
    /// encoder not initialized or tried to initialize multiple times
    Initialize,
    /// internal allocation error
    Allocate,
    /// allocated memory to small to receive encoded data. After allocating sufficient memory the failed call can be repeated.
    NotEnoughMem,
    /// inconsistent or invalid parameters
    Parameter,
    /// unsupported request
    NotSupported,
    /// encoder requires restart
    RestartRequired,
    /// unsupported CPU SSE 4.1 needed
    Cpu,
}

impl TryFrom<i32> for Error {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
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

pub(crate) trait FFIStatusToResult {
    fn to_result(self) -> Result<(), Error>;
}

impl FFIStatusToResult for i32 {
    fn to_result(self) -> Result<(), Error> {
        if self == crate::error::RETURN_OK {
            Ok(())
        } else {
            Err(match self.try_into() {
                Ok(error) => error,
                Err(ffi_error) => panic!(
                    "error code {} from libvvenc not enumerated by this crate. Please file an issue.",
                    ffi_error
                ),
            })
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
