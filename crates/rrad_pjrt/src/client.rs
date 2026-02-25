use crate::ffi::pjrt_sys::*;
use crate::buffer::PJRTBuffer;
use crate::compile::PJRTCompiler;
use crate::device::PJRTDevice;
use crate::error::PJRTError;
use crate::event::PJRTEvent;
use crate::executable::PJRTLoadedExecutable;
use crate::host_to_device_manager::PjrtHtoDeviceManager;
use crate::loader::PjrtRuntime;
use crate::memory::PJRTMemory;
use crate::topology_desc::{PJRTNamedAttribute, PJRTTopologyDescription};
use crate::utils::{BufferFromHostOptions, PJRTShapeSpec, Shape};
use std::ffi::c_void;
use std::ptr;
use std::ptr::null_mut;
pub struct PJRTClient<'rt> {
    pub rt: &'rt PjrtRuntime,
    pub raw: *mut PJRT_Client,
}
/// Client wrapper for [`crate::ffi::pjrt_sys::PJRT_Client`] which is the main part needed to
/// interact with pjrt component such as [`crate::ffi::pjrt_sys::PJRT_Device`] or [`crate::ffi::pjrt_sys::PJRT_Buffer`].

/// Handles the use of all components.
impl<'rt> PJRTClient<'rt> {
    /// Constructs local error definitions for client-related errors.
    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'rt> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    pub fn devices(&self) -> Result<Vec<PJRTDevice<'rt, '_>>, PJRTError<'rt>> {
        self.rt.client_devices(self.raw)
    }

    pub fn raw(&self) -> *mut PJRT_Client {
        self.raw
    }

