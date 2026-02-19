use std::fmt;
use std::ptr::{null, null_mut};

use crate::pjrt_sys::*;
use crate::rrad_pjrt::loader::PjrtRuntime;

pub struct PJRTError<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Error,
    pub local_code: Option<PJRT_Error_Code>,
    pub local_message: Option<String>,
}

impl<'a> PJRTError<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Error) -> Self {
        Self {
            rt,
            raw,
            local_code: None,
            local_message: None,
        }
    }

    pub fn raw(&self) -> *mut PJRT_Error {
        self.raw
    }

    pub fn into_raw(mut self) -> *mut PJRT_Error {
        let raw = self.raw;
        self.raw = null_mut();
        std::mem::forget(self);
        raw
    }

    pub fn raw_checked(&self) -> Result<*mut PJRT_Error, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(PJRTError::invalid_arg(self.rt, "PJRT_Error is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn get_code(&self) -> Result<PJRT_Error_Code, PJRTError<'a>> {
        if let Some(code) = self.local_code {
            return Ok(code);
        }

        let raw = self.raw_checked()?;

        let func = self.rt.api().PJRT_Error_GetCode.ok_or_else(|| {
            PJRTError::invalid_arg(self.rt, "PJRT_Error_GetCode symbol not found")
        })?;

        let mut args = PJRT_Error_GetCode_Args {
            struct_size: PJRT_Error_GetCode_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            error: raw,
            code: PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else {
            Ok(args.code)
        }
    }

    pub fn message(&self) -> Result<String, String> {
        if let Some(msg) = &self.local_message {
            return Ok(msg.clone());
        }

        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let func = self.rt.api().PJRT_Error_Message.ok_or_else(|| {
            PJRTError::invalid_arg(self.rt, "PJRT_Error_Message symbol not found").to_string()
        })?;

        let mut args = PJRT_Error_Message_Args {
            struct_size: PJRT_Error_Message_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            error: raw,
            message: null(),
            message_size: 0,
        };
        unsafe { func(&mut args) };

        if args.message.is_null() {
            if args.message_size == 0 {
                return Ok(String::new());
            }
            return Err("PJRT_Error_Message returned null message with nonzero size".to_string());
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(args.message as *const u8, args.message_size) };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn invalid_arg(rt: &'a PjrtRuntime, msg: impl Into<String>) -> Self {
        Self {
            rt,
            raw: null_mut(),
            local_code: Some(PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT),
            local_message: Some(msg.into()),
        }
    }

    pub fn local(rt: &'a PjrtRuntime, code: PJRT_Error_Code, msg: impl Into<String>) -> Self {
        Self {
            rt,
            local_code: Some(code),
            local_message: Some(msg.into()),
            raw: null_mut(),
        }
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
        if let Some(code) = self.local_code {
            if let Some(msg) = self.local_message.as_deref() {
                if msg.is_empty() {
                    return write!(f, "PJRT error code {}", code);
                }
                return write!(f, "PJRT error code {}: {}", code, msg);
            }
            return write!(f, "PJRT error code {}", code);
        }

        if let Some(msg) = self.local_message.as_deref() {
            return if msg.is_empty() {
                write!(f, "PJRT error")
            } else {
                write!(f, "PJRT error: {}", msg)
            };
        }

        match (self.get_code(), self.message()) {
            (Ok(code), Ok(msg)) if msg.is_empty() => write!(f, "PJRT error code {}", code),
            (Ok(code), Ok(msg)) => write!(f, "PJRT error code {}: {}", code, msg),
            (Ok(code), Err(_)) => write!(f, "PJRT error code {}", code),
            (Err(_), Ok(msg)) if msg.is_empty() => write!(f, "PJRT error"),
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
            extension_start: null_mut(),
            error: self.raw,
        };
        unsafe { func(&mut args) };
        self.raw = null_mut();
    }
}

pub fn from_raw<'a>(rt: &'a PjrtRuntime, raw: *mut PJRT_Error) -> PJRTError<'a> {
    PJRTError::new(rt, raw)
}
