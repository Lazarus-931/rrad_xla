use crate::pjrt_sys::*;
use crate::rrad_pjrt::buffer::PJRTBuffer;
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::event::PJRTEvent;
use crate::rrad_pjrt::execute_context::PJRTExecuteContext;
use crate::rrad_pjrt::loader::PjrtRuntime;
use std::ptr;
use std::ptr::{null, null_mut};
use std::slice::from_raw_parts;

pub struct PJRTLoadedExecutable<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_LoadedExecutable,
}

// Back-compat with the original name in this crate.
pub type PJRTExecutable<'a> = PJRTLoadedExecutable<'a>;

#[derive(Clone, Copy, Default)]
pub struct PJRTExecuteRunOptions<'a> {
    pub execute_context: Option<&'a PJRTExecuteContext<'a>>,
    pub launch_id: i32,
    pub non_donatable_input_indices: &'a [i64],
    pub execute_device: Option<*mut PJRT_Device>,
    pub num_send_ops: usize,
    pub num_recv_ops: usize,
}

impl<'a> PJRTLoadedExecutable<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_LoadedExecutable) -> Self {
        Self { rt, raw }
    }

    fn raw_checked(&self) -> Result<*mut PJRT_LoadedExecutable, PJRTError<'a>> {
        if self.raw.is_null() {
            Err(self.error("PJRT_LoadedExecutable is null"))
        } else {
            Ok(self.raw)
        }
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn executable(&self) -> Result<*mut PJRT_Executable, PJRTError<'a>> {
        let raw = self.raw_checked()?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_GetExecutable
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_GetExecutable symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_GetExecutable_Args {
            struct_size: PJRT_LoadedExecutable_GetExecutable_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            loaded_executable: raw,
            executable: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.executable.is_null() {
            return Err(self.error("PJRT_LoadedExecutable_GetExecutable returned null executable"));
        }
        Ok(args.executable)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, PJRTError<'a>> {
        let executable = self.executable()?;
        let func = self
            .rt
            .api()
            .PJRT_Executable_Serialize
            .ok_or_else(|| self.error("PJRT_Executable_Serialize symbol not found"))?;

        let mut args = PJRT_Executable_Serialize_Args {
            struct_size: PJRT_Executable_Serialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: executable as *const PJRT_Executable,
            serialized_bytes: ptr::null(),
            serialized_bytes_size: 0,
            serialized_executable: ptr::null_mut(),
            serialized_executable_deleter: None,
        };

        let err = unsafe { func(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }

        if !args.serialized_executable.is_null() && args.serialized_executable_deleter.is_none() {
            return Err(self.error(
                "PJRT_Executable_Serialize returned serialized_executable without deleter",
            ));
        }

        let result = if args.serialized_bytes_size == 0 {
            Ok(Vec::new())
        } else if args.serialized_bytes.is_null() {
            Err(self.error(
                "PJRT_Executable_Serialize returned null serialized_bytes with nonzero size",
            ))
        } else {
            let bytes = unsafe {
                from_raw_parts(
                    args.serialized_bytes as *const u8,
                    args.serialized_bytes_size,
                )
            };
            Ok(bytes.to_vec())
        };

        if !args.serialized_executable.is_null() {
            if let Some(deleter) = args.serialized_executable_deleter {
                unsafe { deleter(args.serialized_executable) };
            }
        }

        result
    }

    pub fn deserialize_and_load(
        &self,
        client: *mut PJRT_Client,
        serialized_executable: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'a>, PJRTError<'a>> {
        if client.is_null() {
            return Err(self.error("client must not be null"));
        }
        if serialized_executable.is_empty() {
            return Err(self.error("serialized_executable must not be empty"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Executable_DeserializeAndLoad
            .ok_or_else(|| self.error("PJRT_Executable_DeserializeAndLoad symbol not found"))?;

        let serialized_ptr = serialized_executable.as_ptr() as *const libc::c_char;
        let serialized_size = serialized_executable.len();

        let (override_ptr, override_size) = match overridden_compile_options {
            None => (ptr::null(), 0usize),
            Some(opts) if opts.is_empty() => (ptr::null(), 0usize),
            Some(opts) => (opts.as_ptr() as *const libc::c_char, opts.len()),
        };

        let mut args = PJRT_Executable_DeserializeAndLoad_Args {
            struct_size: PJRT_Executable_DeserializeAndLoad_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            serialized_executable: serialized_ptr,
            serialized_executable_size: serialized_size,
            loaded_executable: ptr::null_mut(),
            overridden_serialized_compile_options: override_ptr,
            overridden_serialized_compile_options_size: override_size,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.loaded_executable.is_null() {
            return Err(self.error(
                "PJRT_Executable_DeserializeAndLoad succeeded but returned null loaded_executable",
            ));
        }

        Ok(PJRTLoadedExecutable::new(self.rt, args.loaded_executable))
    }

    pub fn get_compile_options(&self) -> Result<Vec<u8>, PJRTError<'a>> {
        let exec = self.executable()?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_GetCompileOptions
            .ok_or_else(|| self.error("PJRT_Executable_GetCompileOptions symbol not found"))?;

        let mut args = PJRT_Executable_GetCompileOptions_Args {
            struct_size: PJRT_Executable_GetCompileOptions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            serialized_bytes: ptr::null(),
            serialized_bytes_size: 0,
            serialized_compile_options: ptr::null_mut(),
            serialized_compile_options_deleter: None,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }

        if args.serialized_bytes_size > 0 && args.serialized_bytes.is_null() {
            return Err(
                self.error("PJRT_Executable_GetCompileOptions returned null serialized_bytes with nonzero size"));
        }

        if !args.serialized_compile_options.is_null()
            && args.serialized_compile_options_deleter.is_none()
        {
            return Err(self.error("PJRT_Executable_GetCompileOptions returned serialized_compile_options without deleter"));
        }

        let result = if args.serialized_bytes_size == 0 {
            Ok(Vec::new())
        } else {
            let bytes = unsafe {
                from_raw_parts(
                    args.serialized_bytes as *const u8,
                    args.serialized_bytes_size,
                )
            };
            Ok(bytes.to_vec())
        };

        if !args.serialized_compile_options.is_null() {
            if let Some(deleter) = args.serialized_compile_options_deleter {
                unsafe { deleter(args.serialized_compile_options) };
            }
        }

        result
    }

    fn num_outputs(&self) -> Result<usize, PJRTError<'a>> {
        let exec = self.executable()?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_NumOutputs
            .ok_or_else(|| self.error("PJRT_Executable_NumOutputs symbol not found"))?;

        let mut args = PJRT_Executable_NumOutputs_Args {
            struct_size: PJRT_Executable_NumOutputs_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_outputs: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.num_outputs)
        } else {
            Err(PJRTError::new(self.rt, err))
        }
    }

    pub fn execute(
        &self,
        arguments: &[&PJRTBuffer<'a>],
    ) -> Result<(Vec<PJRTBuffer<'a>>, PJRTEvent<'a>), PJRTError<'a>> {
        self.execute_with_execute_options(arguments, PJRTExecuteRunOptions::default())
            .map_err(|e| e)
    }

    pub fn execute_with_execute_options(
        &self,
        arguments: &[&PJRTBuffer<'a>],
        options: PJRTExecuteRunOptions<'a>,
    ) -> Result<(Vec<PJRTBuffer<'a>>, PJRTEvent<'a>), PJRTError<'a>> {
        let raw_executable = self.raw_checked()?;
        let num_outputs = self.num_outputs()?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_Execute
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_Execute symbol not found"))?;

        let argument_ptrs: Vec<*mut PJRT_Buffer> = arguments.iter().map(|b| b.raw()).collect();
        if argument_ptrs.iter().any(|p| p.is_null()) {
            return Err(self.error("execute arguments contain null PJRT_Buffer"));
        }

        if options.num_send_ops > 0 || options.num_recv_ops > 0 {
            return Err(
                self.error("send/recv callback counts are not supported yet (callbacks are null)")
            );
        }

        if options
            .non_donatable_input_indices
            .iter()
            .any(|index| *index < 0)
        {
            return Err(self.error("non_donatable_input_indices must be non-negative"));
        }

        let per_device_argument_lists: Vec<*const *mut PJRT_Buffer> =
            vec![if arguments.is_empty() {
                ptr::null()
            } else {
                argument_ptrs.as_ptr()
            }];

        let mut output_ptrs: Vec<*mut PJRT_Buffer> = vec![ptr::null_mut(); num_outputs];
        let per_device_output_lists: Vec<*mut *mut PJRT_Buffer> = vec![if num_outputs == 0 {
            ptr::null_mut()
        } else {
            output_ptrs.as_mut_ptr()
        }];

        let context_ptr = options
            .execute_context
            .map_or(ptr::null_mut(), |ctx| ctx.raw());
        if options.execute_context.is_some() && context_ptr.is_null() {
            return Err(self.error("execute_context is null"));
        }

        let non_donatable_ptr = if options.non_donatable_input_indices.is_empty() {
            ptr::null()
        } else {
            options.non_donatable_input_indices.as_ptr()
        };

        let execute_device = options.execute_device.unwrap_or(ptr::null_mut());
        if options.execute_device.is_some() && execute_device.is_null() {
            return Err(self.error("execute_device is null"));
        }

        let mut pjrt_options = PJRT_ExecuteOptions {
            struct_size: PJRT_ExecuteOptions_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            send_callbacks: ptr::null_mut(),
            recv_callbacks: ptr::null_mut(),
            num_send_ops: options.num_send_ops,
            num_recv_ops: options.num_recv_ops,
            launch_id: options.launch_id,
            non_donatable_input_indices: non_donatable_ptr,
            num_non_donatable_input_indices: options.non_donatable_input_indices.len(),
            context: context_ptr,
            call_location: ptr::null(),
            num_tasks: 0,
            task_ids: ptr::null_mut(),
            incarnation_ids: ptr::null_mut(),
        };

        let mut device_complete_event: *mut PJRT_Event = ptr::null_mut();

        let mut args = PJRT_LoadedExecutable_Execute_Args {
            struct_size: PJRT_LoadedExecutable_Execute_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw_executable,
            options: &mut pjrt_options,
            argument_lists: per_device_argument_lists.as_ptr(),
            num_devices: 1,
            num_args: argument_ptrs.len(),
            output_lists: per_device_output_lists.as_ptr(),
            device_complete_events: &mut device_complete_event,
            execute_device,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }

        if args.num_args != argument_ptrs.len() {
            return Err(self.error(format!(
                "execute argument count mismatch: requested {} but runtime used {}",
                argument_ptrs.len(),
                args.num_args
            )));
        }

        let output_list_ptr = per_device_output_lists[0];
        if num_outputs > 0 && output_list_ptr.is_null() {
            return Err(self.error(
                "PJRT_LoadedExecutable_Execute returned null output list with nonzero num_outputs",
            ));
        }

        let output_raws: Vec<*mut PJRT_Buffer> = if num_outputs == 0 {
            Vec::new()
        } else {
            unsafe { from_raw_parts(output_list_ptr, num_outputs).to_vec() }
        };
        if output_raws.iter().any(|p| p.is_null()) {
            return Err(self.error("PJRT_LoadedExecutable_Execute produced null output buffer"));
        }

        if device_complete_event.is_null() {
            return Err(self.error("PJRT_LoadedExecutable_Execute returned null completion event"));
        }

        let output_buffers = output_raws
            .into_iter()
            .map(|raw| PJRTBuffer::new(self.rt, raw))
            .collect();
        let event = PJRTEvent::new(self.rt, device_complete_event);
        Ok((output_buffers, event))
    }

    pub fn execute_with_options(
        &self,
        arguments: &[&PJRTBuffer<'a>],
        execute_context: Option<&'a PJRTExecuteContext<'a>>,
        num_send_ops: usize,
        num_recv_ops: usize,
        launch_id: i32,
        non_donatable_input_indices: &'a [i64],
        execute_device: *mut PJRT_Device,
    ) -> Result<(Vec<PJRTBuffer<'a>>, PJRTEvent<'a>), PJRTError<'a>> {
        let options = PJRTExecuteRunOptions {
            execute_context,
            launch_id,
            non_donatable_input_indices,
            execute_device: if execute_device.is_null() {
                None
            } else {
                Some(execute_device)
            },
            num_send_ops,
            num_recv_ops,
        };
        self.execute_with_execute_options(arguments, options)
    }

    pub fn execute_with_context(
        &self,
        arguments: &[&PJRTBuffer<'a>],
        execute_context: Option<&'a PJRTExecuteContext<'a>>,
    ) -> Result<(Vec<PJRTBuffer<'a>>, PJRTEvent<'a>), PJRTError<'a>> {
        let options = PJRTExecuteRunOptions {
            execute_context,
            ..Default::default()
        };
        self.execute_with_execute_options(arguments, options)
    }

    pub fn num_replicas(&self) -> Result<usize, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_NumReplicas
            .ok_or_else(|| self.error("PJRT_Executable_NumReplicas symbol not found"))?;

        let mut args = PJRT_Executable_NumReplicas_Args {
            struct_size: PJRT_Executable_NumReplicas_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_replicas: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.num_replicas)
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn num_partitions(&self) -> Result<usize, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_NumPartitions
            .ok_or_else(|| self.error("PJRT_Executable_NumPartitions symbol not found"))?;

        let mut args = PJRT_Executable_NumPartitions_Args {
            struct_size: PJRT_Executable_NumPartitions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_partitions: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.num_partitions)
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn destroy_executable_handle(&self) -> Result<(), PJRTError<'a>> {
        let executable = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_Destroy
            .ok_or_else(|| self.error("PJRT_Executable_Destroy symbol not found"))?;

        let mut args = PJRT_Executable_Destroy_Args {
            struct_size: PJRT_Executable_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn delete(&self) -> Result<(), PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_Delete
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_Delete symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_Delete_Args {
            struct_size: PJRT_LoadedExecutable_Delete_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn is_deleted(&self) -> Result<bool, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_IsDeleted
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_IsDeleted symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_IsDeleted_Args {
            struct_size: PJRT_LoadedExecutable_IsDeleted_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            is_deleted: false,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.is_deleted)
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn output_element_types(&self) -> Result<Vec<PJRT_Buffer_Type>, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_OutputElementTypes
            .ok_or_else(|| self.error("PJRT_Executable_OutputElementTypes symbol not found"))?;

        let mut args = PJRT_Executable_OutputElementTypes_Args {
            struct_size: PJRT_Executable_OutputElementTypes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            output_types: ptr::null_mut(),
            num_output_types: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if args.num_output_types == 0 {
            return Ok(Vec::new());
        }
        if args.output_types.is_null() {
            return Err(
                self.error("PJRT_Executable_OutputElementTypes returned null output_types with nonzero count")
            );
        }

        let output_types =
            unsafe { from_raw_parts(args.output_types, args.num_output_types).to_vec() };
        Ok(output_types)
    }

    pub fn addressable_devices(&self) -> Result<Vec<*mut PJRT_Device>, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_AddressableDevices
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_AddressableDevices symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_AddressableDevices_Args {
            struct_size: PJRT_LoadedExecutable_AddressableDevices_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            addressable_devices: ptr::null(),
            num_addressable_devices: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if args.num_addressable_devices == 0 {
            return Ok(Vec::new());
        }
        if args.addressable_devices.is_null() {
            return Err(
                self.error("PJRT_LoadedExecutable_AddressableDevices returned null list with nonzero count")
            );
        }

        let devices =
            unsafe { from_raw_parts(args.addressable_devices, args.num_addressable_devices) };
        Ok(devices.to_vec())
    }

    pub fn addressable_device_refs(&self) -> Result<Vec<PJRTDevice<'a>>, PJRTError<'a>> {
        Ok(self
            .addressable_devices()?
            .into_iter()
            .map(|raw| PJRTDevice::new(self.rt, raw))
            .collect())
    }

    pub fn addressable_device_ids(&self) -> Result<Vec<i32>, PJRTError<'a>> {
        self.addressable_device_refs()?
            .iter()
            .map(PJRTDevice::id)
            .collect()
    }

    pub fn addressable_device_kinds(&self) -> Result<Vec<String>, PJRTError<'a>> {
        self.addressable_device_refs()?
            .iter()
            .map(PJRTDevice::kind)
            .collect()
    }

    pub fn addressable_device_process_indices(&self) -> Result<Vec<i32>, PJRTError<'a>> {
        self.addressable_device_refs()?
            .iter()
            .map(PJRTDevice::process_index)
            .collect()
    }

    pub fn addressable_device_debug_strings(&self) -> Result<Vec<String>, PJRTError<'a>> {
        self.addressable_device_refs()?
            .iter()
            .map(PJRTDevice::debug_string)
            .collect()
    }

    pub fn fingerprint(&self) -> Result<String, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_Fingerprint
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_Fingerprint symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_Fingerprint_Args {
            struct_size: PJRT_LoadedExecutable_Fingerprint_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            executable_fingerprint: ptr::null(),
            executable_fingerprint_size: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }

        if args.executable_fingerprint.is_null() {
            return Err(self.error("PJRT_LoadedExecutable_Fingerprint returned null fingerprint"));
        }

        let bytes = unsafe {
            from_raw_parts(
                args.executable_fingerprint as *const u8,
                args.executable_fingerprint_size,
            )
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn executable_fingerprint(&self) -> Result<String, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_Fingerprint
            .ok_or_else(|| self.error("PJRT_Executable_Fingerprint symbol not found"))?;

        let mut args = PJRT_Executable_Fingerprint_Args {
            struct_size: PJRT_Executable_Fingerprint_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            executable_fingerprint: ptr::null(),
            executable_fingerprint_size: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if args.executable_fingerprint.is_null() {
            if args.executable_fingerprint_size == 0 {
                return Ok(String::new());
            }
            return Err(
                self.error("PJRT_Executable_Fingerprint returned null fingerprint with nonzero size")
            );
        }

        let bytes = unsafe {
            from_raw_parts(
                args.executable_fingerprint as *const u8,
                args.executable_fingerprint_size,
            )
        };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn size_of_generated_code_in_bytes(&self) -> Result<i64, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_SizeOfGeneratedCodeInBytes
            .ok_or_else(|| self.error("PJRT_Executable_SizeOfGeneratedCodeInBytes symbol not found"))?;

        let mut args = PJRT_Executable_SizeOfGeneratedCodeInBytes_Args {
            struct_size: PJRT_Executable_SizeOfGeneratedCodeInBytes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            size_in_bytes: 0,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.size_in_bytes)
        } else {
            Err(self.error("Error is non-null"))
        }
    }

    pub fn output_memory_kinds(&self) -> Result<Vec<String>, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_OutputMemoryKinds
            .ok_or_else(|| self.error("PJRT_Executable_OutputMemoryKinds symbol not found"))?;

        let mut args = PJRT_Executable_OutputMemoryKinds_Args {
            struct_size: PJRT_Executable_OutputMemoryKinds_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            num_outputs: 0,
            memory_kinds: ptr::null(),
            memory_kind_sizes: ptr::null(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if args.num_outputs == 0 {
            return Ok(Vec::new());
        }
        if args.memory_kinds.is_null() || args.memory_kind_sizes.is_null() {
            return Err(
                self.error("PJRT_Executable_OutputMemoryKinds returned null lists with nonzero num_outputs")
            );
        }

        let memory_kinds = unsafe { from_raw_parts(args.memory_kinds, args.num_outputs) };
        let memory_kind_sizes = unsafe { from_raw_parts(args.memory_kind_sizes, args.num_outputs) };

        let mut out = Vec::with_capacity(args.num_outputs);
        for i in 0..args.num_outputs {
            let kind_ptr = memory_kinds[i];
            let kind_size = memory_kind_sizes[i];
            if kind_ptr.is_null() {
                if kind_size == 0 {
                    out.push(String::new());
                    continue;
                }
                return Err(self.error(format!(
                    "PJRT_Executable_OutputMemoryKinds returned null memory kind at index {} with nonzero size",
                    i
                )));
            }
            let bytes = unsafe { from_raw_parts(kind_ptr as *const u8, kind_size) };
            out.push(String::from_utf8_lossy(bytes).into_owned());
        }

        Ok(out)
    }

    pub fn device_assignment_serialized(&self) -> Result<Vec<u8>, PJRTError<'a>> {
        let raw = self.raw_checked().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_LoadedExecutable_GetDeviceAssignment
            .ok_or_else(|| self.error("PJRT_LoadedExecutable_GetDeviceAssignment symbol not found"))?;

        let mut args = PJRT_LoadedExecutable_GetDeviceAssignment_Args {
            struct_size: PJRT_LoadedExecutable_GetDeviceAssignment_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: raw,
            serialized_bytes: ptr::null(),
            serialized_bytes_size: 0,
            serialized_device_assignment: ptr::null_mut(),
            serialized_device_assignment_deleter: None,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }
        if !args.serialized_device_assignment.is_null()
            && args.serialized_device_assignment_deleter.is_none()
        {
            return Err(
                self.error(
                    "PJRT_LoadedExecutable_GetDeviceAssignment returned serialized object without deleter")
            );
        }

        let result = if args.serialized_bytes_size == 0 {
            Ok(Vec::new())
        } else if args.serialized_bytes.is_null() {
            Err(self.error(
                "PJRT_LoadedExecutable_GetDeviceAssignment returned null bytes with nonzero size",
            ))
        } else {
            let bytes = unsafe {
                from_raw_parts(
                    args.serialized_bytes as *const u8,
                    args.serialized_bytes_size,
                )
            };
            Ok(bytes.to_vec())
        };

        if !args.serialized_device_assignment.is_null() {
            if let Some(deleter) = args.serialized_device_assignment_deleter {
                unsafe { deleter(args.serialized_device_assignment) };
            }
        }

        result
    }

    pub fn name(&self) -> Result<String, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let f = self
            .rt
            .api()
            .PJRT_Executable_Name
            .ok_or_else(|| self.error("PJRT_Executable_Name symbol not found"))?;

        let mut args = PJRT_Executable_Name_Args {
            struct_size: PJRT_Executable_Name_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: exec,
            executable_name: ptr::null(),
            executable_name_size: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(self.error("Error is non-null"));
        }

        if args.executable_name.is_null() {
            return Err(self.error("PJRT_Executable_Name returned null executable_name"));
        }

        let bytes =
            unsafe { from_raw_parts(args.executable_name as *const u8, args.executable_name_size) };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn get_compiled_memory_stats(&self) -> Result<Vec<i64>, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Executable_GetCompiledMemoryStats
            .ok_or_else(|| self.error("PJRT_Executable_GetCompiledMemoryStats symbol not found"))?;

        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args {
            struct_size: PJRT_Executable_GetCompiledMemoryStats_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            executable: exec,
            generated_code_size_in_bytes: 0,
            argument_size_in_bytes: 0,
            output_size_in_bytes: 0,
            alias_size_in_bytes: 0,
            temp_size_in_bytes: 0,
            host_generated_code_size_in_bytes: 0,
            host_argument_size_in_bytes: 0,
            host_output_size_in_bytes: 0,
            host_alias_size_in_bytes: 0,
            host_temp_size_in_bytes: 0,
            peak_memory_in_bytes: 0,
            total_size_in_bytes: 0,
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else {
            let stats = vec![
                args.generated_code_size_in_bytes,
                args.argument_size_in_bytes,
                args.output_size_in_bytes,
                args.alias_size_in_bytes,
                args.temp_size_in_bytes,
                args.host_generated_code_size_in_bytes,
                args.host_argument_size_in_bytes,
                args.host_output_size_in_bytes,
                args.host_alias_size_in_bytes,
                args.host_temp_size_in_bytes,
                args.peak_memory_in_bytes,
                args.total_size_in_bytes,
            ];

            Ok(stats)
        }
    }

    pub fn get_cost_analysis(&self) -> Result<String, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Executable_GetCostAnalysis
            .ok_or_else(|| self.error("PJRT_Executable_GetCostAnalysis symbol not found"))?;

        let mut args = PJRT_Executable_GetCostAnalysis_Args {
            struct_size: PJRT_Executable_GetCostAnalysis_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            executable: exec,
            num_properties: 0,
            properties: ptr::null(),
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else if args.num_properties == 0 {
            Ok(String::new())
        } else if args.properties.is_null() {
            Err(
                self.error(
                    "PJRT_Executable_GetCostAnalysis returned null properties with nonzero count"))
        } else {
            let properties = unsafe { from_raw_parts(args.properties, args.num_properties) };
            let names = properties
                .iter()
                .map(|property| {
                    if property.name.is_null() {
                        "<null>".to_owned()
                    } else {
                        let bytes = unsafe {
                            from_raw_parts(property.name as *const u8, property.name_size)
                        };
                        String::from_utf8_lossy(bytes).into_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            Ok(names)
        }
    }

    pub fn optimized_program(&self) -> Result<(), PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Executable_OptimizedProgram
            .ok_or_else(|| self.error("PJRT_Exectuable_Optimized not found."))?;

        let mut args = PJRT_Executable_OptimizedProgram_Args {
            struct_size: PJRT_Executable_OptimizedProgram_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            executable: exec,
            program: null_mut(),
        };

        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else {
            Ok(())
        }
    }

    pub fn output_dimension(&self) -> Result<i64, PJRTError<'a>> {
        let exec = self.executable().map_err(|e| e)?;

        let func = self
            .rt
            .api()
            .PJRT_Executable_OutputDimensions
            .ok_or_else(|| self.error("PJRT_Executable_OutputDimensions not found."))?;

        let mut args = PJRT_Executable_OutputDimensions_Args {
            struct_size: PJRT_Executable_OutputDimensions_Args_STRUCT_SIZE as usize,
            extension_start: null_mut(),
            executable: exec,
            num_outputs: 0,
            dims: ptr::null(),
            dim_sizes: null(),
        };
        let err = unsafe { func(&mut args) };

        if !err.is_null() {
            Err(self.error("Error is non-null"))
        } else if args.num_outputs == 0 {
            Err(self.error("PJRT_Executable_OutputDimensions returned no outputs"))
        } else if args.dims.is_null() {
            Err(self.error("PJRT_Executable_OutputDimensions returned null dims"))
        } else {
            let dims = unsafe { from_raw_parts(args.dims as *const i64, args.num_outputs) };
            Ok(dims[0])
        }
    }
}

impl Drop for PJRTLoadedExecutable<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_LoadedExecutable_Destroy else {
            return;
        };

        let mut args = PJRT_LoadedExecutable_Destroy_Args {
            struct_size: PJRT_LoadedExecutable_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: self.raw,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            // Drop must not panic; best-effort cleanup.
            let _ = self.error("Error is non-null");
        }
    }
}
