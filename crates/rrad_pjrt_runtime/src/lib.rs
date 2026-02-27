pub mod bridge;

pub mod wrapper;
pub mod error;
pub mod runtime_util;

pub mod utils;

pub mod domain;
pub mod sys;

use sys::pjrt::PJRT_Api;

#[no_mangle]
pub extern "C" fn GetPjrtApi() -> *const PJRT_Api {
    bridge::c_api::get_pjrt_api()
}
