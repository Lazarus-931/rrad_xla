use std::ffi::c_char;
use std::ptr::{null, null_mut};

use crate::sys::pjrt::{
    PJRT_Error, PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
    PJRT_Error_Destroy_Args, PJRT_Error_GetCode_Args, PJRT_Error_Message_Args,
};

use crate::runtime_util::error::PJRTRuntimeError;

#[repr(C)]
struct RuntimeError {
    code: PJRT_Error_Code,
    message: String,
}

pub fn new_error(code: PJRT_Error_Code, message: impl Into<String>) -> *mut PJRT_Error {
    let inner = RuntimeError {
        code,
        message: message.into(),
    };
    Box::into_raw(Box::new(inner)) as *mut PJRT_Error
}

pub fn new_runtime_error(error: &PJRTRuntimeError) -> *mut PJRT_Error {
    let msg = format!("{}: {}", error.error_literal(), error.sentence());
    new_error(error.code(), msg)
}

unsafe fn cast_error<'a>(error: *const PJRT_Error) -> &'a RuntimeError {
    // SAFETY: PJRT_Error pointers produced by this crate are allocated from RuntimeError.
    unsafe { &*(error as *const RuntimeError) }
}

#[inline]
fn invalid_arg_error(message: &str) -> *mut PJRT_Error {
    new_error(
        PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
        message.to_string(),
    )
}

pub unsafe extern "C" fn pjrt_error_destroy(args: *mut PJRT_Error_Destroy_Args) {
    if args.is_null() {
        return;
    }

    // SAFETY: args is checked non-null above and points to C-compatible args.
    let args = unsafe { &mut *args };
    if args.error.is_null() {
        return;
    }

    let ptr = args.error as *mut RuntimeError;
    // SAFETY: pointer was allocated via Box::into_raw in new_error.
    unsafe {
        drop(Box::from_raw(ptr));
    }
    args.error = null_mut();
}

pub unsafe extern "C" fn pjrt_error_message(args: *mut PJRT_Error_Message_Args) {
    if args.is_null() {
        return;
    }

    // SAFETY: args is checked non-null above and points to C-compatible args.
    let args = unsafe { &mut *args };
    if args.error.is_null() {
        args.message = null();
        args.message_size = 0;
        return;
    }

    // SAFETY: args.error is non-null and produced by new_error.
    let err = unsafe { cast_error(args.error) };
    args.message = err.message.as_ptr() as *const c_char;
    args.message_size = err.message.len();
}

pub unsafe extern "C" fn pjrt_error_get_code(
    args: *mut PJRT_Error_GetCode_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return invalid_arg_error("PJRT_Error_GetCode args is null");
    }

    // SAFETY: args is checked non-null above and points to C-compatible args.
    let args = unsafe { &mut *args };
    if args.error.is_null() {
        return invalid_arg_error("PJRT_Error_GetCode error is null");
    }

    // SAFETY: args.error is non-null and produced by new_error.
    let err = unsafe { cast_error(args.error) };
    args.code = err.code;
    null_mut()
}
