use std::mem;
use std::ptr;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use crate::pjrt_sys::*;
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::event::PJRTEvent;
use crate::rrad_pjrt::loader::PjrtRuntime;
use crate::rrad_pjrt::memory::PJRTMemory;
use crate::rrad_pjrt::topology_desc::PJRTNamedAttribute;

pub struct PJRTBuffer<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Buffer,
}

impl<'a> PJRTBuffer<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Buffer) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_Buffer {
        self.raw
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Buffer, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(self.error("PJRTBuffer is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn delete(&self) -> Result<(), PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Delete
            .ok_or_else(|| self.error("PJRT_Buffer_Delete symbol not found"))?;

        let mut args = PJRT_Buffer_Delete_Args {
            struct_size: PJRT_Buffer_Delete_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn is_deleted(&self) -> Result<bool, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_IsDeleted
            .ok_or_else(|| self.error("PJRT_Buffer_IsDeleted symbol not found"))?;

        let mut args = PJRT_Buffer_IsDeleted_Args {
            struct_size: PJRT_Buffer_IsDeleted_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            is_deleted: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_deleted)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn element_type(&self) -> Result<PJRT_Buffer_Type, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_ElementType
            .ok_or_else(|| self.error("PJRT_Buffer_ElementType symbol not found"))?;

        let mut args = PJRT_Buffer_ElementType_Args {
            struct_size: PJRT_Buffer_ElementType_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            type_: PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.type_)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn dimensions(&self) -> Result<Vec<i64>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Dimensions
            .ok_or_else(|| self.error("PJRT_Buffer_Dimensions symbol not found"))?;

        let mut args = PJRT_Buffer_Dimensions_Args {
            struct_size: PJRT_Buffer_Dimensions_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            dims: ptr::null(),
            num_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.num_dims == 0 {
            return Ok(Vec::new());
        }
        if args.dims.is_null() {
            return Err(
                self.error("PJRT_Buffer_Dimensions returned null dims with nonzero num_dims")
            );
        }

        Ok(unsafe { from_raw_parts(args.dims, args.num_dims).to_vec() })
    }

    pub fn unpadded_dimensions(&self) -> Result<Vec<i64>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_UnpaddedDimensions
            .ok_or_else(|| self.error("PJRT_Buffer_UnpaddedDimensions symbol not found"))?;

        let mut args = PJRT_Buffer_UnpaddedDimensions_Args {
            struct_size: PJRT_Buffer_UnpaddedDimensions_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            unpadded_dims: ptr::null(),
            num_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.num_dims == 0 {
            return Ok(Vec::new());
        }
        if args.unpadded_dims.is_null() {
            return Err(self.error(
                "PJRT_Buffer_UnpaddedDimensions returned null unpadded_dims with nonzero num_dims",
            ));
        }

        Ok(unsafe { from_raw_parts(args.unpadded_dims, args.num_dims).to_vec() })
    }

    pub fn dynamic_dimension_indices(&self) -> Result<Vec<usize>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_DynamicDimensionIndices
            .ok_or_else(|| self.error("PJRT_Buffer_DynamicDimensionIndices symbol not found"))?;

        let mut args = PJRT_Buffer_DynamicDimensionIndices_Args {
            struct_size: PJRT_Buffer_DynamicDimensionIndices_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            dynamic_dim_indices: ptr::null(),
            num_dynamic_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.num_dynamic_dims == 0 {
            return Ok(Vec::new());
        }
        if args.dynamic_dim_indices.is_null() {
            return Err(
                self.error(
                    "PJRT_Buffer_DynamicDimensionIndices returned null indices with nonzero count",
                ),
            );
        }

        Ok(unsafe { from_raw_parts(args.dynamic_dim_indices, args.num_dynamic_dims).to_vec() })
    }

    pub fn device(&self) -> Result<PJRTDevice<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Device
            .ok_or_else(|| self.error("PJRT_Buffer_Device symbol not found"))?;

        let mut args = PJRT_Buffer_Device_Args {
            struct_size: PJRT_Buffer_Device_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            device: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            if args.device.is_null() {
                Err(self.error("PJRT_Buffer_Device returned null device"))
            } else {
                Ok(PJRTDevice::new(self.rt, args.device))
            }
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn device_id(&self) -> Result<i32, PJRTError<'a>> {
        self.device()?.id()
    }

    pub fn device_kind(&self) -> Result<String, PJRTError<'a>> {
        self.device()?.kind()
    }

    pub fn device_process_index(&self) -> Result<i32, PJRTError<'a>> {
        self.device()?.process_index()
    }

    pub fn device_debug_string(&self) -> Result<String, PJRTError<'a>> {
        self.device()?.debug_string()
    }

    pub fn device_attributes(&self) -> Result<Vec<PJRTNamedAttribute>, PJRTError<'a>> {
        self.device()?.attributes()
    }

    pub fn on_device_size_in_bytes(&self) -> Result<usize, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_OnDeviceSizeInBytes
            .ok_or_else(|| self.error("PJRT_Buffer_OnDeviceSizeInBytes symbol not found"))?;

        let mut args = PJRT_Buffer_OnDeviceSizeInBytes_Args {
            struct_size: PJRT_Buffer_OnDeviceSizeInBytes_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            on_device_size_in_bytes: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.on_device_size_in_bytes)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn get_memory_layout(&self) -> Result<PJRT_Buffer_MemoryLayout, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_GetMemoryLayout
            .ok_or_else(|| self.error("PJRT_Buffer_GetMemoryLayout symbol not found"))?;

        let mut args = PJRT_Buffer_GetMemoryLayout_Args {
            struct_size: PJRT_Buffer_GetMemoryLayout_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            layout: unsafe { mem::zeroed() },
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.layout)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn ready_event(&self) -> Result<PJRTEvent<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_ReadyEvent
            .ok_or_else(|| self.error("PJRT_Buffer_ReadyEvent symbol not found"))?;

        let mut args = PJRT_Buffer_ReadyEvent_Args {
            struct_size: PJRT_Buffer_ReadyEvent_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            event: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.event.is_null() {
            return Err(self.error("PJRT_Buffer_ReadyEvent returned null event"));
        }

        Ok(PJRTEvent::new(self.rt, args.event))
    }

    pub fn to_host_buffer_async(&self, dst: &mut [u8]) -> Result<PJRTEvent<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_Buffer_ToHostBuffer
            .ok_or_else(|| self.error("PJRT_Buffer_ToHostBuffer symbol not found"))?;

        let mut args = PJRT_Buffer_ToHostBuffer_Args {
            struct_size: PJRT_Buffer_ToHostBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            src: raw,
            host_layout: null_mut(),
            dst: if dst.is_empty() {
                null_mut()
            } else {
                dst.as_mut_ptr().cast::<libc::c_void>()
            },
            dst_size: dst.len(),
            event: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.event.is_null() {
            return Err(self.error("PJRT_Buffer_ToHostBuffer returned null completion event"));
        }
        Ok(PJRTEvent::new(self.rt, args.event))
    }

    pub fn unsafe_pointer(&self) -> Result<usize, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_UnsafePointer
            .ok_or_else(|| self.error("PJRT_Buffer_UnsafePointer symbol not found"))?;

        let mut args = PJRT_Buffer_UnsafePointer_Args {
            struct_size: PJRT_Buffer_UnsafePointer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            buffer_pointer: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.buffer_pointer)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn opaque_device_memory_data_pointer(&self) -> Result<Option<*mut libc::c_void>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_OpaqueDeviceMemoryDataPointer
            .ok_or_else(|| self.error("PJRT_Buffer_OpaqueDeviceMemoryDataPointer symbol not found"))?;

        let mut args = PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args {
            struct_size: PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            device_memory_ptr: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok((!args.device_memory_ptr.is_null()).then_some(args.device_memory_ptr))
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn to_host_buffer_blocking(&self, dst: &mut [u8]) -> Result<(), PJRTError<'a>> {
        let event = self.to_host_buffer_async(dst)?;
        event.ok().map_err(|e| self.error(e))
    }

    pub fn copy_raw_to_host_async(
        &self,
        dst: &mut [u8],
        offset: i64,
    ) -> Result<PJRTEvent<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;
        if offset < 0 {
            return Err(self.error("offset must be >= 0"));
        }
        let transfer_size = i64::try_from(dst.len()).map_err(|_| {
            self.error("destination size does not fit i64 for CopyRawToHost")
        })?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_CopyRawToHost
            .ok_or_else(|| self.error("PJRT_Buffer_CopyRawToHost symbol not found"))?;

        let mut args = PJRT_Buffer_CopyRawToHost_Args {
            struct_size: PJRT_Buffer_CopyRawToHost_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            dst: if dst.is_empty() {
                null_mut()
            } else {
                dst.as_mut_ptr().cast::<libc::c_void>()
            },
            offset,
            transfer_size,
            event: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.event.is_null() {
            return Err(self.error("PJRT_Buffer_CopyRawToHost returned null completion event"));
        }
        Ok(PJRTEvent::new(self.rt, args.event))
    }

    pub fn copy_to_device(&self, device: &PJRTDevice) -> Result<PJRTBuffer<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;
        let dst_device = device.raw();
        if dst_device.is_null() {
            return Err(self.error("copy_to_device: destination device is null"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Buffer_CopyToDevice
            .ok_or_else(|| self.error("PJRT_Buffer_CopyToDevice symbol not found"))?;

        let mut args = PJRT_Buffer_CopyToDevice_Args {
            struct_size: PJRT_Buffer_CopyToDevice_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            dst_device,
            dst_buffer: null_mut(),
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else if args.dst_buffer.is_null() {
            Err(self.error("PJRT_Buffer_CopyToDevice returned null dst_buffer"))
        } else {
            Ok(PJRTBuffer::new(self.rt, args.dst_buffer))
        }
    }

    pub fn donate_with_control_dependency(
        &self,
        dependency: &PJRTEvent<'a>,
    ) -> Result<PJRTBuffer<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_DonateWithControlDependency
            .ok_or_else(|| self.error("PJRT_Buffer_DonateWithControlDependency symbol not found"))?;

        let mut args = PJRT_Buffer_DonateWithControlDependency_Args {
            struct_size: PJRT_Buffer_DonateWithControlDependency_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            callback_data: null_mut(),
            dependency_ready_callback: None,
            out_buffer: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }

        let callback = args.dependency_ready_callback.ok_or_else(|| {
            self.error(
                "PJRT_Buffer_DonateWithControlDependency returned null dependency_ready_callback",
            )
        })?;
        if args.out_buffer.is_null() {
            return Err(
                self.error("PJRT_Buffer_DonateWithControlDependency returned null out_buffer"),
            );
        }

        let dependency_status = dependency.ok().map_err(|e| self.error(e));
        let callback_message = match &dependency_status {
            Ok(()) => Vec::<u8>::new(),
            Err(message) => message.to_string().into_bytes(),
        };
        let mut callback_args = PJRT_Buffer_DonateWithControlDependency_Callback_Args {
            struct_size: PJRT_Buffer_DonateWithControlDependency_Callback_Args_STRUCT_SIZE as usize,
            callback_data: args.callback_data,
            error_code: if dependency_status.is_ok() {
                PJRT_Error_Code_PJRT_Error_Code_OK
            } else {
                PJRT_Error_Code_PJRT_Error_Code_UNKNOWN
            },
            error_message: if callback_message.is_empty() {
                ptr::null()
            } else {
                callback_message.as_ptr() as *const libc::c_char
            },
            error_message_size: callback_message.len(),
        };
        unsafe { callback(&mut callback_args) };

        dependency_status?;
        Ok(PJRTBuffer::new(self.rt, args.out_buffer))
    }

    pub fn copy_to_memory(&self, dst_memory: *mut PJRT_Memory) -> Result<PJRTBuffer<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;
        if dst_memory.is_null() {
            return Err(self.error("copy_to_memory: dst_memory is null"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Buffer_CopyToMemory
            .ok_or_else(|| self.error("PJRT_Buffer_CopyToMemory symbol not found"))?;

        let mut args = PJRT_Buffer_CopyToMemory_Args {
            struct_size: PJRT_Buffer_CopyToMemory_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            dst_memory,
            dst_buffer: null_mut(),
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else if args.dst_buffer.is_null() {
            Err(self.error("PJRT_Buffer_CopyToMemory returned null dst_buffer"))
        } else {
            Ok(PJRTBuffer::new(self.rt, args.dst_buffer))
        }
    }

    pub fn copy_raw_to_host_blocking(&self, dst: &mut [u8], offset: i64) -> Result<(), PJRTError<'a>> {
        let event = self.copy_raw_to_host_async(dst, offset)?;
        event.ok().map_err(|e| self.error(e))
    }

    pub fn copy_raw_to_host_future(
        &self,
        offset: i64,
        transfer_size: i64,
        callback_data: *mut libc::c_void,
        future_ready_callback: Option<
            unsafe extern "C" fn(args: *mut PJRT_Buffer_CopyRawToHostFuture_Callback_Args),
        >,
    ) -> Result<PJRTEvent<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;
        if offset < 0 {
            return Err(self.error("offset must be >= 0"));
        }
        if transfer_size < 0 {
            return Err(self.error("transfer_size must be >= 0"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Buffer_CopyRawToHostFuture
            .ok_or_else(|| self.error("PJRT_Buffer_CopyRawToHostFuture symbol not found"))?;

        let mut args = PJRT_Buffer_CopyRawToHostFuture_Args {
            struct_size: PJRT_Buffer_CopyRawToHostFuture_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            offset,
            transfer_size,
            event: null_mut(),
            callback_data,
            future_ready_callback,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.event.is_null() {
            return Err(self.error("PJRT_Buffer_CopyRawToHostFuture returned null event"));
        }
        Ok(PJRTEvent::new(self.rt, args.event))
    }

    pub fn is_on_cpu(&self) -> Result<bool, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_IsOnCpu
            .ok_or_else(|| self.error("PJRT_Buffer_IsOnCpu symbol not found"))?;

        let mut args = PJRT_Buffer_IsOnCpu_Args {
            struct_size: PJRT_Buffer_IsOnCpu_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            is_on_cpu: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_on_cpu)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn memory(&self) -> Result<PJRTMemory<'a>, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Memory
            .ok_or_else(|| self.error("PJRT_Buffer_Memory symbol not found"))?;

        let mut args = PJRT_Buffer_Memory_Args {
            struct_size: PJRT_Buffer_Memory_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
            memory: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.memory.is_null() {
            return Err(self.error("PJRT_Buffer_Memory returned null memory"));
        }

        Ok(PJRTMemory::new(self.rt, args.memory))
    }

    pub fn increase_external_ref(&self) -> Result<(), PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Buffer_IncreaseExternalReferenceCount
            .ok_or_else(|| self.error("PJRT_Buffer_IncreaseExternalReferenceCount symbol not found"))?;

        let mut args = PJRT_Buffer_IncreaseExternalReferenceCount_Args {
            struct_size: PJRT_Buffer_IncreaseExternalReferenceCount_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
        };

        let err = unsafe { func(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn decrease_external_ref(&self) -> Result<(), PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let func = self
            .rt
            .api()
            .PJRT_Buffer_DecreaseExternalReferenceCount
            .ok_or_else(|| self.error("PJRT_Buffer_DecreaseExternalReferenceCount symbol not found"))?;

        let mut args = PJRT_Buffer_DecreaseExternalReferenceCount_Args {
            struct_size: PJRT_Buffer_DecreaseExternalReferenceCount_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: raw,
        };

        let err = unsafe { func(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }
}

impl Drop for PJRTBuffer<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(destroy) = self.rt.api().PJRT_Buffer_Destroy else {
            return;
        };

        let mut args = PJRT_Buffer_Destroy_Args {
            struct_size: PJRT_Buffer_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            buffer: self.raw,
        };

        let err = unsafe { destroy(&mut args) };
        if !err.is_null() {
            let _ = PJRTError::new(self.rt, err);
        }
    }
}
