use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use crate::ffi::pjrt_bindings::PJRT_Api;


pub struct PjrtBindingError {
    error_type: PjrtFfiError,
}

// central binding to ffi error
#[derive(Error, Debug)]
pub enum PjrtFfiError {

    // when rrad_pjrt does not support the pjrt_c_api
    #[error("rrad_pjrt currently does not support {pjrt_c_api}this xla pjrt ffi api : {message}")]
    IncompatibleApi{pjrt_c_api: &'static str, message: String},

    // when rrad_pjrt is not able to get api
    #[error("failed to get pjrt api: {message}")]
    FailedToGetPjrtApi { message: String },

    // failed to load pjrt_c_api lib
    #[error("failed to load pjrt_c_api lib: {message}")]
    FailedToLoadPjrtLib { message: String },

    // api is depreciated on PJRT side
    #[error("pjrt api: {rrad_pjrt_bind} is depreciated : {message}")]
    DeprecatedApi { rrad_pjrt_bind: String, message: String },

    // null value returned from pjrt api
    #[error("null value returned from pjrt api: {message}")]
    NullValueReturned { message: String },

    // invalid utf8 string returned from pjrt api
    #[error("invalid utf8 string returned from pjrt api: {message}")]
    InvalidUtf8StringReturned { message: String },

    // api mismatched between rrad_pjrt and pjrt_c_api
    #[error("api mismatched between rrad_pjrt and pjrt_c_api: {message}")]
    ApiMismatch { message: String, expected: &'static str, actual: &'static str, api: &'static str},

    // struct-size mismatched between rrad_pjrt and pjrt_c_api
    #[error("struct-size mismatched between rrad_pjrt and pjrt_c_api: {message}")]
    StructSizeMismatch { message: String, expected: usize, actual: usize, api: &'static str },

    // when the api-version does not match between host and plugin
    #[error("pjrt api version mismatch: host={major_version} plugin={minor_version}")]
    ApiVersionMismatch { major_version: i32, minor_version: i32 },

}

impl PjrtBindingError {
    pub fn new(error_type: PjrtFfiError) -> Self {
        Self {
            error_type
        }
    }

    pub fn lib_load_failed<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::FailedToLoadPjrtLib { message: msg.into() })
    }

    pub fn deprecated_api<M: Into<String>>(pjrt_c_api: &'static str, msg: M) -> Self {
        Self::new(PjrtFfiError::DeprecatedApi { rrad_pjrt_bind: pjrt_c_api.into(), message: msg.into() })
    }

    pub fn failed_to_get_pjrt_api<M: Into<String>>(msg: M) -> Self {
        Self::new(PjrtFfiError::FailedToGetPjrtApi { message: msg.into() })
    }

    pub fn null_value_returned<M: Into<String>>(msg: &'static str) -> Self {
        Self::new(PjrtFfiError::NullValueReturned { message: msg.into() })
    }

    pub fn invalid_utf8_string_returned<M: Into<String>>(msg: &'static str) -> Self {
        Self::new(PjrtFfiError::InvalidUtf8StringReturned { message: msg.into() })
    }

    pub fn api_mismatched<M: Into<String>>(msg: &'static str, expected: &'static str, actual: &'static str, api: &'static str) -> Self {
        Self::new(PjrtFfiError::ApiMismatch { message: msg.into(), expected, actual, api })
    }

    pub fn struct_size_mismatch<M: Into<String>>(msg: &'static str, expected: usize, actual: usize, api: &'static str) -> Self {
        Self::new(PjrtFfiError::StructSizeMismatch { message: msg.into(), expected, actual, api })
    }

    pub fn api_version_mismatch(major_version: i32, minor_version: i32) -> Self {
        Self::new(PjrtFfiError::ApiVersionMismatch { major_version, minor_version })
    }
}

impl Debug for PjrtBindingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for PjrtBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl Error for PjrtBindingError {

}
