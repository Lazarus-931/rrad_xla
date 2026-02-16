use std::ptr;
use std::slice::from_raw_parts;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;

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
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_DeviceDescription,
}

impl<'a> PJRTDeviceDescriptionRef<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_DeviceDescription) -> Self {
        Self { rt, raw }
    }

    pub fn raw(&self) -> *mut PJRT_DeviceDescription {
        self.raw
    }

    fn raw_checked(&self) -> Result<*mut PJRT_DeviceDescription, String> {
        if self.raw.is_null() {
            Err("PJRT_DeviceDescription is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn id(&self) -> Result<i32, String> {
        let raw = self.raw_checked()?;
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_Id
            .ok_or("PJRT_DeviceDescription_Id symbol not found")?;

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
            .ok_or("PJRT_DeviceDescription_ProcessIndex symbol not found")?;

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
        let f = self
            .rt
            .api()
            .PJRT_DeviceDescription_Kind
            .ok_or("PJRT_DeviceDescription_Kind symbol not found")?;

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
            .ok_or("PJRT_DeviceDescription_DebugString symbol not found")?;

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
            .ok_or("PJRT_DeviceDescription_ToString symbol not found")?;

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
            .ok_or("PJRT_DeviceDescription_Attributes symbol not found")?;

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

    fn raw_checked(&self) -> Result<*mut PJRT_TopologyDescription, String> {
        if self.raw.is_null() {
            Err("PJRT_TopologyDescription is null".to_string())
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
            .ok_or("PJRT_TopologyDescription_PlatformName symbol not found")?;

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
            .ok_or("PJRT_TopologyDescription_PlatformVersion symbol not found")?;

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
            .ok_or("PJRT_TopologyDescription_GetDeviceDescriptions symbol not found")?;

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
            return Err("Topology returned null descriptions with nonzero count".to_string());
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
            .ok_or("PJRT_TopologyDescription_Attributes symbol not found")?;

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
            .ok_or("PJRT_TopologyDescription_Serialize symbol not found")?;

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
        if args.serialized_bytes.is_null() && args.serialized_bytes_size != 0 {
            return Err("Serialize returned null bytes with nonzero size".to_string());
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

        if let Some(deleter) = args.serialized_topology_deleter {
            unsafe { deleter(args.serialized_topology) };
        }

        Ok(bytes)
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

fn bytes_to_string(ptr: *const i8, size: usize, field_name: &str) -> Result<String, String> {
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
        if value.name.is_null() {
            return Err("NamedValue name pointer is null".to_string());
        }
        let name = {
            let name_bytes = unsafe { from_raw_parts(value.name as *const u8, value.name_size) };
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
