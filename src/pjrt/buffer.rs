use std::mem;
use std::ptr;
use std::slice::from_raw_parts;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PJRTBuffer<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_Buffer,
}

impl<'a> PJRTBuffer<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_Buffer) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_Buffer {
        self.raw
    }

    fn raw_checked(&self) -> Result<*mut PJRT_Buffer, String> {
        if self.raw.is_null() {
            Err("PJRT_Buffer is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn delete(&self) -> Result<(), String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Delete
            .ok_or("PJRT_Buffer_Delete symbol not found")?;

        let mut args = PJRT_Buffer_Delete_Args {
            struct_size: PJRT_Buffer_Delete_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn is_deleted(&self) -> Result<bool, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_IsDeleted
            .ok_or("PJRT_Buffer_IsDeleted symbol not found")?;

        let mut args = PJRT_Buffer_IsDeleted_Args {
            struct_size: PJRT_Buffer_IsDeleted_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            is_deleted: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_deleted)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn element_type(&self) -> Result<PJRT_Buffer_Type, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_ElementType
            .ok_or("PJRT_Buffer_ElementType symbol not found")?;

        let mut args = PJRT_Buffer_ElementType_Args {
            struct_size: PJRT_Buffer_ElementType_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            type_: PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.type_)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn dimensions(&self) -> Result<Vec<i64>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Dimensions
            .ok_or("PJRT_Buffer_Dimensions symbol not found")?;

        let mut args = PJRT_Buffer_Dimensions_Args {
            struct_size: PJRT_Buffer_Dimensions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            dims: ptr::null(),
            num_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_dims == 0 {
            return Ok(Vec::new());
        }
        if args.dims.is_null() {
            return Err("PJRT_Buffer_Dimensions returned null dims with nonzero num_dims".into());
        }

        Ok(unsafe { from_raw_parts(args.dims, args.num_dims).to_vec() })
    }

    pub fn unpadded_dimensions(&self) -> Result<Vec<i64>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_UnpaddedDimensions
            .ok_or("PJRT_Buffer_UnpaddedDimensions symbol not found")?;

        let mut args = PJRT_Buffer_UnpaddedDimensions_Args {
            struct_size: PJRT_Buffer_UnpaddedDimensions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            unpadded_dims: ptr::null(),
            num_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_dims == 0 {
            return Ok(Vec::new());
        }
        if args.unpadded_dims.is_null() {
            return Err(
                "PJRT_Buffer_UnpaddedDimensions returned null unpadded_dims with nonzero num_dims"
                    .into(),
            );
        }

        Ok(unsafe { from_raw_parts(args.unpadded_dims, args.num_dims).to_vec() })
    }

    pub fn dynamic_dimension_indices(&self) -> Result<Vec<usize>, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_DynamicDimensionIndices
            .ok_or("PJRT_Buffer_DynamicDimensionIndices symbol not found")?;

        let mut args = PJRT_Buffer_DynamicDimensionIndices_Args {
            struct_size: PJRT_Buffer_DynamicDimensionIndices_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            dynamic_dim_indices: ptr::null(),
            num_dynamic_dims: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_dynamic_dims == 0 {
            return Ok(Vec::new());
        }
        if args.dynamic_dim_indices.is_null() {
            return Err("PJRT_Buffer_DynamicDimensionIndices returned null indices with nonzero count".into());
        }

        Ok(unsafe { from_raw_parts(args.dynamic_dim_indices, args.num_dynamic_dims).to_vec() })
    }

    pub fn device(&self) -> Result<*mut PJRT_Device, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_Device
            .ok_or("PJRT_Buffer_Device symbol not found")?;

        let mut args = PJRT_Buffer_Device_Args {
            struct_size: PJRT_Buffer_Device_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            device: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            if args.device.is_null() {
                Err("PJRT_Buffer_Device returned null device".into())
            } else {
                Ok(args.device)
            }
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn on_device_size_in_bytes(&self) -> Result<usize, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_OnDeviceSizeInBytes
            .ok_or("PJRT_Buffer_OnDeviceSizeInBytes symbol not found")?;

        let mut args = PJRT_Buffer_OnDeviceSizeInBytes_Args {
            struct_size: PJRT_Buffer_OnDeviceSizeInBytes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            on_device_size_in_bytes: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.on_device_size_in_bytes)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn get_memory_layout(&self) -> Result<PJRT_Buffer_MemoryLayout, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Buffer_GetMemoryLayout
            .ok_or("PJRT_Buffer_GetMemoryLayout symbol not found")?;

        let mut args = PJRT_Buffer_GetMemoryLayout_Args {
            struct_size: PJRT_Buffer_GetMemoryLayout_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            buffer: raw,
            layout: unsafe { mem::zeroed() },
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.layout)
        } else {
            Err(error_to_string(self.rt.api(), err))
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
            extension_start: ptr::null_mut(),
            buffer: self.raw,
        };

        let err = unsafe { destroy(&mut args) };
        if !err.is_null() {

            let _ = error_to_string(self.rt.api(), err);
        }
    }
}