    pub fn raw_checked(&self) -> Result<*mut PJRT_Client, PJRTError<'rt>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_Client is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn compiler(&self) -> PJRTCompiler<'rt, '_> {
        PJRTCompiler::new(self.rt, self.raw)
    }

    pub fn compile(
        &self,
        program_code: &str,
        format: &str,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'rt, '_>, PJRTError<'rt>> {
        {
            let compiler = self.compiler();
            compiler.compile(program_code, format, compile_options)
        }
    }

    pub fn compile_on_topology(
        &self,
        program: &PJRT_Program,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'rt, '_>, PJRTError<'rt>> {
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
    ) -> Result<PJRTLoadedExecutable<'rt, '_>, PJRTError<'rt>> {
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

    pub fn topology_description(&self) -> Result<PJRTTopologyDescription<'rt, '_>, PJRTError<'rt>> {
        if self.raw.is_null() {
            return Err(self.error("PJRT_Client is null"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Client_TopologyDescription
            .ok_or(self.error("PJRT_Client_TopologyDescription symbol not found"))?;

        let mut args = PJRT_Client_TopologyDescription_Args {
            struct_size: PJRT_Client_TopologyDescription_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client: self.raw,
            topology: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.topology.is_null() {
            return Err(self.error("PJRT_Client_TopologyDescription returned null topology"));
        }

        Ok(PJRTTopologyDescription::new(self.rt, args.topology))
    }

    pub fn topology_platform_name(&self) -> Result<String, PJRTError<'rt>> {
        self.topology_description()?.platform_name()
    }

    pub fn platform_version(&self) -> Result<String, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_PlatformVersion
            .ok_or(self.error("PJRT_Client_PlatformVersion symbol not found"))?;

        let mut args = PJRT_Client_PlatformVersion_Args {
            struct_size: PJRT_Client_PlatformVersion_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            platform_version: ptr::null(),
            platform_version_size: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.platform_version.is_null() {
            if args.platform_version_size == 0 {
                return Ok(String::new());
            }
            return Err(self.error(
                "PJRT_Client_PlatformVersion returned null platform_version with nonzero size",
            ));
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(
                args.platform_version as *const u8,
                args.platform_version_size,
            )
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn topology_attributes(&self) -> Result<Vec<PJRTNamedAttribute>, PJRTError<'rt>> {
        self.topology_description()?.attributes()
    }

    pub fn fulfill_alias_buffer(
        &self,
        fulfill_alias_buffer_cb: *mut PJRT_FulfillAliasBufferCallback,
        buffer: Option<*mut PJRT_Buffer>,
        status_code: PJRT_Error_Code,
        error_message: Option<&str>,
    ) -> Result<(), PJRTError<'rt>> {
        let client = self.raw_checked()?;

        if fulfill_alias_buffer_cb.is_null() {
            return Err(self.error("fulfill_alias_buffer_cb is null"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Client_FulfillAliasBuffer
            .ok_or(self.error("PJRT_Client_FulfillAliasBuffer symbol not found"))?;

        let raw_buffer = buffer.unwrap_or(null_mut());
        if status_code == PJRT_Error_Code_PJRT_Error_Code_OK && raw_buffer.is_null() {
            return Err(
                self.error("buffer must be non-null when status_code is PJRT_Error_Code_OK")
            );
        }

        let error_message_bytes = if status_code == PJRT_Error_Code_PJRT_Error_Code_OK {
            &[][..]
        } else {
            error_message.map(str::as_bytes).unwrap_or(&[])
        };

        let mut args = PJRT_Client_FulfillAliasBuffer_Args {
            struct_size: PJRT_Client_FulfillAliasBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
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
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn process_index(&self) -> Result<i32, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_ProcessIndex
            .ok_or(self.error("PJRT_Client_ProcessIndex symbol not found"))?;

        let mut args = PJRT_Client_ProcessIndex_Args {
            struct_size: PJRT_Client_ProcessIndex_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            process_index: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.process_index)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn lookup_device(&self, id: i32) -> Result<PJRTDevice<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_LookupDevice
            .ok_or(self.error("PJRT_Client_LookupDevice symbol not found"))?;

        let mut args = PJRT_Client_LookupDevice_Args {
            struct_size: PJRT_Client_LookupDevice_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            id,
            device: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.device.is_null() {
            return Err(self.error("PJRT_Client_LookupDevice returned null device"));
        }
        Ok(PJRTDevice::new(self.rt, args.device))
    }

    pub fn lookup_addressable_device(
        &self,
        local_hardware_id: i32,
    ) -> Result<PJRTDevice<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_LookupAddressableDevice
            .ok_or(self.error("PJRT_Client_LookupAddressableDevice symbol not found"))?;

        let mut args = PJRT_Client_LookupAddressableDevice_Args {
            struct_size: PJRT_Client_LookupAddressableDevice_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            local_hardware_id,
            addressable_device: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.addressable_device.is_null() {
            return Err(self.error("PJRT_Client_LookupAddressableDevice returned null device"));
        }
        Ok(PJRTDevice::new(self.rt, args.addressable_device))
    }

    pub fn addressable_memories(&self) -> Result<Vec<PJRTMemory<'rt, '_>>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_AddressableMemories
            .ok_or(self.error("PJRT_Client_AddressableMemories symbol not found"))?;

        let mut args = PJRT_Client_AddressableMemories_Args {
            struct_size: PJRT_Client_AddressableMemories_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            addressable_memories: ptr::null(),
            num_addressable_memories: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.num_addressable_memories == 0 {
            return Ok(Vec::new());
        }
        if args.addressable_memories.is_null() {
            return Err(self.error(
                "PJRT_Client_AddressableMemories returned null memories with nonzero count",
            ));
        }

        let memories = unsafe {
            std::slice::from_raw_parts(args.addressable_memories, args.num_addressable_memories)
        };
        Ok(memories
            .to_vec()
            .iter()
            .copied()
            .map(|memory| PJRTMemory::new(self.rt, memory))
            .collect())
    }

    pub fn create_buffers_for_async_host_to_device(
        &self,
        shape_specs: &mut [PJRT_ShapeSpec],
        device_layouts: &mut [*mut PJRT_Buffer_MemoryLayout],
        memory: Option<PJRTMemory<'rt, '_>>,
    ) -> Result<PjrtHtoDeviceManager<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let function = self
            .rt
            .api()
            .PJRT_Client_CreateBuffersForAsyncHostToDevice
            .ok_or(self.error("PJRT_Client_CreateBuffersForAsyncHostToDevice symbol not found"))?;

        let mut args = PJRT_Client_CreateBuffersForAsyncHostToDevice_Args {
            struct_size: PJRT_Client_CreateBuffersForAsyncHostToDevice_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            shape_specs: if shape_specs.is_empty() {
                null_mut()
            } else {
                shape_specs.as_mut_ptr()
            },
            num_shape_specs: shape_specs.len(),
            device_layouts: if device_layouts.is_empty() {
                null_mut()
            } else {
                device_layouts.as_mut_ptr()
            },
            num_device_layouts: device_layouts.len(),
            memory: memory.as_ref().map_or(null_mut(), |m| m.raw),
            transfer_manager: null_mut(),
        };

        let err = unsafe { function(&mut args) };

        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.transfer_manager.is_null() {
            return Err(self.error(
                "PJRT_Client_CreateBuffersForAsyncHostToDevice returned null transfer_manager",
            ));
        }

        Ok(PjrtHtoDeviceManager::new(self.rt, args.transfer_manager))
    }

    pub fn create_buffers_for_async_host_to_device_specs(
        &self,
        shape_specs: &[PJRTShapeSpec],
        device_layouts: &mut [*mut PJRT_Buffer_MemoryLayout],
        memory: Option<PJRTMemory<'rt, '_>>,
    ) -> Result<PjrtHtoDeviceManager<'rt, '_>, PJRTError<'rt>> {
        let mut raw_specs: Vec<PJRT_ShapeSpec> =
            shape_specs.iter().map(PJRTShapeSpec::to_raw).collect();

        self.create_buffers_for_async_host_to_device(&mut raw_specs, device_layouts, memory)
    }

    pub fn buffer_from_host_slice<T: Copy>(
        &self,
        host: &[T],
        shape: Shape<'_>,
        opts: BufferFromHostOptions<'rt, '_, '_>,
    ) -> Result<PJRTBuffer<'rt, '_>, PJRTError<'rt>> {
        if host.is_empty() {
            return Err(self.error("host slice must not be empty"));
        }

        let semantics = match opts.semantics {
            crate::utils::HostBufferSemantics::ImmutableOnlyDuringCalls => {
                PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall
            }
            crate::utils::HostBufferSemantics::ImmutableUntilTransferCompletes => {
                PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes
            }
            crate::utils::HostBufferSemantics::ImmutableZeroCopy => {
                PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy
            }
            crate::utils::HostBufferSemantics::MutableZeroCopy => {
                PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kMutableZeroCopy
            }
        };

        let device = opts.device;
        let memory = opts.memory;
        let layout = opts
            .layout
            .map(|l| l as *const PJRT_Buffer_MemoryLayout as *mut PJRT_Buffer_MemoryLayout);

        let (buffer, done) = self.buffer_from_host_buffer(
            host.as_ptr().cast::<c_void>(),
            shape.element_type,
            shape.dims,
            None,
            semantics,
            device,
            memory,
            layout,
        )?;

        if let Some(ev) = done {
            ev.await_ready()?;
            ev.ok()?;
        }

        Ok(buffer)
    }

    pub fn dma_map(&self, data: *mut c_void, size: usize) -> Result<(), PJRTError<'rt>> {
        let client = self.raw_checked()?;
        if size > 0 && data.is_null() {
            return Err(self.error("dma_map data pointer is null but size is nonzero"));
        }

        let funct = self
            .rt
            .api()
            .PJRT_Client_DmaMap
            .ok_or(self.error("PJRT_Client_DmaMap symbol not found"))?;

        let mut args = PJRT_Client_DmaMap_Args {
            struct_size: PJRT_Client_DmaMap_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            data,
            size,
        };

        let err = unsafe { funct(&mut args) };

        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else {
            Ok(())
        }
    }

    pub fn dma_unmap(&self, data: *mut c_void) -> Result<(), PJRTError<'rt>> {
        let client = self.raw_checked()?;
        if data.is_null() {
            return Err(self.error("dma_unmap data pointer is null"));
        }

        let func = self
            .rt
            .api()
            .PJRT_Client_DmaUnmap
            .ok_or(self.error("PJRT_Client_DmaUnmap symbol not found"))?;

        let mut args = PJRT_Client_DmaUnmap_Args {
            struct_size: PJRT_Client_DmaUnmap_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            data,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(PJRTError::new(self.rt, err))
        } else {
            Ok(())
        }
    }

    pub fn create_uninitialized_buffer(
        &self,
        element_type: PJRT_Buffer_Type,
    ) -> Result<PJRTBuffer<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let funct = self
            .rt
            .api()
            .PJRT_Client_CreateUninitializedBuffer
            .ok_or(self.error("PJRT_Client_CreateUninitializedBuffer symbol not found"))?;

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
            Err(PJRTError::new(self.rt, err))
        } else if args.buffer.is_null() {
            Err(self.error("PJRT_Client_CreateUninitializedBuffer returned null buffer"))
        } else {
            Ok(PJRTBuffer::new(self.rt, args.buffer))
        }
    }

    pub fn create_view_of_device_buffer(
        &self,
        device_buffer_ptr: *mut c_void,
        dims: &[i64],
        element_type: PJRT_Buffer_Type,
        device: Option<PJRTDevice<'rt, '_>>,
        memory: Option<PJRTMemory<'rt, '_>>,
        layout: Option<*mut PJRT_Buffer_MemoryLayout>,
        stream: isize,
        on_delete_callback: Option<
            unsafe extern "C" fn(device_buffer_ptr: *mut c_void, user_arg: *mut c_void),
        >,
        on_delete_callback_arg: *mut c_void,
    ) -> Result<PJRTBuffer<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;
        if device_buffer_ptr.is_null() {
            return Err(self.error("device_buffer_ptr is null"));
        }
        if dims.is_empty() {
            return Err(self.error("dims must not be empty"));
        }

        let funct = self
            .rt
            .api()
            .PJRT_Client_CreateViewOfDeviceBuffer
            .ok_or(self.error("PJRT_Client_CreateViewOfDeviceBuffer symbol not found"))?;

        let raw_device = match device {
            Some(d) => d.raw(),
            None => {
                let devices = self.devices()?;
                let first = devices
                    .first()
                    .ok_or_else(|| self.error("PJRT_Client has no devices"))?;
                first.raw()
            }
        };
        if raw_device.is_null() {
            return Err(self.error("create_view_of_device_buffer device is null"));
        }
        let raw_layout = match layout {
            Some(l) if l.is_null() => {
                return Err(self.error("create_view_of_device_buffer layout is null"));
            }
            Some(l) => l,
            None => null_mut(),
        };

        let mut args = PJRT_Client_CreateViewOfDeviceBuffer_Args {
            struct_size: PJRT_Client_CreateViewOfDeviceBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            device_buffer_ptr,
            dims: if dims.is_empty() {
                ptr::null()
            } else {
                dims.as_ptr()
            },
            num_dims: dims.len(),
            element_type,
            layout: raw_layout,
            device: raw_device,
            on_delete_callback,
            on_delete_callback_arg,
            stream,
            buffer: null_mut(),
            memory: memory.as_ref().map_or(null_mut(), |m| m.raw),
        };

        let err = unsafe { funct(&mut args) };

        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.buffer.is_null() {
            return Err(self.error("PJRT_Client_CreateViewOfDeviceBuffer returned null buffer"));
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
        device: Option<PJRTDevice<'rt, '_>>,
        memory: Option<PJRTMemory<'rt, '_>>,
        device_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<(PJRTBuffer<'rt, '_>, Option<PJRTEvent<'rt>>), PJRTError<'rt>> {
        let client = self.raw_checked()?;

        if data.is_null() {
            return Err(self.error("host data pointer is null"));
        }

        let buf_from_host = self
            .rt
            .api()
            .PJRT_Client_BufferFromHostBuffer
            .ok_or(self.error("PJRT_Client_BufferFromHostBuffer symbol not found"))?;

        let (byte_strides_ptr, num_byte_strides) = match byte_strides {
            None => (ptr::null(), 0),
            Some(s) => {
                if s.len() != dims.len() {
                    return Err(self.error(format!(
                        "byte_strides len ({}) must match dims len ({})",
                        s.len(),
                        dims.len()
                    )));
                }
                if s.is_empty() {
                    (ptr::null(), 0)
                } else {
                    (s.as_ptr(), s.len())
                }
            }
        };

        let raw_device = match device {
            Some(d) => d.raw(),
            None => {
                let devices = self.devices()?;
                let first = devices
                    .first()
                    .ok_or_else(|| self.error("PJRT_Client has no devices"))?;
                first.raw()
            }
        };
        if raw_device.is_null() {
            return Err(self.error("buffer_from_host_buffer device is null"));
        }

        let memory = memory.map_or(null_mut(), |m| m.raw);

        let device_layout = match device_layout {
            Some(l) if l.is_null() => {
                return Err(self.error("buffer_from_host_buffer device_layout is null"));
            }
            Some(l) => l,
            None => null_mut(),
        };

        let mut args = PJRT_Client_BufferFromHostBuffer_Args {
            struct_size: PJRT_Client_BufferFromHostBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            data,
            type_: element_type,
            dims: if dims.is_empty() {
                ptr::null()
            } else {
                dims.as_ptr()
            },
            num_dims: dims.len(),
            byte_strides: byte_strides_ptr,
            num_byte_strides,
            host_buffer_semantics,
            device: raw_device,
            memory,
            device_layout,
            done_with_host_buffer: null_mut(),
            buffer: null_mut(),
        };

        let err = unsafe { buf_from_host(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.buffer.is_null() {
            return Err(
                self.error("PJRT_Client_BufferFromHostBuffer succeeded but returned null buffer")
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
        memory: Option<PJRTMemory<'rt, '_>>,
        shape_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<(PJRTBuffer<'rt, '_>, *mut PJRT_FulfillAliasBufferCallback), PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_CreateAliasBuffer
            .ok_or(self.error("PJRT_Client_CreateAliasBuffer symbol not found"))?;
        let raw_shape_layout = match shape_layout {
            Some(l) if l.is_null() => {
                return Err(self.error("create_alias_buffer shape_layout is null"));
            }
            Some(l) => l,
            None => null_mut(),
        };

        let mut args = PJRT_Client_CreateAliasBuffer_Args {
            struct_size: PJRT_Client_CreateAliasBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            memory: memory.map_or(null_mut(), |m| m.raw),
            shape_dims: if shape_dims.is_empty() {
                ptr::null()
            } else {
                shape_dims.as_ptr()
            },
            shape_num_dims: shape_dims.len(),
            shape_element_type,
            shape_layout: raw_shape_layout,
            alias_buffer: null_mut(),
            fulfill_alias_buffer_cb: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.alias_buffer.is_null() {
            return Err(self.error("PJRT_Client_CreateAliasBuffer returned null alias_buffer"));
        }
        if args.fulfill_alias_buffer_cb.is_null() {
            return Err(
                self.error("PJRT_Client_CreateAliasBuffer returned null fulfill_alias_buffer_cb")
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
        memory: Option<PJRTMemory<'rt, '_>>,
        shape_layout: Option<*mut PJRT_Buffer_MemoryLayout>,
    ) -> Result<PJRTBuffer<'rt, '_>, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_CreateErrorBuffer
            .ok_or(self.error("PJRT_Client_CreateErrorBuffer symbol not found"))?;
        let raw_shape_layout = match shape_layout {
            Some(l) if l.is_null() => {
                return Err(self.error("create_error_buffer shape_layout is null"));
            }
            Some(l) => l,
            None => null_mut(),
        };

        let error_message_bytes = error_message.as_bytes();
        let mut args = PJRT_Client_CreateErrorBuffer_Args {
            struct_size: PJRT_Client_CreateErrorBuffer_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
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
            shape_layout: raw_shape_layout,
            memory: memory.map_or(null_mut(), |m| m.raw),
            buffer: null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.buffer.is_null() {
            return Err(self.error("PJRT_Client_CreateErrorBuffer returned null buffer"));
        }

        Ok(PJRTBuffer::new(self.rt, args.buffer))
    }

    pub fn update_global_process_info(
        &self,
        process_infos: &mut [PJRT_ProcessInfo],
    ) -> Result<(), PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_Client_UpdateGlobalProcessInfo
            .ok_or(self.error("PJRT_Client_UpdateGlobalProcessInfo symbol not found"))?;

        for info in process_infos.iter_mut() {
            if info.struct_size == 0 {
                info.struct_size = PJRT_ProcessInfo_STRUCT_SIZE as usize;
            }
        }

        let mut args = PJRT_Client_UpdateGlobalProcessInfo_Args {
            struct_size: PJRT_Client_UpdateGlobalProcessInfo_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            process_infos: if process_infos.is_empty() {
                null_mut()
            } else {
                process_infos.as_mut_ptr()
            },
            num_process_infos: process_infos.len(),
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn default_device_assignment(
        &self,
        num_replicas: i32,
        num_partitions: i32,
    ) -> Result<Vec<i32>, PJRTError<'rt>> {
        if num_replicas < 0 || num_partitions < 0 {
            return Err(self.error("num_replicas and num_partitions must be >= 0"));
        }

        let client = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_Client_DefaultDeviceAssignment
            .ok_or(self.error("PJRT_Client_DefaultDeviceAssignment symbol not found"))?;

        let mut probe = PJRT_Client_DefaultDeviceAssignment_Args {
            struct_size: PJRT_Client_DefaultDeviceAssignment_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            num_replicas,
            num_partitions,
            default_assignment_size: 0,
            default_assignment: null_mut(),
        };

        let probe_err = unsafe { f(&mut probe) };
        let expected_size = (num_replicas as usize).saturating_mul(num_partitions as usize);

        if !probe_err.is_null() && expected_size == 0 {
            return Err(PJRTError::new(self.rt, probe_err));
        }

        let mut out = vec![0i32; probe.default_assignment_size.max(expected_size)];
        if out.is_empty() {
            return Ok(out);
        }

        let mut args = PJRT_Client_DefaultDeviceAssignment_Args {
            struct_size: PJRT_Client_DefaultDeviceAssignment_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            num_replicas,
            num_partitions,
            default_assignment_size: out.len(),
            default_assignment: out.as_mut_ptr(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        out.truncate(args.default_assignment_size.min(out.len()));
        Ok(out)
    }

    pub fn buffer_from_host_slice_copy<T: Copy>(
        &self,
        data: &[T],
        element_type: PJRT_Buffer_Type,
        dims: &[i64],
        device: Option<PJRTDevice<'rt, '_>>,
    ) -> Result<PJRTBuffer<'rt, '_>, PJRTError<'rt>> {
        let (buf, done) = self.buffer_from_host_buffer(
            data.as_ptr().cast::<c_void>(),
            element_type,
            dims,
            None,
            PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall,
            device,
            None,
            None,
        )?;

        if let Some(ev) = done {
            // In this mode, it should be safe to drop the host memory after the call returns,
            // but we still await to avoid plugins that implement the transfer asynchronously.
            ev.await_ready()?;
        }

        Ok(buf)
    }

    // destroy errors
    pub fn close(self) -> Result<(), PJRTError<'rt>> {
        let raw = self.raw;
        let rt = self.rt;
        std::mem::forget(self);
        rt.destroy_client(raw)
    }

    pub fn platform_name(&self) -> Result<String, PJRTError<'rt>> {
        let client = self.raw_checked()?;

        let platform = self
            .rt
            .api()
            .PJRT_Client_PlatformName
            .ok_or(self.error("PJRT_Client_PlatformName symbol not found"))?;

        let mut args = PJRT_Client_PlatformName_Args {
            struct_size: PJRT_Client_PlatformName_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            client,
            platform_name: ptr::null(),
            platform_name_size: 0,
        };

        let err = unsafe { platform(&mut args) };

        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.platform_name.is_null() {
            if args.platform_name_size == 0 {
                return Ok(String::new());
            }
            return Err(self
                .error("PJRT_Client_PlatformName returned null platform_name with nonzero size"));
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

        let _ = self.rt.destroy_client(self.raw);
    }
}
