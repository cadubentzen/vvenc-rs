use crate::*;
use vsprintf::vsprintf;

pub struct LoggingCallback<L: LoggingHandler> {
    handler: L,
}

impl<L: LoggingHandler> LoggingCallback<L> {
    pub fn new(handler: L) -> Self {
        Self { handler }
    }

    pub unsafe extern "C" fn c_callback(
        this: *mut core::ffi::c_void,
        verbosity: core::ffi::c_int,
        fmt: *const core::ffi::c_char,
        va_list: *mut ffi::__va_list_tag,
    ) {
        let message = vsprintf(fmt, va_list).unwrap();
        Self::dispatch(
            &*(this as *mut Self),
            Verbosity::try_from_ffi(verbosity).unwrap(),
            message,
        );
    }

    fn dispatch(&self, verbosity: Verbosity, message: String) {
        self.handler.handle_message(verbosity, message);
    }
}

pub trait LoggingHandler {
    fn handle_message(&self, verbosity: Verbosity, message: String);
}
