use std::ptr;
use std::slice::from_raw_parts;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;


pub struct PJRTLoadedExecutable<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_LoadedExecutable,
}

// Back-compat with the original name in this crate.
pub type PJRTExecutable<'a> = PJRTLoadedExecutable<'a>;

impl<'a> PJRTLoadedExecutable<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_LoadedExecutable) -> Self {
        Self { rt, raw }
    }

    fn raw_checked(&self) -> Result<*mut PJRT_LoadedExecutable, String> {
        if self.raw.is_null() {
            Err("PJRT_LoadedExecutable is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    fn executable(&self) -> Result<*mut PJRT_Executable, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_GetExecutable
            .ok_or("PJRT_LoadedExecutable_GetExecutable symbol not found")?;

        let mut args = PJRT_LoadedExecutable_GetExecutable_Args {
            struct_size: PJRT_LoadedExecutable_GetExecutable_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            loaded_executable: raw,
            executable: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.executable.is_null() {
            return Err("PJRT_LoadedExecutable_GetExecutable returned null executable".into());
        }
        Ok(args.executable)
    }

    pub fn num_replicas(&self) -> Result<usize, String> {
        let exec = self.executable()?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_NumReplicas
            .ok_or("PJRT_Executable_NumReplicas symbol not found")?;

        let mut args = PJRT_Executable_NumReplicas_Args {
            struct_size: PJRT_Executable_NumReplicas_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_replicas: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.num_replicas)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn num_partitions(&self) -> Result<usize, String> {
        let exec = self.executable()?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_NumPartitions
            .ok_or("PJRT_Executable_NumPartitions symbol not found")?;

        let mut args = PJRT_Executable_NumPartitions_Args {
            struct_size: PJRT_Executable_NumPartitions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_partitions: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.num_partitions)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }


    pub fn addressable_devices(&self) -> Result<Vec<*mut PJRT_Device>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_AddressableDevices
            .ok_or("PJRT_LoadedExecutable_AddressableDevices symbol not found")?;

        let mut args = PJRT_LoadedExecutable_AddressableDevices_Args {
            struct_size: PJRT_LoadedExecutable_AddressableDevices_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            addressable_devices: ptr::null(),
            num_addressable_devices: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_addressable_devices == 0 {
            return Ok(Vec::new());
        }
        if args.addressable_devices.is_null() {
            return Err("PJRT_LoadedExecutable_AddressableDevices returned null list with nonzero count".into());
        }

        let devices =
            unsafe { from_raw_parts(args.addressable_devices, args.num_addressable_devices) };
        Ok(devices.to_vec())
    }

    pub fn fingerprint(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_Fingerprint
            .ok_or("PJRT_LoadedExecutable_Fingerprint symbol not found")?;

        let mut args = PJRT_LoadedExecutable_Fingerprint_Args {
            struct_size: PJRT_LoadedExecutable_Fingerprint_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            executable_fingerprint: ptr::null(),
            executable_fingerprint_size: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        if args.executable_fingerprint.is_null() {
            return Err("PJRT_LoadedExecutable_Fingerprint returned null fingerprint".into());
        }

        let bytes = unsafe {
            from_raw_parts(
                args.executable_fingerprint as *const u8,
                args.executable_fingerprint_size,
            )
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn name(&self) -> Result<String, String> {
        let exec = self.executable()?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_Name
            .ok_or("PJRT_Executable_Name symbol not found")?;

        let mut args = PJRT_Executable_Name_Args {
            struct_size: PJRT_Executable_Name_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            executable_name: ptr::null(),
            executable_name_size: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        if args.executable_name.is_null() {
            return Err("PJRT_Executable_Name returned null executable_name".into());
        }

        let bytes = unsafe {
            from_raw_parts(args.executable_name as *const u8, args.executable_name_size)
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

impl Drop for PJRTLoadedExecutable<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_LoadedExecutable_Destroy else {
            return;
        };

        let mut args = PJRT_LoadedExecutable_Destroy_Args {
            struct_size: PJRT_LoadedExecutable_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: self.raw,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            // Drop must not panic; best-effort cleanup.
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}
