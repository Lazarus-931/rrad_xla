use std::ptr;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PJRTError<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Error,
}

impl<'a> PJRTError<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Error) -> Self {
        Self { rt, raw }
    }

    pub fn raw_checked(&self) -> Result<*mut PJRT_Error, String> {
        if self.raw.is_null() {
            Err("PJRT_Error is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn get_code(&self) -> Result<PJRT_Error_Code, String> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Error_GetCode
            .ok_or("PJRT_Error_GetCode symbol not found")?;

        let mut args = PJRT_Error_GetCode_Args {
            struct_size: PJRT_Error_GetCode_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            error: raw,
            code: 0,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else {
            Ok(args.code)
        }
    }
}
