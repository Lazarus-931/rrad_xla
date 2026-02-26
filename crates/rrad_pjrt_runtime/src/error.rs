use std::error::Error;
use std::fmt;

/// Macro for a standard version of error's for the pjrt crate
macro_rules! define_pjrt_error {
    ($(($code:literal, $name:ident, $status:expr)),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum X_PJRTError {
            $($name),*
        }

        impl X_PJRTError {
            pub fn code(&self) -> &'static str {
                match self {
                    $(X_PJRTError::$name => $code),*
                }
            }

            pub fn status(&self) -> &'static str {
                match self {
                    $(X_PJRTError::$name => $status),*
                }
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn PJRT_Error_Destroy(err: *mut X_PJRTError) {
            if err.is_null() {
                return;
            } else {
                drop(Box::from_raw(err));
            }
        }
    }
}

macro_rules! defines_error_function {
    () => {};
}

/// Defines a list of pjrt errors between rrad and external c
/// TODO: Above, make it so that macros generate custom functions in replacement of std error calls
pjrt_error_code_list! {
        ("E000", NotFound, "kNotFound"),
        ("E001", Unknown, "kUnknown"),
        ("E002", InvalidArg, "kInvalidArg"),
        ("E003", DeadLineExceeded, "kDeadLineExceeded"),
        ("E004", DataLoss, "kDataLoss"),
        ("E005", Unauthenticated, "kUnauthenticated"),
        ("E006", AlreadyExists, "kAlreadyExists"),
}







