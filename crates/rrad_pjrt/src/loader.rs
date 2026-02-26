use libloading::{Library, Symbol};
use std::path::Path;
use std::ptr;
use std::slice::from_raw_parts;
use std::vec::Vec;

use crate::ffi::pjrt_sys::*;
use crate::client::PJRTClient;
use crate::device::PJRTDevice;
use crate::error::PJRTError;
use crate::ffi::error::{PjrtBindingError, PjrtFfiError};
use crate::topology_desc::{PJRTNamedAttribute, PJRTNamedValue};

type GetPjrtApiFn = unsafe extern "C" fn() -> *const PJRT_Api;



pub struct PjrtRuntime {
    _lib: Library,
    api: *const PJRT_Api,
}

impl PjrtRuntime {
    pub fn load(plugin_path: &Path) -> Result<Self, PjrtBindingError> {
        let lib = unsafe { Library::new(plugin_path) }
            .map_err(|e| PjrtBindingError::lib_load_failed(e.to_string()))?;

        let get_api: Symbol<GetPjrtApiFn> = unsafe { lib.get(b"GetPjrtApi\0") }
            .map_err(|e| PjrtBindingError::failed_to_get_pjrt_api(e.to_string()))?;

        let api = unsafe { get_api() };
        unsafe { validate_api_table(api)? };


        let ver = unsafe { (*api).pjrt_api_version };

        if ver.major_version != PJRT_API_MAJOR as i32 {
            return Err(PjrtBindingError::api_version_mismatch(ver.major_version, ver.minor_version));
        }


        if (ver.major_version == PJRT_API_MAJOR as i32) && (ver.minor_version < PJRT_API_MINOR as i32) {
            return Err(PjrtBindingError::api_version_mismatch(
                ver.major_version,
                ver.minor_version
            ));
        }


        Ok(Self { _lib: lib, api })
    }

    pub fn api(&self) -> &PJRT_Api {
        unsafe { &*self.api }
    }

    pub fn initialize_plugin(&self) -> Result<(), PJRTError<'_>> {
        let init = self
            .api()
            .PJRT_Plugin_Initialize
            .ok_or_else(|| PJRTError::invalid_arg(self, "PJRT_Plugin_Initialize symbol not found"))?;

        let mut args = PJRT_Plugin_Initialize_Args {
            struct_size: PJRT_Plugin_Initialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
        };

        let err = unsafe { init(&mut args) };

        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self, err))
        }
    }

    pub fn plugin_attributes(&self) -> Result<Vec<PJRTNamedAttribute>, PJRTError<'_>> {
        let f = self
            .api()
            .PJRT_Plugin_Attributes
            .ok_or_else(|| PJRTError::invalid_arg(self, "PJRT_Plugin_Attributes symbol not found"))?;

        let mut args = PJRT_Plugin_Attributes_Args {
            struct_size: PJRT_Plugin_Attributes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            attributes: ptr::null(),
            num_attributes: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(PJRTError::new(self, err));
        }

        decode_named_values(self, args.attributes, args.num_attributes)
    }

    pub fn create_client<'rt>(&'rt self) -> Result<PJRTClient<'rt>, PJRTError<'rt>> {
        let f = self
            .api()
            .PJRT_Client_Create
            .ok_or_else(|| PJRTError::invalid_arg(self, "PJRT_Client_Create symbol not found"))?;

        let mut args = PJRT_Client_Create_Args {
            struct_size: PJRT_Client_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            create_options: ptr::null(),
            num_options: 0,
            kv_get_callback: None,
            kv_get_user_arg: ptr::null_mut(),
            kv_put_callback: None,
            kv_put_user_arg: ptr::null_mut(),
            client: ptr::null_mut(),
            kv_try_get_callback: None,
            kv_try_get_user_arg: ptr::null_mut(),
        };

        let err = unsafe { f(&mut args) };

        if err.is_null() {
            if args.client.is_null() {
                return Err(PJRTError::invalid_arg(
                    self,
                    "PJRT_Client_Create succeeded but returned null client",
                ));
            }
            let client = PJRTClient {
                rt: self,
                raw: args.client,
            };
            Ok(client)
        } else {
            Err(PJRTError::new(self, err))
        }
    }

    pub fn destroy_client(&self, client: *mut PJRT_Client) -> Result<(), PJRTError<'_>> {
        if client.is_null() {
            return Err(PJRTError::invalid_arg(self, "PJRT_Client is null"));
        }

        let f = self
            .api()
            .PJRT_Client_Destroy
            .ok_or_else(|| PJRTError::invalid_arg(self, "PJRT_Client_Destroy symbol not found"))?;
        let mut args = PJRT_Client_Destroy_Args {
            struct_size: PJRT_Client_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
        };

        let err = unsafe { f(&mut args) };

        if err.is_null() {
            Ok(())
        } else {
            Err(PJRTError::new(self, err))
        }
    }

    #[allow(dead_code)]
    pub fn create_device(&self) -> Result<*mut PJRT_Device, PJRTError<'_>> {
        Err(PJRTError::invalid_arg(
            self,
            "PjrtDevice objects are obtained from PJRT_Client_Devices; there is no PJRT_Device_Create in the C API",
        ))
    }

    pub fn client_devices<'rt, 'client>(
        &'rt self,
        raw_client: *mut PJRT_Client,
        ) -> Result<Vec<PJRTDevice<'rt, 'client>>, PJRTError<'rt>> {
        if raw_client.is_null() {
            return Err(PJRTError::invalid_arg(self, "PJRT_Client is null"));
        }

        let f = self
            .api()
            .PJRT_Client_Devices
            .ok_or_else(|| PJRTError::invalid_arg(self, "PJRT_Client_Devices symbol not found"))?;
        let mut args = PJRT_Client_Devices_Args {
            struct_size: PJRT_Client_Devices_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client: raw_client,
            devices: ptr::null(),
            num_devices: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(PJRTError::new(self, err));
        }

        if args.num_devices == 0 {
            return Ok(Vec::new());
        }

        if args.devices.is_null() {
            return Err(PJRTError::invalid_arg(
                self,
                "PJRT_Client_Devices returned null devices with nonzero count",
            ));
        }

        let raw_devices = unsafe { from_raw_parts(args.devices, args.num_devices) };
        Ok(raw_devices
            .iter()
            .copied()
            .map(|raw_device| PJRTDevice::new(self, raw_device))
            .collect())
    }
}

