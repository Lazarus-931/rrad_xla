use crate::pjrt::buffer::PJRTBuffer;
use crate::pjrt::compile::PJRTCompiler;
use crate::pjrt::event::PJRTEvent;
use crate::pjrt::executable::PJRTLoadedExecutable;
use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt::topology_desc::{PJRTNamedAttribute, PJRTTopologyDescription};
use crate::pjrt_sys::*;
use std::ffi::c_void;
use std::ptr;

//raii wrapper for PJRT_Client

pub struct PJRTClient<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw_client: *mut PJRT_Client,
}

impl<'a> PJRTClient<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw_client: *mut PJRT_Client) -> Self {
        Self { rt, raw_client }
    }

    pub fn devices(&self) -> Result<Vec<*mut PJRT_Device>, String> {
        self.rt.client_devices(self.raw_client)
    }

    pub fn raw(&self) -> *mut PJRT_Client {
        self.raw_client
    }

    pub fn raw_checked(&self) -> Result<*mut PJRT_Client, String> {
        if self.raw_client.is_null() {
            Err("PJRT_Client is null".to_string())
        } else {
            Ok(self.raw_client)
        }
    }

    pub fn compiler(&self) -> PJRTCompiler<'a> {
        PJRTCompiler::new(self.rt, self.raw_client)
    }

    pub fn compile(
        &self,
        program_code: &str,
        format: &str,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'a>, String> {
        self.compiler()
            .compile(program_code, format, compile_options)
    }

    pub fn topology_description(&self) -> Result<PJRTTopologyDescription<'a>, String> {
        if self.raw_client.is_null() {
            return Err("PJRT_Client is null".to_string());
        }

        let f = self
            .rt
            .api()
            .PJRT_Client_TopologyDescription
            .ok_or("PJRT_Client_TopologyDescription symbol not found")?;

        let mut args = PJRT_Client_TopologyDescription_Args {
            struct_size: PJRT_Client_TopologyDescription_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client: self.raw_client,
            topology: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.topology.is_null() {
            return Err("PJRT_Client_TopologyDescription returned null topology".into());
        }

        Ok(PJRTTopologyDescription::new(self.rt, args.topology))
    }

    pub fn topology_platform_name(&self) -> Result<String, String> {
        self.topology_description()?.platform_name()
    }

    pub fn platform_version(&self) -> Result<String, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_PlatformVersion
            .ok_or("PJRT_Client_PlatformVersion symbol not found")?;

        let mut args = PJRT_Client_PlatformVersion_Args {
            struct_size: PJRT_Client_PlatformVersion_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            platform_version: ptr::null(),
            platform_version_size: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.platform_version.is_null() {
            if args.platform_version_size == 0 {
                return Ok(String::new());
            }
            return Err(
                "PJRT_Client_PlatformVersion returned null platform_version with nonzero size"
                    .to_string(),
            );
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(
                args.platform_version as *const u8,
                args.platform_version_size,
            )
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn topology_attributes(&self) -> Result<Vec<PJRTNamedAttribute>, String> {
        self.topology_description()?.attributes()
    }

    pub fn buffer_from_host_buffer(
        &self,
        data: *const c_void,
        element_type: PJRT_Buffer_Type,
        dims: &[i64],
        byte_strides: Option<&[i64]>,
        host_buffer_semantics: PJRT_HostBufferSemantics,
        device: Option<*mut PJRT_Device>,
    ) -> Result<(PJRTBuffer<'a>, Option<PJRTEvent<'a>>), String> {
        let client = self.raw_checked()?;

        if data.is_null() {
            return Err("host data pointer is null".to_string());
        }

        let buf_from_host = self
            .rt
            .api()
            .PJRT_Client_BufferFromHostBuffer
            .ok_or("PJRT_Client_BufferFromHostBuffer symbol not found")?;

        let (byte_strides_ptr, num_byte_strides) = match byte_strides {
            None => (ptr::null(), 0),
            Some(s) => {
                if s.len() != dims.len() {
                    return Err(format!(
                        "byte_strides len ({}) must match dims len ({})",
                        s.len(),
                        dims.len()
                    ));
                }
                (s.as_ptr(), s.len())
            }
        };

        let device = match device {
            Some(d) => d,
            None => self
                .devices()?
                .into_iter()
                .next()
                .ok_or("PJRT_Client has no devices")?,
        };

        let mut args = PJRT_Client_BufferFromHostBuffer_Args {
            struct_size: PJRT_Client_BufferFromHostBuffer_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            data,
            type_: element_type,
            dims: dims.as_ptr(),
            num_dims: dims.len(),
            byte_strides: byte_strides_ptr,
            num_byte_strides,
            host_buffer_semantics,
            device,
            memory: ptr::null_mut(),
            device_layout: ptr::null_mut(),
            done_with_host_buffer: ptr::null_mut(),
            buffer: ptr::null_mut(),
        };

        let err = unsafe { buf_from_host(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.buffer.is_null() {
            return Err(
                "PJRT_Client_BufferFromHostBuffer succeeded but returned null buffer".into(),
            );
        }

        let buffer = PJRTBuffer::new(self.rt, args.buffer);
        let event = if args.done_with_host_buffer.is_null() {
            None
        } else {
            Some(PJRTEvent::new(self.rt, args.done_with_host_buffer))
        };
        Ok((buffer, event))
    }

    pub fn buffer_from_host_slice_copy<T: Copy>(
        &self,
        data: &[T],
        element_type: PJRT_Buffer_Type,
        dims: &[i64],
        device: Option<*mut PJRT_Device>,
    ) -> Result<PJRTBuffer<'a>, String> {
        let (buf, done) = self.buffer_from_host_buffer(
            data.as_ptr().cast::<c_void>(),
            element_type,
            dims,
            None,
            PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall,
            device,
        )?;

        if let Some(ev) = done {
            // In this mode, it should be safe to drop the host memory after the call returns,
            // but we still await to avoid plugins that implement the transfer asynchronously.
            ev.await_ready()?;
        }

        Ok(buf)
    }

    // destory errors
    pub fn close(self) -> Result<(), String> {
        let raw = self.raw_client;
        let rt = self.rt;
        std::mem::forget(self);
        rt.destroy_client(raw)
    }

    pub fn platform_name(&self) -> Result<String, String> {
        let client = self.raw_checked()?;

        let platform = self
            .rt
            .api()
            .PJRT_Client_PlatformName
            .ok_or("PJRT_Client_PlatformName symbol not found")?;

        let mut args = PJRT_Client_PlatformName_Args {
            struct_size: PJRT_Client_PlatformName_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            platform_name: ptr::null(),
            platform_name_size: 0,
        };

        let err = unsafe { platform(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.platform_name.is_null() {
            if args.platform_name_size == 0 {
                return Ok(String::new());
            }
            return Err(
                "PJRT_Client_PlatformName returned null platform_name with nonzero size"
                    .to_string(),
            );
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(args.platform_name as *const u8, args.platform_name_size)
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

impl Drop for PJRTClient<'_> {
    fn drop(&mut self) {
        if self.raw_client.is_null() {
            return;
        }

        // Drop must not panic; best effort cleanup.
        let _ = self.rt.destroy_client(self.raw_client);
    }
}
