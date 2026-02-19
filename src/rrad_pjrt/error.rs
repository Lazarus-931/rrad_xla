use std::fmt;
use std::ptr;

use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PJRTError<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Error,
}

impl<'a> PJRTError<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Error) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_Error {
        self.raw
    }

    pub fn into_raw(mut self) -> *mut PJRT_Error {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        std::mem::forget(self);
        raw
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
            code: PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(args.code)
        }
    }

    pub fn message(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Error_Message
            .ok_or("PJRT_Error_Message symbol not found")?;

        let mut args = PJRT_Error_Message_Args {
            struct_size: PJRT_Error_Message_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            error: raw,
            message: ptr::null(),
            message_size: 0,
        };
        unsafe { func(&mut args) };

        if args.message.is_null() {
            if args.message_size == 0 {
                return Ok(String::new());
            }
            return Err("PJRT_Error_Message returned null message with nonzero size".to_string());
        }

        let bytes = unsafe { std::slice::from_raw_parts(args.message as *const u8, args.message_size) };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

impl fmt::Debug for PJRTError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.get_code().ok();
        let message = self.message().ok();
        f.debug_struct("PJRTError")
            .field("raw", &self.raw)
            .field("code", &code)
            .field("message", &message)
            .finish()
    }
}

impl fmt::Display for PJRTError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.get_code(), self.message()) {
            (Ok(code), Ok(msg)) if msg.is_empty() => write!(f, "PJRT error code {}", code),
            (Ok(code), Ok(msg)) => write!(f, "PJRT error code {}: {}", code, msg),
            (Ok(code), Err(_)) => write!(f, "PJRT error code {}", code),
            (Err(_), Ok(msg)) => write!(f, "PJRT error: {}", msg),
            (Err(_), Err(_)) => write!(f, "PJRT error"),
        }
    }
}

impl std::error::Error for PJRTError<'_> {}

impl Drop for PJRTError<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(func) = self.rt.api().PJRT_Error_Destroy else {
            return;
        };

        let mut args = PJRT_Error_Destroy_Args {
            struct_size: PJRT_Error_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            error: self.raw,
        };
        unsafe { func(&mut args) };
        self.raw = ptr::null_mut();
    }
}
