use std::ptr;

use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PJRTExecuteContext<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_ExecuteContext,
}

impl<'a> PJRTExecuteContext<'a> {
    pub fn create(rt: &'a PjrtRuntime) -> Result<Self, String> {
        let f = rt
            .api()
            .PJRT_ExecuteContext_Create
            .ok_or("PJRT_ExecuteContext_Create symbol not found")?;

        let mut args = PJRT_ExecuteContext_Create_Args {
            struct_size: PJRT_ExecuteContext_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            context: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(rt.api(), err));
        }
        if args.context.is_null() {
            return Err("PJRT_ExecuteContext_Create returned null context".to_string());
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
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}
