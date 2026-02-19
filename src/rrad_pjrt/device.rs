use std::ptr;

use crate::pjrt_sys::*;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::loader::error_to_string;
use crate::rrad_pjrt::loader::PjrtRuntime;
use crate::rrad_pjrt::memory::PJRTMemory;
use crate::rrad_pjrt::topology_desc::{PJRTDeviceDescriptionRef, PJRTNamedAttribute};

#[derive(Debug, Clone)]
pub struct PJRTDeviceMemoryStats {
    pub bytes_in_use: i64,
    pub peak_bytes_in_use: Option<i64>,
    pub num_allocs: Option<i64>,
    pub largest_alloc_size: Option<i64>,
    pub bytes_limit: Option<i64>,
    pub bytes_reserved: Option<i64>,
    pub peak_bytes_reserved: Option<i64>,
    pub bytes_reservable_limit: Option<i64>,
    pub largest_free_block_bytes: Option<i64>,
    pub pool_bytes: Option<i64>,
    pub peak_pool_bytes: Option<i64>,
}

pub struct PJRTAsyncTrackingEvent<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_AsyncTrackingEvent,
}

impl<'a> PJRTAsyncTrackingEvent<'a> {
    fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_AsyncTrackingEvent) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_AsyncTrackingEvent {
        self.raw
    }
}

impl Drop for PJRTAsyncTrackingEvent<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_AsyncTrackingEvent_Destroy else {
            return;
        };

        let mut args = PJRT_AsyncTrackingEvent_Destroy_Args {
            struct_size: PJRT_AsyncTrackingEvent_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            event: self.raw,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}

pub struct PJRTDevice<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Device,
}

