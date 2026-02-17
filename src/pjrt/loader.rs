use libloading::{Library, Symbol};
use std::path::Path;
use std::ptr;
use std::slice::from_raw_parts;
use std::vec::Vec;

use crate::pjrt::client::PJRTClient;
use crate::pjrt::topology_desc::{PJRTNamedAttribute, PJRTNamedValue};
use crate::pjrt_sys::*;

type GetPjrtApiFn = unsafe extern "C" fn() -> *const PJRT_Api;

pub struct PjrtRuntime {
    _lib: Library,
    api: *const PJRT_Api,
}

impl PjrtRuntime {
    pub fn load(plugin_path: &Path) -> Result<Self, String> {
        let lib = unsafe { Library::new(plugin_path) }
            .map_err(|e| format!("Failed to load plugin: {e}"))?;

        let get_api: Symbol<GetPjrtApiFn> = unsafe { lib.get(b"GetPjrtApi\0") }
            .map_err(|e| format!("GetPjrtApi symbol not found: {e}"))?;

        let api = unsafe { get_api() };

        if api.is_null() {
            return Err("GetPjrtApi returned null".to_string());
        }

        let ver = unsafe { (*api).pjrt_api_version };

        if ver.major_version != PJRT_API_MAJOR as i32 {
            return Err(format!(
                "PJRT API major mismatch: host={} plugin={}",
                PJRT_API_MAJOR, ver.major_version
            ));
        }

        if ver.minor_version < PJRT_API_MINOR as i32 {
            eprintln!(
                "warning: plugin minor {} is older than header minor {}",
                ver.minor_version, PJRT_API_MINOR
            );
        }

        Ok(Self { _lib: lib, api })
    }

    pub fn api(&self) -> &PJRT_Api {
        unsafe { &*self.api }
    }

    pub fn initialize_plugin(&self) -> Result<(), String> {
        let init = self
            .api()
            .PJRT_Plugin_Initialize
            .ok_or("PJRT_Plugin_Initialize symbol not found")?;

        let mut args = PJRT_Plugin_Initialize_Args {
            struct_size: PJRT_Plugin_Initialize_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
        };

        let err = unsafe { init(&mut args) };

        if err.is_null() {
            return Ok(());
        }

        Err(error_to_string(self.api(), err))
    }

    pub fn plugin_attributes(&self) -> Result<Vec<PJRTNamedAttribute>, String> {
        let f = self
            .api()
            .PJRT_Plugin_Attributes
            .ok_or("PJRT_Plugin_Attributes symbol not found")?;

        let mut args = PJRT_Plugin_Attributes_Args {
            struct_size: PJRT_Plugin_Attributes_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            attributes: ptr::null(),
            num_attributes: 0,
        };

        let err = unsafe { f(&mut args) };
        if !err.is_null() {
            return Err(error_to_string(self.api(), err));
        }

        decode_named_values(args.attributes, args.num_attributes)
    }

