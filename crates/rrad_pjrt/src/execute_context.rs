use std::ptr;

use crate::ffi::pjrt_sys::*;
use crate::error::PJRTError;
use crate::loader::PjrtRuntime;

pub struct PJRTExecuteContext<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_ExecuteContext,
}

impl<'a> PJRTExecuteContext<'a> {
    fn invalid_arg(rt: &'a PjrtRuntime, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(rt, msg)
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_ExecuteContext, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_ExecuteContext is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn create(rt: &'a PjrtRuntime) -> Result<Self, PJRTError<'a>> {
        let f = rt
            .api()
            .PJRT_ExecuteContext_Create
            .ok_or_else(|| Self::invalid_arg(rt, "PJRT_ExecuteContext_Create symbol not found"))?;

        let mut args = PJRT_ExecuteContext_Create_Args {
            struct_size: PJRT_ExecuteContext_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            context: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(rt, err));
        }
        if args.context.is_null() {
            return Err(Self::invalid_arg(
                rt,
                "PJRT_ExecuteContext_Create returned null context",
            ));
        }

        Ok(Self {
            rt,
            raw: args.context,
        })
    }

    pub fn raw(&self) -> *mut PJRT_ExecuteContext {
        self.raw
    }

    pub fn into_raw(self) -> *mut PJRT_ExecuteContext {
        let raw = self.raw;
        std::mem::forget(self);
        raw
    }
}

impl Drop for PJRTExecuteContext<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_ExecuteContext_Destroy else {
            return;
        };

        let mut args = PJRT_ExecuteContext_Destroy_Args {
            struct_size: PJRT_ExecuteContext_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            context: self.raw,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            let _ = PJRTError::new(self.rt, err);
        }

        self.raw = ptr::null_mut();
    }
}