impl<'a> PJRTDevice<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw_device: *mut PJRT_Device) -> Self {
        Self {
            rt,
            raw: raw_device,
        }
    }

    pub fn raw(&self) -> *mut PJRT_Device {
        self.raw
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Device, PJRTError> {
        if self.raw.is_null() {
            Err(self.error("PJRT_Device is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn description(&self) -> Result<PJRTDeviceDescriptionRef<'a>, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let get_desc = self.rt.api().PJRT_Device_GetDescription.ok_or_else(|| {
            self.error("PJRT_Device_GetDescription symbol not found")
                .to_string()
        })?;

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
            return Err(self
                .error("PJRT_Device_GetDescription returned null device_description")
                .to_string());
        }

        Ok(PJRTDeviceDescriptionRef::new(
            self.rt,
            get_desc_args.device_description,
        ))
    }

    pub fn is_addressable(&self) -> Result<bool, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self.rt.api().PJRT_Device_IsAddressable.ok_or_else(|| {
            self.error("PJRT_Device_IsAddressable symbol not found")
                .to_string()
        })?;

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

    pub fn memory_stats(&self) -> Result<PJRTDeviceMemoryStats, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self.rt.api().PJRT_Device_MemoryStats.ok_or_else(|| {
            self.error("PJRT_Device_MemoryStats symbol not found")
                .to_string()
        })?;

        let mut args = PJRT_Device_MemoryStats_Args {
            struct_size: PJRT_Device_MemoryStats_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            bytes_in_use: 0,
            peak_bytes_in_use: 0,
            peak_bytes_in_use_is_set: false,
            num_allocs: 0,
            num_allocs_is_set: false,
            largest_alloc_size: 0,
            largest_alloc_size_is_set: false,
            bytes_limit: 0,
            bytes_limit_is_set: false,
            bytes_reserved: 0,
            bytes_reserved_is_set: false,
            peak_bytes_reserved: 0,
            peak_bytes_reserved_is_set: false,
            bytes_reservable_limit: 0,
            bytes_reservable_limit_is_set: false,
            largest_free_block_bytes: 0,
            largest_free_block_bytes_is_set: false,
            pool_bytes: 0,
            pool_bytes_is_set: false,
            peak_pool_bytes: 0,
            peak_pool_bytes_is_set: false,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        Ok(PJRTDeviceMemoryStats {
            bytes_in_use: args.bytes_in_use,
            peak_bytes_in_use: args
                .peak_bytes_in_use_is_set
                .then_some(args.peak_bytes_in_use),
            num_allocs: args.num_allocs_is_set.then_some(args.num_allocs),
            largest_alloc_size: args
                .largest_alloc_size_is_set
                .then_some(args.largest_alloc_size),
            bytes_limit: args.bytes_limit_is_set.then_some(args.bytes_limit),
            bytes_reserved: args.bytes_reserved_is_set.then_some(args.bytes_reserved),
            peak_bytes_reserved: args
                .peak_bytes_reserved_is_set
                .then_some(args.peak_bytes_reserved),
            bytes_reservable_limit: args
                .bytes_reservable_limit_is_set
                .then_some(args.bytes_reservable_limit),
            largest_free_block_bytes: args
                .largest_free_block_bytes_is_set
                .then_some(args.largest_free_block_bytes),
            pool_bytes: args.pool_bytes_is_set.then_some(args.pool_bytes),
            peak_pool_bytes: args.peak_pool_bytes_is_set.then_some(args.peak_pool_bytes),
        })
    }

    pub fn poison_execution(
        &self,
        launch_id: i32,
        error_code: PJRT_Error_Code,
        error_message: &str,
    ) -> Result<bool, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self.rt.api().PJRT_Device_PoisonExecution.ok_or_else(|| {
            self.error("PJRT_Device_PoisonExecution symbol not found")
                .to_string()
        })?;

        let error_message_bytes = error_message.as_bytes();
        let mut args = PJRT_Device_PoisonExecution_Args {
            struct_size: PJRT_Device_PoisonExecution_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            launch_id,
            error_code,
            error_message: if error_message_bytes.is_empty() {
                ptr::null()
            } else {
                error_message_bytes.as_ptr() as *const libc::c_char
            },
            error_message_size: error_message_bytes.len(),
            poisoned: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.poisoned)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn create_async_tracking_event(
        &self,
        description: &str,
    ) -> Result<PJRTAsyncTrackingEvent<'a>, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self
            .rt
            .api()
            .PJRT_Device_CreateAsyncTrackingEvent
            .ok_or_else(|| {
                self.error("PJRT_Device_CreateAsyncTrackingEvent symbol not found")
                    .to_string()
            })?;

        let description_bytes = description.as_bytes();
        let mut args = PJRT_Device_CreateAsyncTrackingEvent_Args {
            struct_size: PJRT_Device_CreateAsyncTrackingEvent_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: raw,
            description: if description_bytes.is_empty() {
                ptr::null()
            } else {
                description_bytes.as_ptr() as *const libc::c_char
            },
            description_size: description_bytes.len(),
            event: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.event.is_null() {
            return Err(self
                .error("PJRT_Device_CreateAsyncTrackingEvent returned null event")
                .to_string());
        }

        Ok(PJRTAsyncTrackingEvent::new(self.rt, args.event))
    }

    pub fn local_hardware_id(&self) -> Result<i32, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self.rt.api().PJRT_Device_LocalHardwareId.ok_or_else(|| {
            self.error("PJRT_Device_LocalHardwareId symbol not found")
                .to_string()
        })?;

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

    pub fn addressable_memories(&self) -> Result<Vec<PJRTMemory<'a>>, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self
            .rt
            .api()
            .PJRT_Device_AddressableMemories
            .ok_or_else(|| {
                self.error("PJRT_Device_AddressableMemories symbol not found")
                    .to_string()
            })?;

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
            return Err(self
                .error("PJRT_Device_AddressableMemories returned null memories with nonzero count")
                .to_string());
        }

        let memories = unsafe { std::slice::from_raw_parts(args.memories, args.num_memories) };

        Ok(memories
            .iter()
            .copied()
            .map(|memory| PJRTMemory::new(self.rt, memory))
            .collect())
    }

    pub fn default_memory(&self) -> Result<*mut PJRT_Memory, String> {
        let raw = self.raw_checked().map_err(|e| e.to_string())?;

        let f = self.rt.api().PJRT_Device_DefaultMemory.ok_or_else(|| {
            self.error("PJRT_Device_DefaultMemory symbol not found")
                .to_string()
        })?;

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
            return Err(self
                .error("PJRT_Device_DefaultMemory returned null memory")
                .to_string());
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