    pub fn create_client(&self) -> Result<*mut PJRT_Client, String> {
        let f = self
            .api()
            .PJRT_Client_Create
            .ok_or("PJRT_Client_Create symbol not found")?;

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
                return Err("PJRT_Client_Create succeeded but returned null client".into());
            }
            Ok(args.client)
        } else {
            Err(error_to_string(self.api(), err))
        }
    }

    pub fn create_client_raii(&self) -> Result<PJRTClient<'_>, String> {
        let raw = self.create_client()?;
        Ok(PJRTClient::new(self, raw))
    }

    pub fn destroy_client(&self, client: *mut PJRT_Client) -> Result<(), String> {
        let f = self
            .api()
            .PJRT_Client_Destroy
            .ok_or("PJRT_Client_Destroy symbol not found")?;

        let mut args = PJRT_Client_Destroy_Args {
            struct_size: PJRT_Client_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
        };

        let err = unsafe { f(&mut args) };

        if err.is_null() {
            Ok(())
        } else {
            Err(error_to_string(self.api(), err))
        }
    }

    #[allow(dead_code)]
    pub fn create_device(&self) -> Result<*mut PJRT_Device, String> {
        Err("PJRT_Device objects are obtained from PJRT_Client_Devices; there is no PJRT_Device_Create in the C API".to_string())
    }

    pub fn client_devices(
        &self,
        client: *mut PJRT_Client,
    ) -> Result<Vec<*mut PJRT_Device>, String> {
        let f = self
            .api()
            .PJRT_Client_Devices
            .ok_or("PJRT_Client_Devices symbol not found")?;

        let mut args = PJRT_Client_Devices_Args {
            struct_size: PJRT_Client_Devices_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client,
            devices: ptr::null(),
            num_devices: 0,
        };

        let err = unsafe { f(&mut args) };

        if !err.is_null() {
            return Err(error_to_string(self.api(), err));
        }

        if args.num_devices == 0 {
            return Ok(Vec::new());
        }

        if args.devices.is_null() {
            return Err("PJRT_Client_Devices returned null devices with nonzero count".to_string());
        }

        let devices = unsafe { from_raw_parts(args.devices, args.num_devices) };
        Ok(devices.to_vec())
    }
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
                    return Err(format!("NamedValue '{name}' has null string pointer"));
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
                    return Err(format!("NamedValue '{name}' has null int64 list pointer"));
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
            other => return Err(format!("NamedValue '{name}' has unknown type tag {other}")),
        };

        out.push(PJRTNamedAttribute {
            name,
            value: parsed,
        });
    }
    Ok(out)
}

pub(crate) fn error_to_string(api: &PJRT_Api, error: *mut PJRT_Error) -> String {
    if error.is_null() {
        return "unknown PJRT error".to_string();
    }

    let mut msg = if let Some(msg_fn) = api.PJRT_Error_Message {
        let mut msg_args = PJRT_Error_Message_Args {
            struct_size: PJRT_Error_Message_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            error,
            message: ptr::null(),
            message_size: 0,
        };
        unsafe {
            msg_fn(&mut msg_args);
        }
        if msg_args.message.is_null() {
            "unknown PJRT error".to_string()
        } else {
            let bytes =
                unsafe { from_raw_parts(msg_args.message as *const u8, msg_args.message_size) };
            String::from_utf8_lossy(bytes).into_owned()
        }
    } else {
        "unknown PJRT error".to_string()
    };

    if let Some(destroy_fn) = api.PJRT_Error_Destroy {
        let mut destroy_args = PJRT_Error_Destroy_Args {
            struct_size: PJRT_Error_Destroy_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            error,
        };
        unsafe {
            destroy_fn(&mut destroy_args);
        }
    } else if msg == "unknown PJRT error" {
        msg = "unknown PJRT error (and PJRT_Error_Destroy unavailable)".to_string();
    }

    msg
}

#[cfg(test)]
mod pjrt_runtime_tests {
    use crate::pjrt::loader::PjrtRuntime;
    use std::path::Path;

    fn runtime_or_skip() -> Option<PjrtRuntime> {
        let path = Path::new("target/debug/libpjrt_test_plugin.so");
        if !path.is_file() {
            eprintln!("Skipping loader unit test: {} not found", path.display());
            return None;
        }
        match PjrtRuntime::load(path) {
            Ok(rt) => Some(rt),
            Err(err) => {
                eprintln!("Skipping loader unit test: failed to load plugin: {err}");
                None
            }
        }
    }

    #[test]
    fn test_load() {
        let _ = runtime_or_skip();
    }

    #[test]
    fn test_create_client() {
        let Some(rt) = runtime_or_skip() else {
            return;
        };
        let client = rt.create_client();
        assert!(client.is_ok(), "create_client failed: {:?}", client.err());
    }

    #[test]
    fn test_initialize_plugin() {
        let Some(rt) = runtime_or_skip() else {
            return;
        };
        let init = rt.initialize_plugin();
        assert!(init.is_ok(), "initialize_plugin failed: {:?}", init.err());
    }

    #[test]
    fn test_client_devices() {
        let Some(rt) = runtime_or_skip() else {
            return;
        };
        let client = rt.create_client().expect("create_client should succeed");
        assert!(!client.is_null());
        assert!(!rt.client_devices(client).unwrap().is_empty())
    }
}
