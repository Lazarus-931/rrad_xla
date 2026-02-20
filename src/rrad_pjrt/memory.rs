use std::ptr;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use crate::pjrt_sys::*;
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};

pub struct PJRTMemory<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Memory,
}

impl<'a> PJRTMemory<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Memory) -> Self {
        Self { rt, raw }
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Memory, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_Memory is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn id(&self) -> Result<usize, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Memory_Id
            .ok_or_else(|| self.error("PJRT_Memory_Id symbol not found"))?;

        let mut args = PJRT_Memory_Id_Args {
            struct_size: PJRT_Memory_Id_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            id: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(self.error("Erroris non-null"))
        } else {
            Ok(args.id as usize)
        }
    }

    pub fn kind(&self) -> Result<String, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Memory_Kind
            .ok_or_else(|| self.error("PJRT_Memory_Kind symbol not found"))?;

        let mut args = PJRT_Memory_Kind_Args {
            struct_size: PJRT_Memory_Kind_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            kind: ptr::null(),
            kind_size: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else if args.kind_size == 0 {
            Ok(String::new())
        } else if args.kind.is_null() {
            Err(self
                .error("PJRT_Memory_Kind returned null kind with nonzero size"))
        } else {
            let bytes = unsafe { from_raw_parts(args.kind as *const u8, args.kind_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn kind_id(&self) -> Result<i32, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self.rt.api().PJRT_Memory_Kind_Id.ok_or_else(|| {
            self.error("PJRT_Memory_Kind_Id symbol not found")
        })?;

        let mut args = PJRT_Memory_Kind_Id_Args {
            struct_size: PJRT_Memory_Kind_Id_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            kind_id: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else {
            Ok(args.kind_id)
        }
    }

    pub fn debug_string(&self) -> Result<String, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self.rt.api().PJRT_Memory_DebugString.ok_or_else(|| {
            self.error("PJRT_Memory_DebugString symbol not found")
        })?;

        let mut args = PJRT_Memory_DebugString_Args {
            struct_size: PJRT_Memory_DebugString_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            debug_string: ptr::null(),
            debug_string_size: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else if args.debug_string_size == 0 {
            Ok(String::new())
        } else if args.debug_string.is_null() {
            Err(self
                .error("PJRT_Memory_DebugString returned null debug string with nonzero size"))
        } else {
            let bytes =
                unsafe { from_raw_parts(args.debug_string as *const u8, args.debug_string_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn to_string(&self) -> Result<String, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self.rt.api().PJRT_Memory_ToString.ok_or_else(|| {
            self.error("PJRT_Memory_ToString symbol not found")
        })?;

        let mut args = PJRT_Memory_ToString_Args {
            struct_size: PJRT_Memory_ToString_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            to_string: ptr::null(),
            to_string_size: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else if args.to_string_size == 0 {
            Ok(String::new())
        } else if args.to_string.is_null() {
            Err(self
                .error("PJRT_Memory_ToString returned null string with nonzero size"))
        } else {
            let bytes = unsafe { from_raw_parts(args.to_string as *const u8, args.to_string_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn addressable_by_device(&self) -> Result<Vec<PJRTDevice<'a>>, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Memory_AddressableByDevices
            .ok_or_else(|| {
                self.error("PJRT_Memory_AddressableByDevices symbol not found")
            })?;

        let mut args = PJRT_Memory_AddressableByDevices_Args {
            struct_size: PJRT_Memory_AddressableByDevices_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            devices: ptr::null(),
            num_devices: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if args.num_devices == 0 {
            return Ok(Vec::new());
        }
        if args.devices.is_null() {
            return Err(self
                .error("PJRT_Memory_AddressableByDevices returned null devices with nonzero count"));
        }

        let devices = unsafe { from_raw_parts(args.devices, args.num_devices) };
        Ok(devices
            .iter()
            .copied()
            .map(|raw_device| PJRTDevice::new(self.rt, raw_device))
            .collect())
    }
}
