use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;
use std::mem;
use std::ptr;
use std::ptr::null_mut;

pub struct PJRTEvent<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_Event,
}

impl<'a> PJRTEvent<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Event) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_Event {
        self.raw
    }

    pub fn into_raw(self) -> *mut PJRT_Event {
        let raw = self.raw;
        mem::forget(self);
        raw
    }

    pub fn create(rt: &'a PjrtRuntime) -> Result<PJRTEvent<'a>, String> {
        let f = rt
            .api()
            .PJRT_Event_Create
            .ok_or("PJRT_Event_Create symbol not found")?;

        let mut args = PJRT_Event_Create_Args {
            struct_size: PJRT_Event_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(rt.api(), err));
        }
        if args.event.is_null() {
            return Err("PJRT_Event_Create returned null event".to_string());
        }

        Ok(PJRTEvent {
            rt,
            raw: args.event,
        })
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Event, String> {
        if self.raw.is_null() {
            Err("PJRT_Event is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn is_ready(&self) -> Result<bool, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Event_IsReady
            .ok_or("PJRT_Event_IsReady symbol not found")?;

        let mut args = PJRT_Event_IsReady_Args {
            struct_size: PJRT_Event_IsReady_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: raw,
            is_ready: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_ready)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn on_ready(
        &self,
        callback: PJRT_Event_OnReadyCallback,
        user_arg: *mut libc::c_void,
    ) -> Result<(), String> {
        let raw = self.raw_checked()?;
        if callback.is_none() {
            return Err("PJRT_Event_OnReady callback must be provided".to_string());
        }

        let func = self
            .rt
            .api()
            .PJRT_Event_OnReady
            .ok_or("PJRT_Event_OnReady symbol not found")?;

        let mut args = PJRT_Event_OnReady_Args {
            struct_size: PJRT_Event_OnReady_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            event: raw,
            callback,
            user_arg,
        };
        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else {
            Ok(())
        }
    }

    pub fn set(&self, error: &PJRTError) -> Result<(), String> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Event_Set
            .ok_or("PJRT_Event_Set symbol not found")?;

        let mut args = PJRT_Event_Set_Args {
            struct_size: PJRT_Event_Set_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            event: raw,
            error_code: error.get_code()?,
            error_message: ptr::null(),
            error_message_size: 0,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else {
            Ok(())
        }
    }

    pub fn await_ready(&self) -> Result<(), String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Event_Await
            .ok_or("PJRT_Event_Await symbol not found")?;

        let mut args = PJRT_Event_Await_Args {
            struct_size: PJRT_Event_Await_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: raw,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn ok(&self) -> Result<(), String> {
        self.await_ready()?;

        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_Event_Error
            .ok_or("PJRT_Event_Error symbol not found")?;

        let mut args = PJRT_Event_Error_Args {
            struct_size: PJRT_Event_Error_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: raw,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }
}

impl Drop for PJRTEvent<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_Event_Destroy else {
            return;
        };

        let mut args = PJRT_Event_Destroy_Args {
            struct_size: PJRT_Event_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: self.raw,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}
