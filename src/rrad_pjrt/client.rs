use std::any::Any;
use crate::rrad_pjrt::buffer::PJRTBuffer;
use crate::rrad_pjrt::compile::PJRTCompiler;
use crate::rrad_pjrt::event::PJRTEvent;
use crate::rrad_pjrt::executable::PJRTLoadedExecutable;
use crate::rrad_pjrt::host_to_device_manager::PjrtHtoDeviceManager;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};
use crate::rrad_pjrt::memory::PJRTMemory;
use crate::rrad_pjrt::topology_desc::{PJRTNamedAttribute, PJRTTopologyDescription};
use crate::pjrt_sys::*;
use std::ffi::c_void;
use std::ptr;
use std::ptr::{null, null_mut};
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::utils::{BufferFromHostOptions, Shape};
use crate::rrad_pjrt::error::PJRTError;
//raii wrapper for PJRT_Client

pub struct PJRTClient<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_Client,
}

impl<'a> PJRTClient<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw_client: *mut PJRT_Client) -> Self {
        Self { rt, raw: raw_client }
    }

    pub fn devices(&self) -> Result<Vec<PJRTDevice<'a>>, String> {
        self.rt.client_devices(self.raw)
    }

    pub fn raw(&self) -> *mut PJRT_Client {
        self.raw
    }

    pub fn raw_checked(&self) -> Result<*mut PJRT_Client, String> {
        if self.raw.is_null() {
            Err("PJRT_Client is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn compiler(&self) -> PJRTCompiler<'a> {
        PJRTCompiler::new(self.rt, self.raw)
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

    pub fn compile_on_topology(
        &self,
        program: &PJRT_Program,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'a>, String> {
        let client = self.raw_checked()?;
        let topology = self.topology_description()?;
        topology.compile_and_load(client, program, compile_options, overridden_compile_options)
    }

    pub fn compile_on_topology_code(
        &self,
        program_code: &str,
        format: &str,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'a>, String> {
        let client = self.raw_checked()?;
        let topology = self.topology_description()?;
        topology.compile_and_load_code(
            client,
            program_code,
            format,
            compile_options,
            overridden_compile_options,
        )
    }

    pub fn topology_description(&self) -> Result<PJRTTopologyDescription<'a>, String> {
        if self.raw.is_null() {
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
            client: self.raw,
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

    pub fn fulfill_alias_buffer(
        &self,
        fulfill_alias_buffer_cb: *mut PJRT_FulfillAliasBufferCallback,
        buffer: Option<*mut PJRT_Buffer>,
        status_code: PJRT_Error_Code,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let client = self.raw_checked()?;

        if fulfill_alias_buffer_cb.is_null() {
            return Err("fulfill_alias_buffer_cb is null".to_string());
        }

        let f = self
            .rt
            .api()
            .PJRT_Client_FulfillAliasBuffer
            .ok_or("PJRT_Client_FulfillAliasBuffer symbol not found")?;

        let raw_buffer = buffer.unwrap_or(ptr::null_mut());
        if status_code == PJRT_Error_Code_PJRT_Error_Code_OK && raw_buffer.is_null() {
            return Err(
                "buffer must be non-null when status_code is PJRT_Error_Code_OK".to_string(),
            );
        }

        let error_message_bytes = if status_code == PJRT_Error_Code_PJRT_Error_Code_OK {
            &[][..]
        } else {
            error_message.map(str::as_bytes).unwrap_or(&[])
        };

        let mut args = PJRT_Client_FulfillAliasBuffer_Args {
            struct_size: PJRT_Client_FulfillAliasBuffer_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            buffer: raw_buffer,
            status_code,
            error_message: if error_message_bytes.is_empty() {
                ptr::null()
            } else {
                error_message_bytes.as_ptr() as *const libc::c_char
            },
            error_message_size: error_message_bytes.len(),
            fulfill_alias_buffer_cb,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn process_index(&self) -> Result<i32, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_ProcessIndex
            .ok_or("PJRT_Client_ProcessIndex symbol not found")?;

        let mut args = PJRT_Client_ProcessIndex_Args {
            struct_size: PJRT_Client_ProcessIndex_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            process_index: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.process_index)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn lookup_device(&'a self, id: i32) -> Result<PJRTDevice<'a>, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_LookupDevice
            .ok_or("PJRT_Client_LookupDevice symbol not found")?;

        let mut args = PJRT_Client_LookupDevice_Args {
            struct_size: PJRT_Client_LookupDevice_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            id,
            device: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.device.is_null() {
            return Err("PJRT_Client_LookupDevice returned null device".to_string());
        }
        let device = PJRTDevice::new(self.rt, args.device);
        Ok(device)
    }

    pub fn lookup_addressable_device(
        &'a self,
        local_hardware_id: i32,
    ) -> Result<PJRTDevice<'a>, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_LookupAddressableDevice
            .ok_or("PJRT_Client_LookupAddressableDevice symbol not found")?;

        let mut args = PJRT_Client_LookupAddressableDevice_Args {
            struct_size: PJRT_Client_LookupAddressableDevice_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            local_hardware_id,
            addressable_device: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.addressable_device.is_null() {
            return Err("PJRT_Client_LookupAddressableDevice returned null device".to_string());
        }
        let device = PJRTDevice::new(self.rt, args.addressable_device);
        Ok(device)
    }

    pub fn addressable_memories(&self) -> Result<Vec<PJRTMemory<'a>>, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_AddressableMemories
            .ok_or("PJRT_Client_AddressableMemories symbol not found")?;

        let mut args = PJRT_Client_AddressableMemories_Args {
            struct_size: PJRT_Client_AddressableMemories_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            addressable_memories: ptr::null(),
            num_addressable_memories: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_addressable_memories == 0 {
            return Ok(Vec::new());
        }
        if args.addressable_memories.is_null() {
            return Err(
                "PJRT_Client_AddressableMemories returned null memories with nonzero count"
                    .to_string(),
            );
        }

        let memories = unsafe {
            std::slice::from_raw_parts(args.addressable_memories, args.num_addressable_memories)
        };
        Ok(memories
            .to_vec()
            .iter()
            .copied()
            .map(|memory| PJRTMemory::new(self.rt, memory))
            .collect()
        )
    }

    pub fn create_buffers_for_async_host_to_device(

        &self,
        shape_specs: &mut [PJRT_ShapeSpec],
        device_layouts: &mut [*mut PJRT_Buffer_MemoryLayout],
        memory: Option<*mut PJRT_Memory>,
    ) -> Result<PjrtHtoDeviceManager<'a>, String> {
        let client = self.raw_checked()?;

        let function = self
            .rt
            .api()
            .PJRT_Client_CreateBuffersForAsyncHostToDevice
            .ok_or("PJRT_Client_CreateBuffersForAsyncHostToDevice symbol not found")?;

        let mut args = PJRT_Client_CreateBuffersForAsyncHostToDevice_Args {
            struct_size: PJRT_Client_CreateBuffersForAsyncHostToDevice_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            shape_specs: if shape_specs.is_empty() {
                ptr::null_mut()
            } else {
                shape_specs.as_mut_ptr()
            },
            num_shape_specs: shape_specs.len(),
            device_layouts: if device_layouts.is_empty() {
                ptr::null_mut()
            } else {
                device_layouts.as_mut_ptr()
            },
            num_device_layouts: device_layouts.len(),
            memory: memory.unwrap_or(ptr::null_mut()),
            transfer_manager: ptr::null_mut(),
        };

        let err = unsafe { function(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.transfer_manager.is_null() {
            return Err(
                "PJRT_Client_CreateBuffersForAsyncHostToDevice returned null transfer_manager"
                    .to_string(),
            );
        }

        Ok(PjrtHtoDeviceManager::new(self.rt, args.transfer_manager))
    }

      pub fn buffer_from_host_slice<T: Copy>(
      &self,
      host: &[T],
      shape: Shape<'_>,
      opts: BufferFromHostOptions<'a>,
  ) -> Result<PJRTBuffer<'a>, PJRTError> {
          let client = self.raw_checked()?;
          let function = self.rt
              .api().PJRT_Client_BufferFromHostBuffer
              .ok_or("PJRT_Client_BufferFromHostBuffer not found");

          let mut args = PJRT_Client_BufferFromHostBuffer_Args {
              struct_size: PJRT_Client_BufferFromHostBuffer_Args_STRUCT_SIZE as usize,
              extension_start: null_mut(),
              client,
              data: null(),
              type_: opts.semantics.type_id()



          }
      }

    pub fn dma_map(&self, data: *mut c_void, size: usize) -> Result<(), String> {
        let client = self.raw_checked()?;
        if size > 0 && data.is_null() {
            return Err("dma_map data pointer is null but size is nonzero".to_string());
        }

        let funct = self
            .rt
            .api()
            .PJRT_Client_DmaMap
            .ok_or("PJRT_Client_DmaMap symbol not found")?;

        let mut args = PJRT_Client_DmaMap_Args {
            struct_size: PJRT_Client_DmaMap_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            data,
            size,
        };

        let err = unsafe { funct(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(())
        }
    }

    pub fn dma_unmap(&self, data: *mut c_void) -> Result<(), String> {
        let client = self.raw_checked()?;
        if data.is_null() {
            return Err("dma_unmap data pointer is null".to_string());
        }

        let func = self
            .rt
            .api()
            .PJRT_Client_DmaUnmap
            .ok_or("PJRT_Client_DmaUnmap symbol not found")?;

        let mut args = PJRT_Client_DmaUnmap_Args {
            struct_size: PJRT_Client_DmaUnmap_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            data,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(())
        }
    }

    pub fn create_uninitialized_buffer(
        &self,
        element_type: PJRT_Buffer_Type,
    ) -> Result<PJRTBuffer<'a>, String> {
        let client = self.raw_checked()?;

        let funct = self
            .rt
            .api()
            .PJRT_Client_CreateUninitializedBuffer
            .ok_or("PJRT_Client_CreateUninitializedBuffer symbol not found")?;

        let mut args = PJRT_Client_CreateUninitializedBuffer_Args {
            struct_size: PJRT_Client_CreateUninitializedBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            shape_dims: null_mut(),
            shape_num_dims: 0,
            shape_element_type: element_type,
            shape_layout: null_mut(),
            device: null_mut(),
            memory: null_mut(),
            buffer: null_mut(),
        };

        let err = unsafe { funct(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err))
        } else {
            Ok(PJRTBuffer {
                rt: self.rt,
                raw: args.buffer,
            })
        }
    }

    pub fn create_view_of_device_buffer(
        &self,
        device_buffer_ptr: *mut c_void,
        dims: &[i64],
        element_type: PJRT_Buffer_Type,
        device: Option<*mut PJRT_Device>,
        memory: Option<*mut PJRT_Memory>,
        layout: Option<*mut PJRT_Buffer_MemoryLayout>,
        stream: isize,
        on_delete_callback: Option<
            unsafe extern "C" fn(device_buffer_ptr: *mut c_void, user_arg: *mut c_void),
        >,
        on_delete_callback_arg: *mut c_void,
    ) -> Result<PJRTBuffer<'a>, String> {
        let client = self.raw_checked()?;
        if device_buffer_ptr.is_null() {
            return Err("device_buffer_ptr is null".to_string());
        }
        if dims.is_empty() {
            return Err("dims must not be empty".to_string());
        }

        let funct = self
            .rt
            .api()
            .PJRT_Client_CreateViewOfDeviceBuffer
            .ok_or("PJRT_Client_CreateViewOfDeviceBuffer symbol not found")?;

        let device = match device {
            Some(d) => d,
            None => self
                .devices()?
                .into_iter()
                .next()
                .ok_or("PJRT_Client has no devices")?
                .raw(),
        };
        if device.is_null() {
            return Err("create_view_of_device_buffer device is null".to_string());
        }

        let mut args = PJRT_Client_CreateViewOfDeviceBuffer_Args {
            struct_size: PJRT_Client_CreateViewOfDeviceBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            device_buffer_ptr,
            dims: dims.as_ptr(),
            num_dims: dims.len(),
            element_type,
            layout: layout.unwrap_or(null_mut()),
            device,
            on_delete_callback,
            on_delete_callback_arg,
            stream,
            buffer: null_mut(),
            memory: memory.unwrap_or(null_mut()),
        };

        let err = unsafe { funct(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.buffer.is_null() {
            return Err("PJRT_Client_CreateViewOfDeviceBuffer returned null buffer".to_string());
        }

        Ok(PJRTBuffer::new(self.rt, args.buffer))
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
                .ok_or("PJRT_Client has no devices")?
                .raw(),
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

    pub fn create_alias_buffer(
        &self,
        shape_dims: &[i64],
        shape_element_type: PJRT_Buffer_Type,
        memory: Option<*mut PJRT_Memory>,
        shape_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<(PJRTBuffer<'a>, *mut PJRT_FulfillAliasBufferCallback), String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_CreateAliasBuffer
            .ok_or("PJRT_Client_CreateAliasBuffer symbol not found")?;

        let mut args = PJRT_Client_CreateAliasBuffer_Args {
            struct_size: PJRT_Client_CreateAliasBuffer_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            memory: memory.unwrap_or(ptr::null_mut()),
            shape_dims: if shape_dims.is_empty() {
                ptr::null()
            } else {
                shape_dims.as_ptr()
            },
            shape_num_dims: shape_dims.len(),
            shape_element_type,
            shape_layout: shape_layout.unwrap_or(ptr::null_mut()),
            alias_buffer: ptr::null_mut(),
            fulfill_alias_buffer_cb: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.alias_buffer.is_null() {
            return Err("PJRT_Client_CreateAliasBuffer returned null alias_buffer".to_string());
        }
        if args.fulfill_alias_buffer_cb.is_null() {
            return Err(
                "PJRT_Client_CreateAliasBuffer returned null fulfill_alias_buffer_cb".to_string(),
            );
        }

        Ok((
            PJRTBuffer::new(self.rt, args.alias_buffer),
            args.fulfill_alias_buffer_cb,
        ))
    }

    pub fn create_error_buffer(
        &self,
        error_code: PJRT_Error_Code,
        error_message: &str,
        shape_dims: &[i64],
        shape_element_type: PJRT_Buffer_Type,
        memory: Option<*mut PJRT_Memory>,
        shape_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<PJRTBuffer<'a>, String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_CreateErrorBuffer
            .ok_or("PJRT_Client_CreateErrorBuffer symbol not found")?;

        let error_message_bytes = error_message.as_bytes();
        let mut args = PJRT_Client_CreateErrorBuffer_Args {
            struct_size: PJRT_Client_CreateErrorBuffer_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            error_code,
            error_message: if error_message_bytes.is_empty() {
                ptr::null()
            } else {
                error_message_bytes.as_ptr() as *const libc::c_char
            },
            error_message_size: error_message_bytes.len(),
            shape_dims: if shape_dims.is_empty() {
                ptr::null()
            } else {
                shape_dims.as_ptr()
            },
            shape_num_dims: shape_dims.len(),
            shape_element_type,
            shape_layout: shape_layout.unwrap_or(ptr::null_mut()),
            memory: memory.unwrap_or(ptr::null_mut()),
            buffer: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.buffer.is_null() {
            return Err("PJRT_Client_CreateErrorBuffer returned null buffer".to_string());
        }

        Ok(PJRTBuffer::new(self.rt, args.buffer))
    }

    pub fn update_global_process_info(
        &self,
        process_infos: &mut [PJRT_ProcessInfo],
    ) -> Result<(), String> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_UpdateGlobalProcessInfo
            .ok_or("PJRT_Client_UpdateGlobalProcessInfo symbol not found")?;

        for info in process_infos.iter_mut() {
            if info.struct_size == 0 {
                info.struct_size = PJRT_ProcessInfo_STRUCT_SIZE as usize;
            }
        }

        let mut args = PJRT_Client_UpdateGlobalProcessInfo_Args {
            struct_size: PJRT_Client_UpdateGlobalProcessInfo_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            process_infos: if process_infos.is_empty() {
                ptr::null_mut()
            } else {
                process_infos.as_mut_ptr()
            },
            num_process_infos: process_infos.len(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn default_device_assignment(
        &self,
        num_replicas: i32,
        num_partitions: i32,
    ) -> Result<Vec<i32>, String> {
        if num_replicas < 0 || num_partitions < 0 {
            return Err("num_replicas and num_partitions must be >= 0".to_string());
        }

        let client = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_Client_DefaultDeviceAssignment
            .ok_or("PJRT_Client_DefaultDeviceAssignment symbol not found")?;

        let mut probe = PJRT_Client_DefaultDeviceAssignment_Args {
            struct_size: PJRT_Client_DefaultDeviceAssignment_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            num_replicas,
            num_partitions,
            default_assignment_size: 0,
            default_assignment: ptr::null_mut(),
        };

        let probe_err = unsafe { f(&mut probe) };
        let expected_size = (num_replicas as usize).saturating_mul(num_partitions as usize);

        if !probe_err.is_null() && expected_size == 0 {
            return Err(error_to_string(self.rt.api(), probe_err));
        }

        let mut out = vec![0i32; probe.default_assignment_size.max(expected_size)];
        if out.is_empty() {
            return Ok(out);
        }

        let mut args = PJRT_Client_DefaultDeviceAssignment_Args {
            struct_size: PJRT_Client_DefaultDeviceAssignment_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            num_replicas,
            num_partitions,
            default_assignment_size: out.len(),
            default_assignment: out.as_mut_ptr(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        out.truncate(args.default_assignment_size.min(out.len()));
        Ok(out)
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
        let raw = self.raw;
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
        if self.raw.is_null() {
            return;
        }

        // Drop must not panic; best effort cleanup.
        let _ = self.rt.destroy_client(self.raw);
    }
}
