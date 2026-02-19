use std::ptr;
use std::slice::from_raw_parts;

use crate::pjrt_sys::*;
use crate::rrad_pjrt::error::PJRTError;
use crate::rrad_pjrt::executable::PJRTLoadedExecutable;
use crate::rrad_pjrt::loader::{error_to_string, PjrtRuntime};

#[derive(Debug, Clone)]
pub enum PJRTNamedValue {
    String(String),
    Int64(i64),
    Int64List(Vec<i64>),
    Float(f32),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct PJRTNamedAttribute {
    pub name: String,
    pub value: PJRTNamedValue,
}

pub struct PJRTDeviceDescriptionRef<'a> {
    pub rt: &'a PjrtRuntime,
    pub raw: *mut PJRT_DeviceDescription,
}

impl<'a> PJRTDeviceDescriptionRef<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_DeviceDescription) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_DeviceDescription {
        self.raw
    }

    fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    fn raw_checked(&self) -> Result<*mut PJRT_DeviceDescription, String> {
        if self.raw.is_null() {
            Err(self.error("PJRT_DeviceDescription is null").to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn id(&self) -> Result<i32, String> {
        let raw = self.raw_checked()?;
        let f = self.rt.api().PJRT_DeviceDescription_Id.ok_or_else(|| {
            self.error("PJRT_DeviceDescription_Id symbol not found")
                .to_string()
        })?;

        let mut args = PJRT_DeviceDescription_Id_Args {
            struct_size: PJRT_DeviceDescription_Id_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            id: 0,
        };
        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.id)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn process_index(&self) -> Result<i32, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_ProcessIndex
            .ok_or_else(|| {
                self.error("PJRT_DeviceDescription_ProcessIndex symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_DeviceDescription_ProcessIndex_Args {
            struct_size: PJRT_DeviceDescription_ProcessIndex_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            process_index: 0,
        };
        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(args.process_index)
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    pub fn kind(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;
        let f = self.rt.api().PJRT_DeviceDescription_Kind.ok_or_else(|| {
            self.error("PJRT_DeviceDescription_Kind symbol not found")
                .to_string()
        })?;

        let mut args = PJRT_DeviceDescription_Kind_Args {
            struct_size: PJRT_DeviceDescription_Kind_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            device_kind: ptr::null(),
            device_kind_size: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        bytes_to_string(args.device_kind, args.device_kind_size, "device_kind")
    }

    pub fn debug_string(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_DebugString
            .ok_or_else(|| {
                self.error("PJRT_DeviceDescription_DebugString symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_DeviceDescription_DebugString_Args {
            struct_size: PJRT_DeviceDescription_DebugString_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            debug_string: ptr::null(),
            debug_string_size: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        bytes_to_string(args.debug_string, args.debug_string_size, "debug_string")
    }

    pub fn to_string(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_ToString
            .ok_or_else(|| {
                self.error("PJRT_DeviceDescription_ToString symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_DeviceDescription_ToString_Args {
            struct_size: PJRT_DeviceDescription_ToString_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            to_string: ptr::null(),
            to_string_size: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        bytes_to_string(args.to_string, args.to_string_size, "to_string")
    }

    pub fn attributes(&self) -> Result<Vec<PJRTNamedAttribute>, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_Attributes
            .ok_or_else(|| {
                self.error("PJRT_DeviceDescription_Attributes symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_DeviceDescription_Attributes_Args {
            struct_size: PJRT_DeviceDescription_Attributes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            device_description: raw,
            num_attributes: 0,
            attributes: ptr::null(),
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        decode_named_values(args.attributes, args.num_attributes)
    }
}

pub struct PJRTTopologyDescription<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_TopologyDescription,
}

impl<'a> PJRTTopologyDescription<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_TopologyDescription) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_TopologyDescription {
        self.raw
    }

    pub fn error(&self, msg: impl Into<String>) -> PJRTError<'a> {
        PJRTError::invalid_arg(self.rt, msg)
    }

    pub fn create(
        rt: &'a PjrtRuntime,
        topology_name: Option<&str>,
        create_options: &[PJRT_NamedValue],
    ) -> Result<Self, String> {
        let function = rt.api().PJRT_TopologyDescription_Create.ok_or_else(|| {
            PJRTError::invalid_arg(rt, "PJRT_TopologyDescription_Create symbol not found")
                .to_string()
        })?;

        let (topology_name_ptr, topology_name_size) = match topology_name {
            None => (ptr::null(), 0usize),
            Some(name) if name.is_empty() => (ptr::null(), 0usize),
            Some(name) => (name.as_ptr() as *const libc::c_char, name.len()),
        };

        let mut args = PJRT_TopologyDescription_Create_Args {
            struct_size: PJRT_TopologyDescription_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology_name: topology_name_ptr,
            topology_name_size,
            create_options: if create_options.is_empty() {
                ptr::null()
            } else {
                create_options.as_ptr()
            },
            num_options: create_options.len(),
            topology: ptr::null_mut(),
        };

        let err = unsafe { function(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(rt.api(), err));
        }
        if args.topology.is_null() {
            return Err(PJRTError::invalid_arg(
                rt,
                "PJRT_TopologyDescription_Create returned null topology",
            )
            .to_string());
        }

        Ok(Self::new(rt, args.topology))
    }

    pub fn create_default(rt: &'a PjrtRuntime) -> Result<Self, String> {
        Self::create(rt, None, &[])
    }

    fn raw_checked(&self) -> Result<*mut PJRT_TopologyDescription, String> {
        if self.raw.is_null() {
            Err(self.error("PJRT_TopologyDescription is null").to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn platform_name(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_TopologyDescription_PlatformName
            .ok_or_else(|| {
                self.error("PJRT_TopologyDescription_PlatformName symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_PlatformName_Args {
            struct_size: PJRT_TopologyDescription_PlatformName_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: raw,
            platform_name: ptr::null(),
            platform_name_size: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        bytes_to_string(args.platform_name, args.platform_name_size, "platform_name")
    }

    pub fn platform_version(&self) -> Result<String, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_TopologyDescription_PlatformVersion
            .ok_or_else(|| {
                self.error("PJRT_TopologyDescription_PlatformVersion symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_PlatformVersion_Args {
            struct_size: PJRT_TopologyDescription_PlatformVersion_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: raw,
            platform_version: ptr::null(),
            platform_version_size: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        bytes_to_string(
            args.platform_version,
            args.platform_version_size,
            "platform_version",
        )
    }

    pub fn device_descriptions(&self) -> Result<Vec<PJRTDeviceDescriptionRef<'a>>, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_TopologyDescription_GetDeviceDescriptions
            .ok_or_else(|| {
                self.error("PJRT_TopologyDescription_GetDeviceDescriptions symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_GetDeviceDescriptions_Args {
            struct_size: PJRT_TopologyDescription_GetDeviceDescriptions_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: raw,
            descriptions: ptr::null(),
            num_descriptions: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.num_descriptions == 0 {
            return Ok(Vec::new());
        }
        if args.descriptions.is_null() {
            return Err(self
                .error("Topology returned null descriptions with nonzero count")
                .to_string());
        }

        let descriptions = unsafe { from_raw_parts(args.descriptions, args.num_descriptions) };
        let out = descriptions
            .iter()
            .copied()
            .map(|raw_desc| PJRTDeviceDescriptionRef::new(self.rt, raw_desc))
            .collect();
        Ok(out)
    }

    pub fn attributes(&self) -> Result<Vec<PJRTNamedAttribute>, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_TopologyDescription_Attributes
            .ok_or_else(|| {
                self.error("PJRT_TopologyDescription_Attributes symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_Attributes_Args {
            struct_size: PJRT_TopologyDescription_Attributes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: raw,
            attributes: ptr::null(),
            num_attributes: 0,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        decode_named_values(args.attributes, args.num_attributes)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_TopologyDescription_Serialize
            .ok_or_else(|| {
                self.error("PJRT_TopologyDescription_Serialize symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_Serialize_Args {
            struct_size: PJRT_TopologyDescription_Serialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: raw,
            serialized_bytes: ptr::null(),
            serialized_bytes_size: 0,
            serialized_topology: ptr::null_mut(),
            serialized_topology_deleter: None,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if !args.serialized_topology.is_null() && args.serialized_topology_deleter.is_none() {
            return Err(self
                .error("Serialize returned serialized_topology without a deleter")
                .to_string());
        }
        if args.serialized_bytes.is_null() && args.serialized_bytes_size != 0 {
            return Err(self
                .error("Serialize returned null bytes with nonzero size")
                .to_string());
        }

        let bytes = if args.serialized_bytes_size == 0 {
            Vec::new()
        } else {
            unsafe {
                from_raw_parts(
                    args.serialized_bytes as *const u8,
                    args.serialized_bytes_size,
                )
                .to_vec()
            }
        };

        if !args.serialized_topology.is_null() {
            if let Some(deleter) = args.serialized_topology_deleter {
                unsafe { deleter(args.serialized_topology) };
            }
        }

        Ok(bytes)
    }

    pub fn deserialize(rt: &'a PjrtRuntime, serialized_topology: &[u8]) -> Result<Self, String> {
        if serialized_topology.is_empty() {
            return Err(
                PJRTError::invalid_arg(rt, "serialized_topology must not be empty").to_string(),
            );
        }

        let f = rt
            .api()
            .PJRT_TopologyDescription_Deserialize
            .ok_or_else(|| {
                PJRTError::invalid_arg(rt, "PJRT_TopologyDescription_Deserialize symbol not found")
                    .to_string()
            })?;

        let mut args = PJRT_TopologyDescription_Deserialize_Args {
            struct_size: PJRT_TopologyDescription_Deserialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            serialized_topology: serialized_topology.as_ptr() as *const libc::c_char,
            serialized_topology_size: serialized_topology.len(),
            topology: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(rt.api(), err));
        }
        if args.topology.is_null() {
            return Err(PJRTError::invalid_arg(
                rt,
                "PJRT_TopologyDescription_Deserialize returned null topology",
            )
            .to_string());
        }

        Ok(Self::new(rt, args.topology))
    }

    pub fn compile(
        &self,
        client: *mut PJRT_Client,
        program: &PJRT_Program,
        compile_options: &[u8],
    ) -> Result<*mut PJRT_Executable, String> {
        let topology = self.raw_checked()?;
        if client.is_null() {
            return Err(self.error("PJRT_Client is null").to_string());
        }

        let mut program_local = *program;
        if program_local.struct_size == 0 {
            program_local.struct_size = std::mem::size_of::<PJRT_Program>();
        }
        if program_local.code_size > 0 && program_local.code.is_null() {
            return Err(self
                .error("PJRT_Program.code is null but code_size is nonzero")
                .to_string());
        }
        if program_local.format_size > 0 && program_local.format.is_null() {
            return Err(self
                .error("PJRT_Program.format is null but format_size is nonzero")
                .to_string());
        }

        let f = self
            .rt
            .api()
            .PJRT_Compile
            .ok_or_else(|| self.error("PJRT_Compile symbol not found").to_string())?;

        let (compile_options_ptr, compile_options_size) = if compile_options.is_empty() {
            (ptr::null(), 0usize)
        } else {
            (
                compile_options.as_ptr() as *const libc::c_char,
                compile_options.len(),
            )
        };

        let mut args = PJRT_Compile_Args {
            struct_size: PJRT_Compile_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology,
            program: &program_local,
            compile_options: compile_options_ptr,
            compile_options_size,
            client,
            executable: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }
        if args.executable.is_null() {
            return Err(self
                .error("PJRT_Compile returned null executable")
                .to_string());
        }
        Ok(args.executable)
    }

    fn destroy_executable(&self, executable: *mut PJRT_Executable) -> Result<(), String> {
        if executable.is_null() {
            return Ok(());
        }

        let f = self.rt.api().PJRT_Executable_Destroy.ok_or_else(|| {
            self.error("PJRT_Executable_Destroy symbol not found")
                .to_string()
        })?;

        let mut args = PJRT_Executable_Destroy_Args {
            struct_size: PJRT_Executable_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable,
        };

        let err = unsafe { f(&mut args) };
        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.rt.api(), err))
        }
    }

    fn serialize_executable(&self, executable: *mut PJRT_Executable) -> Result<Vec<u8>, String> {
        let f = self.rt.api().PJRT_Executable_Serialize.ok_or_else(|| {
            self.error("PJRT_Executable_Serialize symbol not found")
                .to_string()
        })?;

        let mut args = PJRT_Executable_Serialize_Args {
            struct_size: PJRT_Executable_Serialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            executable: executable as *const PJRT_Executable,
            serialized_bytes: ptr::null(),
            serialized_bytes_size: 0,
            serialized_executable: ptr::null_mut(),
            serialized_executable_deleter: None,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.rt.api(), err));
        }

        if !args.serialized_executable.is_null() && args.serialized_executable_deleter.is_none() {
            return Err(self
                .error("PJRT_Executable_Serialize returned serialized_executable without deleter")
                .to_string());
        }

        let result = if args.serialized_bytes_size == 0 {
            Ok(Vec::new())
        } else if args.serialized_bytes.is_null() {
            Err(self
                .error("PJRT_Executable_Serialize returned null serialized_bytes with nonzero size")
                .to_string())
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

    pub fn compile_and_load(
        &self,
        client: *mut PJRT_Client,
        program: &PJRT_Program,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'a>, PJRTError<'a>> {
        let executable = self
            .compile(client, program, compile_options)
            .map_err(|e| self.error(e))?;
        let serialized = self.serialize_executable(executable);
        let destroy_result = self.destroy_executable(executable);

        let serialized = match serialized {
            Ok(bytes) => bytes,
            Err(e) => {
                if let Err(cleanup_err) = destroy_result {
                    return Err(self.error(format!(
                        "{e}; additionally failed to destroy compiled executable: {cleanup_err}"
                    )));
                }
                return Err(self.error(format!("Failed to serialize compiled executable: {e}")));
            }
        };

        if let Err(e) = destroy_result {
            return Err(self.error(format!("Failed to destroy compiled executable: {e}")));
        }

        if serialized.is_empty() {
            return Err(self.error("Compiled executable serialized to empty bytes"));
        }

        let f = self
            .rt
            .api()
            .PJRT_Executable_DeserializeAndLoad
            .ok_or_else(|| self.error("PJRT_Executable_DeserializeAndLoad symbol not found"))?;

        let (override_ptr, override_size) = match overridden_compile_options {
            None => (ptr::null(), 0usize),
            Some(opts) if opts.is_empty() => (ptr::null(), 0usize),
            Some(opts) => (opts.as_ptr() as *const libc::c_char, opts.len()),
        };

        let mut args = PJRT_Executable_DeserializeAndLoad_Args {
            struct_size: PJRT_Executable_DeserializeAndLoad_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            serialized_executable: serialized.as_ptr() as *const libc::c_char,
            serialized_executable_size: serialized.len(),
            loaded_executable: ptr::null_mut(),
            overridden_serialized_compile_options: override_ptr,
            overridden_serialized_compile_options_size: override_size,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self.rt, err));
        }
        if args.loaded_executable.is_null() {
            return Err(
                self.error("PJRT_Executable_DeserializeAndLoad returned null loaded_executable")
            );
        }

        Ok(PJRTLoadedExecutable::new(self.rt, args.loaded_executable))
    }

    pub fn compile_and_load_code(
        &self,
        client: *mut PJRT_Client,
        program_code: &str,
        format: &str,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'a>, String> {
        if program_code.is_empty() {
            return Err(self.error("program_code must not be empty").to_string());
        }
        if format.is_empty() {
            return Err(self.error("format must not be empty").to_string());
        }

        let program = PJRT_Program {
            struct_size: std::mem::size_of::<PJRT_Program>(),
            extension_start: ptr::null_mut(),
            code: program_code.as_ptr() as *mut libc::c_char,
            code_size: program_code.len(),
            format: format.as_ptr() as *const libc::c_char,
            format_size: format.len(),
        };

        self.compile_and_load(
            client,
            &program,
            compile_options,
            overridden_compile_options,
        )
        .map_err(|e| e.to_string())
    }
}

impl Drop for PJRTTopologyDescription<'_> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        let Some(f) = self.rt.api().PJRT_TopologyDescription_Destroy else {
            return;
        };

        let mut args = PJRT_TopologyDescription_Destroy_Args {
            struct_size: PJRT_TopologyDescription_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology: self.raw,
        };
        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            // Drop must not panic; best-effort cleanup.
            let _ = error_to_string(self.rt.api(), err);
        }
    }
}

fn bytes_to_string(
    ptr: *const libc::c_char,
    size: usize,
    field_name: &str,
) -> Result<String, String> {
    if size == 0 {
        return Ok(String::new());
    }
    if ptr.is_null() {
        return Err(format!("{field_name} pointer is null for non-empty string"));
    }
    let bytes = unsafe { from_raw_parts(ptr as *const u8, size) };
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn decode_named_values(
    attrs: *const PJRT_NamedValue,
    num_attrs: usize,
) -> Result<Vec<PJRTNamedAttribute>, String> {
    const NV_STRING: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kString;
    const NV_INT64: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64;
    const NV_INT64_LIST: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List;
    const NV_FLOAT: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kFloat;
    const NV_BOOL: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kBool;

    if num_attrs == 0 {
        return Ok(Vec::new());
    }
    if attrs.is_null() {
        return Err("NamedValue pointer is null with nonzero count".to_string());
    }

    let values = unsafe { from_raw_parts(attrs, num_attrs) };
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        if value.name.is_null() && value.name_size != 0 {
            return Err("NamedValue name pointer is null".to_string());
        }
        let name = {
            let name_bytes = if value.name_size == 0 {
                &[][..]
            } else {
                unsafe { from_raw_parts(value.name as *const u8, value.name_size) }
            };
            String::from_utf8_lossy(name_bytes).into_owned()
        };

        let parsed = match value.type_ {
            NV_STRING => {
                let ptr = unsafe { value.__bindgen_anon_1.string_value };
                if ptr.is_null() && value.value_size != 0 {
                    return Err(format!("NamedValue '{name}' has null string pointer"));
                }
                let bytes = if value.value_size == 0 {
                    &[]
                } else {
                    unsafe { from_raw_parts(ptr as *const u8, value.value_size) }
                };
                PJRTNamedValue::String(String::from_utf8_lossy(bytes).into_owned())
            }
            NV_INT64 => {
                let v = unsafe { value.__bindgen_anon_1.int64_value };
                PJRTNamedValue::Int64(v)
            }
            NV_INT64_LIST => {
                let ptr = unsafe { value.__bindgen_anon_1.int64_array_value };
                if ptr.is_null() && value.value_size != 0 {
                    return Err(format!("NamedValue '{name}' has null int64 list pointer"));
                }
                let ints = if value.value_size == 0 {
                    Vec::new()
                } else {
                    unsafe { from_raw_parts(ptr, value.value_size).to_vec() }
                };
                PJRTNamedValue::Int64List(ints)
            }
            NV_FLOAT => {
                let v = unsafe { value.__bindgen_anon_1.float_value };
                PJRTNamedValue::Float(v)
            }
            NV_BOOL => {
                let v = unsafe { value.__bindgen_anon_1.bool_value };
                PJRTNamedValue::Bool(v)
            }
            other => return Err(format!("NamedValue '{name}' has unknown type tag {other}")),
        };

        out.push(PJRTNamedAttribute {
            name,
            value: parsed,
        });
    }
    Ok(out)
}
