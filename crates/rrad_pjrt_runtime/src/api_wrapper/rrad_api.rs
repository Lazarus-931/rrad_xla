use crate::c::pjrt::{PJRT_Client_Create_Args, PJRT_Error};

pub struct RradApi {
    pub struct_size: usize,
    pub pjrt_client_creation: unsafe extern "C" fn( args: *mut PJRT_Client_Create_Args ) -> Result<(), *mut PJRT_Error>,
    pub pjrt_client_destroy: unsafe extern "C" fn( args: *mut PJRT_Client_Create_Args ) -> Result<(), *mut PJRT_Error>,
}

impl RradApi {
    pub fn get_registered_api
}