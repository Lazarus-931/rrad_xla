use std::ptr;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt::memory::PJRTMemory;
use crate::pjrt::topology_desc::{PJRTDeviceDescriptionRef, PJRTNamedAttribute};
use crate::pjrt_sys::*;

pub struct PJRTDevice<'a> {
    rt: &'a PjrtRuntime,
    raw_device: *mut PJRT_Device,
}

impl<'a> PJRTDevice<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw_device: *mut PJRT_Device) -> Self {
        Self { rt, raw_device }
    }

    pub fn raw(&self) -> *mut PJRT_Device {
        self.raw_device
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Device, String> {
        if self.raw_device.is_null() {
            Err("PJRT_Device is null".to_string())
        } else {
            Ok(self.raw_device)
        }
    }

    pub fn description(&self) -> Result<PJRTDeviceDescriptionRef<'a>, String> {
        let raw = self.raw_checked()?;

        let get_desc = self
            .rt
            .api()
            .PJRT_Device_GetDescription
            .ok_or("PJRT_Device_GetDescription symbol not found")?;

        let mut get_desc_args = PJRT_Device_GetDescription_Args {
            struct_size: PJRT_Device_GetDescription_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            device_description: ptr::null_mut(),
        };
        let err = unsafe { get_desc(&mut get_desc_args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if get_desc_args.device_description.is_null() {
            return Err("PJRT_Device_GetDescription returned null device_description".to_string());
        }

        Ok(PJRTDeviceDescriptionRef::new(
            self.rt,
            get_desc_args.device_description,
        ))
    }

    pub fn is_addressable(&self) -> Result<bool, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Device_IsAddressable
            .ok_or("PJRT_Device_IsAddressable symbol not found")?;

        let mut args = PJRT_Device_IsAddressable_Args {
            struct_size: PJRT_Device_IsAddressable_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            is_addressable: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_addressable)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn local_hardware_id(&self) -> Result<i32, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Device_LocalHardwareId
            .ok_or("PJRT_Device_LocalHardwareId symbol not found")?;

        let mut args = PJRT_Device_LocalHardwareId_Args {
            struct_size: PJRT_Device_LocalHardwareId_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            local_hardware_id: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.local_hardware_id)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn addressable_memories(&self) -> Result<Vec<*mut PJRT_Memory>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Device_AddressableMemories
            .ok_or("PJRT_Device_AddressableMemories symbol not found")?;

        let mut args = PJRT_Device_AddressableMemories_Args {
            struct_size: PJRT_Device_AddressableMemories_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            memories: ptr::null(),
            num_memories: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_memories == 0 {
            return Ok(Vec::new());
        }
        if args.memories.is_null() {
            return Err(
                "PJRT_Device_AddressableMemories returned null memories with nonzero count"
                    .to_string(),
            );
        }

        let memories = unsafe { std::slice::from_raw_parts(args.memories, args.num_memories) };
        Ok(memories.to_vec())
    }

    pub fn addressable_memory_refs(&self) -> Result<Vec<PJRTMemory<'a>>, String> {
        Ok(self
            .addressable_memories()?
            .into_iter()
            .map(|raw| PJRTMemory::new(self.rt, raw))
            .collect())
    }

    pub fn default_memory(&self) -> Result<*mut PJRT_Memory, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Device_DefaultMemory
            .ok_or("PJRT_Device_DefaultMemory symbol not found")?;

        let mut args = PJRT_Device_DefaultMemory_Args {
            struct_size: PJRT_Device_DefaultMemory_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            memory: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.memory.is_null() {
            return Err("PJRT_Device_DefaultMemory returned null memory".to_string());
        }
        Ok(args.memory)
    }

    pub fn default_memory_ref(&self) -> Result<PJRTMemory<'a>, String> {
        Ok(PJRTMemory::new(self.rt, self.default_memory()?))
    }

    pub fn id(&self) -> Result<i32, String> {
        self.description()?.id()
    }

    pub fn kind(&self) -> Result<String, String> {
        self.description()?.kind()
    }

    pub fn process_index(&self) -> Result<i32, String> {
        self.description()?.process_index()
    }

    pub fn debug_string(&self) -> Result<String, String> {
        self.description()?.debug_string()
    }

    pub fn to_string(&self) -> Result<String, String> {
        self.description()?.to_string()
    }

    pub fn attributes(&self) -> Result<Vec<PJRTNamedAttribute>, String> {
        self.description()?.attributes()
    }

    // Backward compatibility with existing call sites.
    pub fn debug_error(&self) -> Result<String, String> {
        self.debug_string()
    }
}
