use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::sync::Once;

use crate::sys::pjrt::{
    PJRT_API_MAJOR, PJRT_API_MINOR, PJRT_Api, PJRT_Buffer_Destroy_Args, PJRT_Client_Compile_Args,
    PJRT_Client_Create_Args, PJRT_Client_Destroy_Args, PJRT_Error,
    PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT, PJRT_Event_Destroy_Args,
    PJRT_LoadedExecutable_Execute_Args, PJRT_Plugin_Attributes_Args,
    PJRT_Plugin_Initialize_Args,
};

use crate::error::{
    new_error, new_runtime_error, pjrt_error_destroy, pjrt_error_get_code, pjrt_error_message,
};
use crate::runtime_util::error::PJRTRuntimeError;

static API_INIT: Once = Once::new();
static mut API_PTR: *const PJRT_Api = null();

pub fn get_pjrt_api() -> *const PJRT_Api {
    unsafe {
        API_INIT.call_once(|| {
            let api = build_api_table();
            API_PTR = Box::into_raw(Box::new(api));
        });
        API_PTR
    }
}

fn build_api_table() -> PJRT_Api {
    // SAFETY: PJRT_Api is a C table where null function pointers are valid defaults.
    let mut api: PJRT_Api = unsafe { zeroed() };

    api.struct_size = core::mem::size_of::<PJRT_Api>();
    api.extension_start = null_mut();

    api.pjrt_api_version.struct_size = core::mem::size_of::<crate::sys::pjrt::PJRT_Api_Version>();
    api.pjrt_api_version.extension_start = null_mut();
    api.pjrt_api_version.major_version = PJRT_API_MAJOR as i32;
    api.pjrt_api_version.minor_version = PJRT_API_MINOR as i32;

    api.PJRT_Error_Destroy = Some(pjrt_error_destroy);
    api.PJRT_Error_Message = Some(pjrt_error_message);
    api.PJRT_Error_GetCode = Some(pjrt_error_get_code);

    api.PJRT_Plugin_Initialize = Some(pjrt_plugin_initialize);
    api.PJRT_Plugin_Attributes = Some(pjrt_plugin_attributes);

    // Minimal non-null surface for loader validation and basic lifecycle.
    api.PJRT_Client_Create = Some(pjrt_client_create);
    api.PJRT_Client_Destroy = Some(pjrt_client_destroy);
    api.PJRT_Client_Compile = Some(pjrt_client_compile);
    api.PJRT_LoadedExecutable_Execute = Some(pjrt_loaded_executable_execute);
    api.PJRT_Buffer_Destroy = Some(pjrt_buffer_destroy);
    api.PJRT_Event_Destroy = Some(pjrt_event_destroy);

    api
}

unsafe extern "C" fn pjrt_plugin_initialize(
    args: *mut PJRT_Plugin_Initialize_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Plugin_Initialize args is null",
        );
    }
    null_mut()
}

unsafe extern "C" fn pjrt_plugin_attributes(
    args: *mut PJRT_Plugin_Attributes_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Plugin_Attributes args is null",
        );
    }

    // SAFETY: args is checked non-null above.
    let args = unsafe { &mut *args };
    args.attributes = null();
    args.num_attributes = 0;
    null_mut()
}

unsafe extern "C" fn pjrt_client_create(args: *mut PJRT_Client_Create_Args) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Client_Create args is null",
        );
    }

    new_runtime_error(&PJRTRuntimeError::unimplemented(
        "rrad_pjrt_runtime: PJRT_Client_Create is not implemented yet",
    ))
}

unsafe extern "C" fn pjrt_client_destroy(args: *mut PJRT_Client_Destroy_Args) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Client_Destroy args is null",
        );
    }

    // No owned runtime client object yet, so this is a no-op placeholder.
    null_mut()
}

unsafe extern "C" fn pjrt_client_compile(
    args: *mut PJRT_Client_Compile_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Client_Compile args is null",
        );
    }

    new_runtime_error(&PJRTRuntimeError::unimplemented(
        "rrad_pjrt_runtime: PJRT_Client_Compile is not implemented yet",
    ))
}

unsafe extern "C" fn pjrt_loaded_executable_execute(
    args: *mut PJRT_LoadedExecutable_Execute_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_LoadedExecutable_Execute args is null",
        );
    }

    new_runtime_error(&PJRTRuntimeError::unimplemented(
        "rrad_pjrt_runtime: PJRT_LoadedExecutable_Execute is not implemented yet",
    ))
}

unsafe extern "C" fn pjrt_buffer_destroy(args: *mut PJRT_Buffer_Destroy_Args) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Buffer_Destroy args is null",
        );
    }
    null_mut()
}

unsafe extern "C" fn pjrt_event_destroy(args: *mut PJRT_Event_Destroy_Args) -> *mut PJRT_Error {
    if args.is_null() {
        return new_error(
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            "PJRT_Event_Destroy args is null",
        );
    }
    null_mut()
}
