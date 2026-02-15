use std::ptr;
use std::slice::from_raw_parts;
use crate::pjrt::loader::PjrtRuntime;
use crate::pjrt_sys::*;

pub struct PJRTDevice<'a> {
    rt: &'a PjrtRuntime,
    raw_device: *mut PJRT_Device,
}

impl<'a> PJRTDevice<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw_device: *mut PJRT_Device) -> Self {
        Self { rt, raw_device }
    }

    pub fn id(&self) -> Result<i32, String> {
        if self.raw_device.is_null() {
            return Err("PJRT_Device is null".to_string());
        }

        let get_desc = self
            .rt
            .api()
            .PJRT_Device_GetDescription
            .ok_or("PJRT_Device_GetDescription symbol not found")?;
        let desc_id = self
            .rt
            .api()
            .PJRT_DeviceDescription_Id
            .ok_or("PJRT_DeviceDescription_Id symbol not found")?;

        let mut get_desc_args = PJRT_Device_GetDescription_Args {
            struct_size: PJRT_Device_GetDescription_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: self.raw_device,
            device_description: ptr::null_mut(),
        };
        let err = unsafe { get_desc(&mut get_desc_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }
        if get_desc_args.device_description.is_null() {
            return Err("PJRT_Device_GetDescription returned null device_description".to_string());
        }

        let mut id_args = PJRT_DeviceDescription_Id_Args {
            struct_size: PJRT_DeviceDescription_Id_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: get_desc_args.device_description,
            id: 0,
        };
        let err = unsafe { desc_id(&mut id_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }

        Ok(id_args.id)
    }

    pub fn kind(&self) -> Result<String, String> {
        if self.raw_device.is_null() {
            return Err("PJRT_Device is null".to_string());
        }

        let get_desc = self
            .rt
            .api()
            .PJRT_Device_GetDescription
            .ok_or("PJRT_Device_GetDescription symbol not found")?;

        let desc_kind = self
            .rt
            .api()
            .PJRT_DeviceDescription_Kind
            .ok_or("PJRT_DeviceDescription_Kind symbol not found")?;

        let mut get_desc_args = PJRT_Device_GetDescription_Args {
            struct_size: PJRT_Device_GetDescription_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: self.raw_device,
            device_description: ptr::null_mut(),
        };

        let mut kind_args = PJRT_DeviceDescription_Kind_Args {
            struct_size: PJRT_DeviceDescription_Kind_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: ptr::null_mut(),
            device_kind: ptr::null(),
            device_kind_size: 0,
        };

        let err = unsafe { get_desc(&mut get_desc_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }
        if get_desc_args.device_description.is_null() {
            return Err("PJRT_Device_GetDescription returned null device_description".to_string());
        }

        kind_args.device_description = get_desc_args.device_description;

        let err = unsafe { desc_kind(&mut kind_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }

        if kind_args.device_kind.is_null() {
            return Err("PJRT_DeviceDescription_Kind returned null device_kind".to_string());
        }
        if kind_args.device_kind_size == 0 {
            return Err("PJRT_DeviceDescription_Kind returned empty device_kind".to_string());
        }

        let bytes = unsafe {
            from_raw_parts(kind_args.device_kind as *const u8, kind_args.device_kind_size)
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }


    pub fn debug_error(&self) -> Result<String, String> {
        if self.raw_device.is_null() {
            return Err("PJRT_Device is null".to_string());
        }

        let get_desc = self
            .rt
            .api()
            .PJRT_Device_GetDescription
            .ok_or("PJRT_Device_GetDescription symbol not found")?;

        let desc_debug_string = self
            .rt
            .api()
            .PJRT_DeviceDescription_DebugString
            .ok_or("PJRT_DeviceDescription_DebugString symbol not found")?;

        let mut desc_args = PJRT_Device_GetDescription_Args {
            struct_size: PJRT_Device_GetDescription_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device: self.raw_device,
            device_description: ptr::null_mut(),
        };

        let err = unsafe { get_desc(&mut desc_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }
        if desc_args.device_description.is_null() {
            return Err("PJRT_Device_GetDescription returned null device_description".to_string());
        }

        let mut debug_args = PJRT_DeviceDescription_DebugString_Args {
            struct_size: PJRT_DeviceDescription_DebugString_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: desc_args.device_description,
            debug_string: ptr::null(),
            debug_string_size: 0,
        };

        let err = unsafe { desc_debug_string(&mut debug_args) };
        if !err.is_null() {
            return Err(crate::pjrt::loader::error_to_string(self.rt.api(), err));
        }

        if debug_args.debug_string.is_null() {
            return Err("PJRT_DeviceDescription_DebugString returned null debug_string".to_string());
        }

        let bytes = unsafe {
            from_raw_parts(debug_args.debug_string as *const u8, debug_args.debug_string_size)
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

}
