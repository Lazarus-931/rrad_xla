use std::ptr;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use crate::pjrt::device::PJRTDevice;
use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PJRTMemory<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Memory,
}

impl <'a> PJRTMemory<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Memory) -> Self {
        Self { rt, raw }
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Memory, String> {
        if self.raw.is_null() {
            Err("PJRT_Memory is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn id(&self) -> Result<usize, String> {
        let raw = self.raw_checked()?;

        let func = self.rt
            .api().PJRT_Memory_Id
            .ok_or("PJRT_Memory_Id symbol not found")?;

        let mut args = PJRT_Memory_Id_Args {
            struct_size: PJRT_Memory_Id_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            id: 0
        };

        let err = unsafe {
            func(&mut args)
        };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else {
            Ok(args.id as usize)
        }
    }

    pub fn kind(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;

        let func = self.rt
            .api().PJRT_Memory_Kind
            .ok_or("PJRT_Memory_Kind symbol not found")?;

        let mut args = PJRT_Memory_Kind_Args {
            struct_size: PJRT_Memory_Kind_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            kind: ptr::null(),
            kind_size: 0
        };

        let err = unsafe {
            func(&mut args)
        };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else if args.kind_size == 0 {
            Ok(String::new())
        } else if args.kind.is_null() {
            Err("PJRT_Memory_Kind returned null kind with nonzero size".to_string())
        } else {
            let bytes = unsafe { from_raw_parts(args.kind as *const u8, args.kind_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn kind_id(&self) -> Result<i32, String> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Memory_Kind_Id
            .ok_or("PJRT_Memory_Kind_Id symbol not found")?;

        let mut args = PJRT_Memory_Kind_Id_Args {
            struct_size: PJRT_Memory_Kind_Id_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            kind_id: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(args.kind_id)
        }
    }

    pub fn debug_string(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;

        let funct = self.rt
            .api().PJRT_Memory_DebugString
            .ok_or("PJRT_Memory_DebugString symbol not found")?;

        let mut args = PJRT_Memory_DebugString_Args {
            struct_size: PJRT_Memory_DebugString_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            debug_string: ptr::null(),
            debug_string_size: 0
        };

        let err = unsafe {
            funct(&mut args)
        };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else if args.debug_string_size == 0 {
            Ok(String::new())
        } else if args.debug_string.is_null() {
            Err("PJRT_Memory_DebugString returned null debug string with nonzero size".to_string())
        } else {
            let bytes = unsafe { from_raw_parts(args.debug_string as *const u8, args.debug_string_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }
    
    
    pub fn to_string(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;

        let function = self.rt
            .api().PJRT_Memory_ToString
            .ok_or("PJRT_Memory_ToString symbol not found")?;

        let mut args = PJRT_Memory_ToString_Args {
            struct_size: PJRT_Memory_ToString_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            to_string: ptr::null(),
            to_string_size: 0
        };

        let err = unsafe {
            function(&mut args)
        };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else if args.to_string_size == 0 {
            Ok(String::new())
        } else if args.to_string.is_null() {
            Err("PJRT_Memory_ToString returned null string with nonzero size".to_string())
        } else {
            let bytes = unsafe { from_raw_parts(args.to_string as *const u8, args.to_string_size) };
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn addressable_by_device(&self) -> Result<Vec<PJRTDevice<'a>>, String> {
        let raw = self.raw_checked()?;

        let function = self.rt
            .api().PJRT_Memory_AddressableByDevices
            .ok_or("PJRT_Memory_AddressableByDevices symbol not found")?;

        let mut args = PJRT_Memory_AddressableByDevices_Args {
            struct_size: PJRT_Memory_AddressableByDevices_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            memory: raw,
            devices: ptr::null(),
            num_devices: 0,
        };

        let err = unsafe { function(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_devices == 0 {
            return Ok(Vec::new());
        }
        if args.devices.is_null() {
            return Err(
                "PJRT_Memory_AddressableByDevices returned null devices with nonzero count"
                    .to_string(),
            );
        }

        let devices = unsafe { from_raw_parts(args.devices, args.num_devices) };
        Ok(devices
            .iter()
            .copied()
            .map(|raw_device| PJRTDevice::new(self.rt, raw_device))
            .collect())
    }
}
