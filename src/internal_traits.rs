use crate::*;

pub trait FFIStatusToResult {
    fn to_result(self) -> Result<()>;
}

impl FFIStatusToResult for i32 {
    fn to_result(self) -> Result<()> {
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