fn decode_named_values<'a>(
    rt: &'a PjrtRuntime,
    attrs: *const PJRT_NamedValue,
    num_attrs: usize,
) -> Result<Vec<PJRTNamedAttribute>, PJRTError<'a>> {
    const NV_STRING: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kString;
    const NV_INT64: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64;
    const NV_INT64_LIST: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List;
    const NV_FLOAT: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kFloat;
    const NV_BOOL: PJRT_NamedValue_Type = PJRT_NamedValue_Type_PJRT_NamedValue_kBool;

    if num_attrs == 0 {
        return Ok(Vec::new());
    }
    if attrs.is_null() {
        return Err(PJRTError::invalid_arg(
            rt,
            "NamedValue pointer is null with nonzero count",
        ));
    }

    let values = unsafe { from_raw_parts(attrs, num_attrs) };
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        if value.name.is_null() && value.name_size != 0 {
            return Err(PJRTError::invalid_arg(rt, "NamedValue name pointer is null"));
        }

        let name_bytes = if value.name_size == 0 {
            &[][..]
        } else {
            unsafe { from_raw_parts(value.name as *const u8, value.name_size) }
        };
        let name = String::from_utf8_lossy(name_bytes).into_owned();

        let parsed = match value.type_ {
            NV_STRING => {
                let ptr = unsafe { value.__bindgen_anon_1.string_value };
                if ptr.is_null() && value.value_size != 0 {
                    return Err(PJRTError::invalid_arg(
                        rt,
                        format!("NamedValue '{name}' has null string pointer"),
                    ));
                }
                let bytes = if value.value_size == 0 {
                    &[][..]
                } else {
                    unsafe { from_raw_parts(ptr as *const u8, value.value_size) }
                };
                PJRTNamedValue::String(String::from_utf8_lossy(bytes).into_owned())
            }
            NV_INT64 => PJRTNamedValue::Int64(unsafe { value.__bindgen_anon_1.int64_value }),
            NV_INT64_LIST => {
                let ptr = unsafe { value.__bindgen_anon_1.int64_array_value };
                if ptr.is_null() && value.value_size != 0 {
                    return Err(PJRTError::invalid_arg(
                        rt,
                        format!("NamedValue '{name}' has null int64 list pointer"),
                    ));
                }
                let ints = if value.value_size == 0 {
                    Vec::new()
                } else {
                    unsafe { from_raw_parts(ptr, value.value_size).to_vec() }
                };
                PJRTNamedValue::Int64List(ints)
            }
            NV_FLOAT => PJRTNamedValue::Float(unsafe { value.__bindgen_anon_1.float_value }),
            NV_BOOL => PJRTNamedValue::Bool(unsafe { value.__bindgen_anon_1.bool_value }),
            other => {
                return Err(PJRTError::invalid_arg(
                    rt,
                    format!("NamedValue '{name}' has unknown type tag {other}"),
                ))
            }
        };

        out.push(PJRTNamedAttribute {
            name,
            value: parsed,
        });
    }
    Ok(out)
}

fn require_fn<T>(name: &'static str, f: Option<T>) -> Result<(), PjrtBindingError> {
    if f.is_none() {
        return Err(PjrtBindingError::new(
            PjrtFfiError::NullFunctionPointer { name },
        ));
    }
    Ok(())
}

unsafe fn validate_api_table(api: *const PJRT_Api) -> Result<(), PjrtBindingError> {
    if api.is_null() {
        return Err(PjrtBindingError::null_value_returned("GetPjrtApi returned null"));
    }
    let a = &*api;

    require_fn("PJRT_Client_Create", a.PJRT_Client_Create)?;
    require_fn("PJRT_Client_Destroy", a.PJRT_Client_Destroy)?;
    require_fn("PJRT_Client_Compile", a.PJRT_Client_Compile)?;
    require_fn("PJRT_LoadedExecutable_Execute", a.PJRT_LoadedExecutable_Execute)?;
    require_fn("PJRT_Buffer_Destroy", a.PJRT_Buffer_Destroy)?;
    require_fn("PJRT_Event_Destroy", a.PJRT_Event_Destroy)?;
    Ok(())
}