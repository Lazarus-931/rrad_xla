pub mod c;
pub mod api_wrapper;

pub mod wrapper;
pub mod error;
pub mod runtime_util;

pub mod utils;

pub mod internal;

use crate::c::pjrt::PJRT_Api;

#[no_mangle]
pub extern "C" fn GetPjrtApi() -> *const PJRT_Api {
    api_wrapper::c_api::get_pjrt_api()
}
