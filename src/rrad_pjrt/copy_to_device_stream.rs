use std::ptr;

use crate::pjrt_sys::*;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};

pub struct PJRTCopyToDeviceStreamRef<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_CopyToDeviceStream,
}

impl<'a> PJRTCopyToDeviceStreamRef<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_CopyToDeviceStream) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_CopyToDeviceStream {
        self.raw
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_CopyToDeviceStream, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_CopyToDeviceStream is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn add_chunk(
        &self,
        chunk: *mut PJRT_Chunk,
        transfer_complete: Option<*mut PJRT_Event>,
    ) -> Result<(), String> {
        let stream = self.raw_checked().map_err(|e| e.to_string())?;
        if chunk.is_null() {
            return Err(self
                .error("PJRT_CopyToDeviceStream_AddChunk chunk is null")
                .to_string());
        }

        let func = self
            .rt
            .api()
            .PJRT_CopyToDeviceStream_AddChunk
            .ok_or_else(|| {
                self.error("PJRT_CopyToDeviceStream_AddChunk symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_CopyToDeviceStream_AddChunk_Args {
            struct_size: PJRT_CopyToDeviceStream_AddChunk_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            stream,
            chunk,
            transfer_complete: transfer_complete.unwrap_or(ptr::null_mut()),
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(())
        }
    }

    pub fn current_bytes(&self) -> Result<i64, String> {
        let stream = self.raw_checked().map_err(|e| e.to_string())?;

        let func = self
            .rt
            .api()
            .PJRT_CopyToDeviceStream_CurrentBytes
            .ok_or_else(|| {
                self.error("PJRT_CopyToDeviceStream_CurrentBytes symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_CopyToDeviceStream_CurrentBytes_Args {
            struct_size: PJRT_CopyToDeviceStream_CurrentBytes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            stream,
            current_bytes: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(args.current_bytes)
        }
    }

    pub fn total_bytes(&self) -> Result<i64, String> {
        let stream = self.raw_checked().map_err(|e| e.to_string())?;

        let func = self
            .rt
            .api()
            .PJRT_CopyToDeviceStream_TotalBytes
            .ok_or_else(|| {
                self.error("PJRT_CopyToDeviceStream_TotalBytes symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_CopyToDeviceStream_TotalBytes_Args {
            struct_size: PJRT_CopyToDeviceStream_TotalBytes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            stream,
            total_bytes: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(args.total_bytes)
        }
    }

    pub fn granule_size(&self) -> Result<i64, String> {
        let stream = self.raw_checked().map_err(|e| e.to_string())?;

        let func = self
            .rt
            .api()
            .PJRT_CopyToDeviceStream_GranuleSize
            .ok_or_else(|| {
                self.error("PJRT_CopyToDeviceStream_GranuleSize symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_CopyToDeviceStream_GranuleSize_Args {
            struct_size: PJRT_CopyToDeviceStream_GranuleSize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            stream,
            granule_size_in_bytes: 0,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(args.granule_size_in_bytes)
        }
    }

    pub fn granul_size(&self) -> Result<i64, String> {
        self.granule_size()
    }
}

impl Drop for PJRTCopyToDeviceStreamRef<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_CopyToDeviceStream_Destroy else {
            return;
        };

        let mut args = PJRT_CopyToDeviceStream_Destroy_Args {
            struct_size: PJRT_CopyToDeviceStream_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            stream: self.raw,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            let _ = error_to_string(self.rt.api(), err);
        }

        self.raw = ptr::null_mut();
    }
}
