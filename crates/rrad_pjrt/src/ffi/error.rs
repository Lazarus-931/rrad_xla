use thiserror::Error;


// central binding to ffi error
#[derive(Error, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PjrtBindingError {

    // when rrad_pjrt does not support the pjrt_c_api (yet!)
    #[error("rrad_pjrt currently does not support {pjrt_c_api}this xla pjrt ffi api : {message}")]
    IncompatibleApi{pjrt_c_api: &'static str, message: String},

    // api is depreciated on PJRT side
    #[error("pjrt api: {rrad_pjrt_bind} is depreciated : {message}")]
    DepreciatedApi { rrad_pjrt_bind: String, message: String },

    // null value returned from pjrt api
    #[error("null value returned from pjrt api: {message}")]
    NullValueReturned { message: String },

}

impl PjrtBindingError {

}