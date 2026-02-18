use std::ffi::c_void;
use std::ptr;

use crate::pjrt::buffer::PJRTBuffer;
use crate::pjrt::device::PJRTDevice;
use crate::pjrt::event::PJRTEvent;
use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

pub struct PjrtHtoDeviceManager<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_AsyncHostToDeviceTransferManager,
}

impl<'a> PjrtHtoDeviceManager<'a> {
    pub(crate) fn new(
        rt: &'a PjrtRuntime,
        raw: *mut PJRT_AsyncHostToDeviceTransferManager,
    ) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_AsyncHostToDeviceTransferManager {
        self.raw
    }

    fn raw_checked(&self) -> Result<*mut PJRT_AsyncHostToDeviceTransferManager, String> {
        if self.raw.is_null() {
            Err("PJRT_AsyncHostToDeviceTransferManager is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn add_metadata(&self, metadata: &[PJRT_NamedValue]) -> Result<(), String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_AddMetadata
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_AddMetadata symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            transfer_metadata: if metadata.is_empty() {
                ptr::null()
            } else {
                metadata.as_ptr()
            },
            num_metadata: metadata.len(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn buffer_count(&self) -> Result<usize, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferCount
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_BufferCount symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_count: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.buffer_count)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn buffer_size(&self, buffer_index: i32) -> Result<usize, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferSize
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_BufferSize symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_index,
            buffer_size: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.buffer_size)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn device(&self) -> Result<*mut PJRT_Device, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_Device
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_Device symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_Device_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_Device_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            device_out: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.device_out.is_null() {
            return Err(
                "PJRT_AsyncHostToDeviceTransferManager_Device returned null device".to_string(),
            );
        }
        Ok(args.device_out)
    }

    pub fn device_ref(&self) -> Result<PJRTDevice<'a>, String> {
        Ok(PJRTDevice::new(self.rt, self.device()?))
    }

    pub fn retrieve_buffer(&self, buffer_index: i32) -> Result<*mut PJRT_Buffer, String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_index,
            buffer_out: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.buffer_out.is_null() {
            return Err(
                "PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer returned null buffer"
                    .to_string(),
            );
        }
        Ok(args.buffer_out)
    }

    pub fn retrieve_buffer_ref(&self, buffer_index: i32) -> Result<PJRTBuffer<'a>, String> {
        Ok(PJRTBuffer::new(
            self.rt,
            self.retrieve_buffer(buffer_index)?,
        ))
    }

    pub fn set_buffer_error(
        &self,
        buffer_index: i32,
        error_code: PJRT_Error_Code,
        error_message: &str,
    ) -> Result<(), String> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_SetBufferError
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_SetBufferError symbol not found")?;

        let error_message_bytes = error_message.as_bytes();
        let mut args = PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_index,
            error_code,
            error_message: if error_message_bytes.is_empty() {
                ptr::null()
            } else {
                error_message_bytes.as_ptr() as *const libc::c_char
            },
            error_message_size: error_message_bytes.len(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn transfer_data(
        &self,
        buffer_index: i32,
        data: &[u8],
        offset: i64,
        is_last_transfer: bool,
    ) -> Result<Option<PJRTEvent<'a>>, String> {
        let raw = self.raw_checked()?;
        if offset < 0 {
            return Err("transfer_data offset must be >= 0".to_string());
        }
        let transfer_size = i64::try_from(data.len())
            .map_err(|_| "transfer_data size does not fit i64".to_string())?;

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_TransferData
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_TransferData symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_TransferData_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_TransferData_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_index,
            data: if data.is_empty() {
                ptr::null()
            } else {
                data.as_ptr() as *const c_void
            },
            offset,
            transfer_size,
            is_last_transfer,
            done_with_h2d_transfer: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        Ok(if args.done_with_h2d_transfer.is_null() {
            None
        } else {
            Some(PJRTEvent::new(self.rt, args.done_with_h2d_transfer))
        })
    }

    pub fn transfer_literal(
        &self,
        buffer_index: i32,
        data: *const c_void,
        shape_dims: &[i64],
        shape_element_type: PJRT_Buffer_Type,
        shape_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<Option<PJRTEvent<'a>>, String> {
        let raw = self.raw_checked()?;
        if data.is_null() {
            return Err("transfer_literal data is null".to_string());
        }

        let f = self
            .rt
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_TransferLiteral
            .ok_or("PJRT_AsyncHostToDeviceTransferManager_TransferLiteral symbol not found")?;

        let mut args = PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args_STRUCT_SIZE
                as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: raw,
            buffer_index,
            data,
            shape_dims: if shape_dims.is_empty() {
                ptr::null()
            } else {
                shape_dims.as_ptr()
            },
            shape_num_dims: shape_dims.len(),
            shape_element_type,
            shape_layout: shape_layout.unwrap_or(ptr::null_mut()),
            done_with_h2d_transfer: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        Ok(if args.done_with_h2d_transfer.is_null() {
            None
        } else {
            Some(PJRTEvent::new(self.rt, args.done_with_h2d_transfer))
        })
    }
}

impl Drop for PjrtHtoDeviceManager<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_AsyncHostToDeviceTransferManager_Destroy else {
            return;
        };

        let mut args = PJRT_AsyncHostToDeviceTransferManager_Destroy_Args {
            struct_size: PJRT_AsyncHostToDeviceTransferManager_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            transfer_manager: self.raw,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}
