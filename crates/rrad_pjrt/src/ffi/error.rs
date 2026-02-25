use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

pub struct PjrtBindingError {
    error_type: PjrtFfiError,
}

/// This is the error type returned by pjrt bindings and the pjrt crate. It covers major errors that
/// such as api depreciation from xla lib as well as failed to load the api library. This is separate
/// from the binding level error such as [`crate::error::PJRTError`], this is rather lower than that, and precedes it in fact.
/// For example, [`PJRTLoader`](crate::loader::PjrtRuntime) uses PjrtBindingError but not [`crate::error::PJRTError`] since the loader establishes the
/// api in the first place, which is needed for [`crate::error::PJRTError`].


#[derive(Error, Debug)]
pub enum PjrtFfiError {
    #[error("rrad_pjrt currently does not support {pjrt_c_api} this xla pjrt ffi api: {message}")]
    IncompatibleApi {
        pjrt_c_api: &'static str,
        message: String,
    },

    #[error("failed to get pjrt api: {message}")]
    FailedToGetPjrtApi { message: String },

    #[error("failed to load pjrt_c_api lib: {message}")]
    FailedToLoadPjrtLib { message: String },

    #[error("pjrt api: {rrad_pjrt_bind} is deprecated: {message}")]
    DeprecatedApi {
        rrad_pjrt_bind: String,
        message: String,
    },

    #[error("null value returned from pjrt api: {message}")]
    NullValueReturned { message: String },

    #[error("invalid utf8 string returned from pjrt api: {message}")]
    InvalidUtf8StringReturned { message: String },

    #[error("api mismatch for {api}: expected={expected} actual={actual}. {message}")]
    ApiMismatch {
        message: String,
        expected: &'static str,
        actual: &'static str,
        api: &'static str,
    },

    #[error("struct-size mismatch for {api}: expected={expected} actual={actual}. {message}")]
    StructSizeMismatch {
        message: String,
        expected: usize,
        actual: usize,
        api: &'static str,
    },

    #[error("pjrt api version mismatch: plugin_major={major_version} plugin_minor={minor_version}")]
    ApiVersionMismatch {
        major_version: i32,
        minor_version: i32,
    },

    #[error("the ffi function pointer '{name}' is null")]
    NullFunctionPointer { name: &'static str },
}

impl PjrtBindingError {
    pub fn new(error_type: PjrtFfiError) -> Self {
        Self { error_type }
    }

    pub fn lib_load_failed<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::FailedToLoadPjrtLib {
            message: msg.into(),
        })
    }

    pub fn deprecated_api<M: Into<String>>(pjrt_c_api: &'static str, msg: M) -> Self {
        Self::new(PjrtFfiError::DeprecatedApi {
            rrad_pjrt_bind: pjrt_c_api.into(),
            message: msg.into(),
        })
    }

    pub fn failed_to_get_pjrt_api<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::FailedToGetPjrtApi {
            message: msg.into(),
        })
    }

    pub fn null_value_returned<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::NullValueReturned {
            message: msg.into(),
        })
    }

    pub fn invalid_utf8_string_returned<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::InvalidUtf8StringReturned {
            message: msg.into(),
        })
    }

    pub fn api_mismatched<M: Into<String>>(
        msg: M,
        expected: &'static str,
        actual: &'static str,
        api: &'static str,
    ) -> Self {
        Self::new(PjrtFfiError::ApiMismatch {
            message: msg.into(),
            expected,
            actual,
            api,
        })
    }

    pub fn struct_size_mismatch<M: Into<String>>(
        msg: M,
        expected: usize,
        actual: usize,
        api: &'static str,
    ) -> Self {
        Self::new(PjrtFfiError::StructSizeMismatch {
            message: msg.into(),
            expected,
            actual,
            api,
        })
    }

    pub fn mismatched_struct_size<M: Into<String>>(expected_size: usize, actual_size: usize, api: &'static str) -> Self {
        Self::struct_size_mismatch(
            format!("expected struct size {} but got {} for {} ",
                    expected_size,
                    actual_size,
                    api.to_string()),
            expected_size,
            actual_size,
            api
        )
    }

    pub fn api_version_mismatch(major_version: i32, minor_version: i32) -> Self {
        Self::new(PjrtFfiError::ApiVersionMismatch {
            major_version,
            minor_version,
        })
    }
}

impl Debug for PjrtBindingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl Display for PjrtBindingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

impl Error for PjrtBindingError {}
