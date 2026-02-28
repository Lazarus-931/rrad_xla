//! Transparent proxy dispatch functions for every entry point in `PJRT_Api`.
//!
//! Each public `unsafe extern "C"` function here has the same signature as the
//! corresponding PJRT C-API type alias.  On entry it looks up the backend API
//! via `state::backend_api()` and forwards the call verbatim.  Only the three
//! error-handling functions (`pjrt_error_destroy`, `pjrt_error_message`,
//! `pjrt_error_get_code`) and `pjrt_plugin_initialize` contain non-trivial
//! Rust logic.

use crate::pjrt_plugin::error::{
    destroy_proxy_error, is_proxy_error, make_proxy_error, proxy_error_code,
    proxy_error_from_string, proxy_error_message,
};
use crate::pjrt_plugin::state::{backend_api, load_backend};
use crate::pjrt_sys::*;

// ---------------------------------------------------------------------------
// Helper macros
// ---------------------------------------------------------------------------

/// Forward an args pointer to the corresponding backend function.
/// Returns a proxy FAILED_PRECONDITION error if the backend is not yet loaded,
/// or a proxy UNIMPLEMENTED error if the backend doesn't provide the function.
macro_rules! fwd {
    ($fn_field:ident, $args:expr) => {{
        let backend = match backend_api() {
            Some(b) => b,
            None => {
                return make_proxy_error(
                    PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION,
                    concat!(
                        "rrad_xla proxy: plugin not initialised (call PJRT_Plugin_Initialize first); \
                         attempted: ",
                        stringify!($fn_field)
                    ),
                )
            }
        };
        match unsafe { (*backend).$fn_field } {
            Some(f) => unsafe { f($args) },
            None => make_proxy_error(
                PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED,
                concat!(
                    "rrad_xla proxy: backend does not implement ",
                    stringify!($fn_field)
                ),
            ),
        }
    }};
}

/// Forward to a void-returning backend function.  If the backend is not loaded
/// or doesn't have the function, do nothing.
macro_rules! fwd_void {
    ($fn_field:ident, $args:expr) => {
        if let Some(backend) = backend_api() {
            if let Some(f) = unsafe { (*backend).$fn_field } {
                unsafe { f($args) }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Error functions — these need special handling for proxy-owned errors.
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_error_destroy(args: *mut PJRT_Error_Destroy_Args) {
    if args.is_null() {
        return;
    }
    let err = (*args).error;
    if err.is_null() {
        return;
    }
    if is_proxy_error(err) {
        destroy_proxy_error(err);
        return;
    }
    // Backend-owned error: forward to backend.
    fwd_void!(PJRT_Error_Destroy, args);
}

pub unsafe extern "C" fn pjrt_error_message(args: *mut PJRT_Error_Message_Args) {
    if args.is_null() {
        return;
    }
    let err = (*args).error;
    if is_proxy_error(err) {
        let msg = proxy_error_message(err);
        (*args).message = msg.as_ptr() as *const libc::c_char;
        (*args).message_size = msg.len();
        return;
    }
    fwd_void!(PJRT_Error_Message, args);
}

pub unsafe extern "C" fn pjrt_error_get_code(
    args: *mut PJRT_Error_GetCode_Args,
) -> *mut PJRT_Error {
    if args.is_null() {
        return std::ptr::null_mut();
    }
    if is_proxy_error((*args).error) {
        (*args).code = proxy_error_code((*args).error);
        return std::ptr::null_mut(); // success
    }
    fwd!(PJRT_Error_GetCode, args)
}

// ---------------------------------------------------------------------------
// Plugin lifecycle
// ---------------------------------------------------------------------------

/// Loads the backend plugin from `RRAD_PJRT_BACKEND_PLUGIN`, then delegates
/// to the backend's own `PJRT_Plugin_Initialize`.
pub unsafe extern "C" fn pjrt_plugin_initialize(
    args: *mut PJRT_Plugin_Initialize_Args,
) -> *mut PJRT_Error {
    if let Err(e) = load_backend() {
        return proxy_error_from_string(e);
    }
    // Forward to backend's own initialisation.
    let backend = match backend_api() {
        Some(b) => b,
        None => {
            return make_proxy_error(
                PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
                "rrad_xla proxy: backend vanished after load (impossible)",
            )
        }
    };
    match (*backend).PJRT_Plugin_Initialize {
        Some(f) => f(args),
        None => std::ptr::null_mut(), // backend doesn't require explicit init
    }
}

pub unsafe extern "C" fn pjrt_plugin_attributes(
    args: *mut PJRT_Plugin_Attributes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Plugin_Attributes, args)
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_event_destroy(
    args: *mut PJRT_Event_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Event_Destroy, args)
}

pub unsafe extern "C" fn pjrt_event_is_ready(
    args: *mut PJRT_Event_IsReady_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Event_IsReady, args)
}

pub unsafe extern "C" fn pjrt_event_error(args: *mut PJRT_Event_Error_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Event_Error, args)
}

pub unsafe extern "C" fn pjrt_event_await(args: *mut PJRT_Event_Await_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Event_Await, args)
}

pub unsafe extern "C" fn pjrt_event_on_ready(
    args: *mut PJRT_Event_OnReady_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Event_OnReady, args)
}

pub unsafe extern "C" fn pjrt_event_create(
    args: *mut PJRT_Event_Create_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Event_Create, args)
}

pub unsafe extern "C" fn pjrt_event_set(args: *mut PJRT_Event_Set_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Event_Set, args)
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_client_create(
    args: *mut PJRT_Client_Create_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_Create, args)
}

pub unsafe extern "C" fn pjrt_client_destroy(
    args: *mut PJRT_Client_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_Destroy, args)
}

pub unsafe extern "C" fn pjrt_client_platform_name(
    args: *mut PJRT_Client_PlatformName_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_PlatformName, args)
}

pub unsafe extern "C" fn pjrt_client_process_index(
    args: *mut PJRT_Client_ProcessIndex_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_ProcessIndex, args)
}

pub unsafe extern "C" fn pjrt_client_platform_version(
    args: *mut PJRT_Client_PlatformVersion_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_PlatformVersion, args)
}

pub unsafe extern "C" fn pjrt_client_devices(
    args: *mut PJRT_Client_Devices_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_Devices, args)
}

pub unsafe extern "C" fn pjrt_client_addressable_devices(
    args: *mut PJRT_Client_AddressableDevices_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_AddressableDevices, args)
}

pub unsafe extern "C" fn pjrt_client_lookup_device(
    args: *mut PJRT_Client_LookupDevice_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_LookupDevice, args)
}

pub unsafe extern "C" fn pjrt_client_lookup_addressable_device(
    args: *mut PJRT_Client_LookupAddressableDevice_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_LookupAddressableDevice, args)
}

pub unsafe extern "C" fn pjrt_client_addressable_memories(
    args: *mut PJRT_Client_AddressableMemories_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_AddressableMemories, args)
}

pub unsafe extern "C" fn pjrt_client_compile(
    args: *mut PJRT_Client_Compile_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_Compile, args)
}

pub unsafe extern "C" fn pjrt_client_default_device_assignment(
    args: *mut PJRT_Client_DefaultDeviceAssignment_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_DefaultDeviceAssignment, args)
}

pub unsafe extern "C" fn pjrt_client_buffer_from_host_buffer(
    args: *mut PJRT_Client_BufferFromHostBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_BufferFromHostBuffer, args)
}

pub unsafe extern "C" fn pjrt_client_create_view_of_device_buffer(
    args: *mut PJRT_Client_CreateViewOfDeviceBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_CreateViewOfDeviceBuffer, args)
}

pub unsafe extern "C" fn pjrt_client_topology_description(
    args: *mut PJRT_Client_TopologyDescription_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_TopologyDescription, args)
}

pub unsafe extern "C" fn pjrt_client_create_buffers_for_async_htod(
    args: *mut PJRT_Client_CreateBuffersForAsyncHostToDevice_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_CreateBuffersForAsyncHostToDevice, args)
}

pub unsafe extern "C" fn pjrt_client_dma_map(
    args: *mut PJRT_Client_DmaMap_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_DmaMap, args)
}

pub unsafe extern "C" fn pjrt_client_dma_unmap(
    args: *mut PJRT_Client_DmaUnmap_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_DmaUnmap, args)
}

pub unsafe extern "C" fn pjrt_client_create_uninitialized_buffer(
    args: *mut PJRT_Client_CreateUninitializedBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_CreateUninitializedBuffer, args)
}

pub unsafe extern "C" fn pjrt_client_update_global_process_info(
    args: *mut PJRT_Client_UpdateGlobalProcessInfo_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_UpdateGlobalProcessInfo, args)
}

pub unsafe extern "C" fn pjrt_client_create_alias_buffer(
    args: *mut PJRT_Client_CreateAliasBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_CreateAliasBuffer, args)
}

pub unsafe extern "C" fn pjrt_client_fulfill_alias_buffer(
    args: *mut PJRT_Client_FulfillAliasBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_FulfillAliasBuffer, args)
}

pub unsafe extern "C" fn pjrt_client_create_error_buffer(
    args: *mut PJRT_Client_CreateErrorBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Client_CreateErrorBuffer, args)
}

// ---------------------------------------------------------------------------
// DeviceDescription
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_device_description_id(
    args: *mut PJRT_DeviceDescription_Id_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_Id, args)
}

pub unsafe extern "C" fn pjrt_device_description_process_index(
    args: *mut PJRT_DeviceDescription_ProcessIndex_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_ProcessIndex, args)
}

pub unsafe extern "C" fn pjrt_device_description_attributes(
    args: *mut PJRT_DeviceDescription_Attributes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_Attributes, args)
}

pub unsafe extern "C" fn pjrt_device_description_kind(
    args: *mut PJRT_DeviceDescription_Kind_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_Kind, args)
}

pub unsafe extern "C" fn pjrt_device_description_debug_string(
    args: *mut PJRT_DeviceDescription_DebugString_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_DebugString, args)
}

pub unsafe extern "C" fn pjrt_device_description_to_string(
    args: *mut PJRT_DeviceDescription_ToString_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_DeviceDescription_ToString, args)
}

// ---------------------------------------------------------------------------
// Device
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_device_get_description(
    args: *mut PJRT_Device_GetDescription_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_GetDescription, args)
}

pub unsafe extern "C" fn pjrt_device_is_addressable(
    args: *mut PJRT_Device_IsAddressable_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_IsAddressable, args)
}

pub unsafe extern "C" fn pjrt_device_local_hardware_id(
    args: *mut PJRT_Device_LocalHardwareId_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_LocalHardwareId, args)
}

pub unsafe extern "C" fn pjrt_device_addressable_memories(
    args: *mut PJRT_Device_AddressableMemories_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_AddressableMemories, args)
}

pub unsafe extern "C" fn pjrt_device_default_memory(
    args: *mut PJRT_Device_DefaultMemory_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_DefaultMemory, args)
}

pub unsafe extern "C" fn pjrt_device_memory_stats(
    args: *mut PJRT_Device_MemoryStats_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_MemoryStats, args)
}

pub unsafe extern "C" fn pjrt_device_poison_execution(
    args: *mut PJRT_Device_PoisonExecution_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_PoisonExecution, args)
}

pub unsafe extern "C" fn pjrt_device_create_async_tracking_event(
    args: *mut PJRT_Device_CreateAsyncTrackingEvent_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Device_CreateAsyncTrackingEvent, args)
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_memory_id(args: *mut PJRT_Memory_Id_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_Id, args)
}

pub unsafe extern "C" fn pjrt_memory_kind(args: *mut PJRT_Memory_Kind_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_Kind, args)
}

pub unsafe extern "C" fn pjrt_memory_debug_string(
    args: *mut PJRT_Memory_DebugString_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_DebugString, args)
}

pub unsafe extern "C" fn pjrt_memory_to_string(
    args: *mut PJRT_Memory_ToString_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_ToString, args)
}

pub unsafe extern "C" fn pjrt_memory_addressable_by_devices(
    args: *mut PJRT_Memory_AddressableByDevices_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_AddressableByDevices, args)
}

pub unsafe extern "C" fn pjrt_memory_kind_id(
    args: *mut PJRT_Memory_Kind_Id_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Memory_Kind_Id, args)
}

// ---------------------------------------------------------------------------
// Executable / LoadedExecutable
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_executable_destroy(
    args: *mut PJRT_Executable_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_Destroy, args)
}

pub unsafe extern "C" fn pjrt_executable_name(
    args: *mut PJRT_Executable_Name_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_Name, args)
}

pub unsafe extern "C" fn pjrt_executable_num_replicas(
    args: *mut PJRT_Executable_NumReplicas_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_NumReplicas, args)
}

pub unsafe extern "C" fn pjrt_executable_num_partitions(
    args: *mut PJRT_Executable_NumPartitions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_NumPartitions, args)
}

pub unsafe extern "C" fn pjrt_executable_num_outputs(
    args: *mut PJRT_Executable_NumOutputs_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_NumOutputs, args)
}

pub unsafe extern "C" fn pjrt_executable_size_of_generated_code_in_bytes(
    args: *mut PJRT_Executable_SizeOfGeneratedCodeInBytes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_SizeOfGeneratedCodeInBytes, args)
}

pub unsafe extern "C" fn pjrt_executable_get_cost_analysis(
    args: *mut PJRT_Executable_GetCostAnalysis_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_GetCostAnalysis, args)
}

pub unsafe extern "C" fn pjrt_executable_output_memory_kinds(
    args: *mut PJRT_Executable_OutputMemoryKinds_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_OutputMemoryKinds, args)
}

pub unsafe extern "C" fn pjrt_executable_optimized_program(
    args: *mut PJRT_Executable_OptimizedProgram_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_OptimizedProgram, args)
}

pub unsafe extern "C" fn pjrt_executable_serialize(
    args: *mut PJRT_Executable_Serialize_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_Serialize, args)
}

pub unsafe extern "C" fn pjrt_executable_deserialize_and_load(
    args: *mut PJRT_Executable_DeserializeAndLoad_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_DeserializeAndLoad, args)
}

pub unsafe extern "C" fn pjrt_executable_fingerprint(
    args: *mut PJRT_Executable_Fingerprint_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_Fingerprint, args)
}

pub unsafe extern "C" fn pjrt_executable_output_element_types(
    args: *mut PJRT_Executable_OutputElementTypes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_OutputElementTypes, args)
}

pub unsafe extern "C" fn pjrt_executable_output_dimensions(
    args: *mut PJRT_Executable_OutputDimensions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_OutputDimensions, args)
}

pub unsafe extern "C" fn pjrt_executable_get_compiled_memory_stats(
    args: *mut PJRT_Executable_GetCompiledMemoryStats_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_GetCompiledMemoryStats, args)
}

pub unsafe extern "C" fn pjrt_executable_get_compile_options(
    args: *mut PJRT_Executable_GetCompileOptions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Executable_GetCompileOptions, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_destroy(
    args: *mut PJRT_LoadedExecutable_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_Destroy, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_get_executable(
    args: *mut PJRT_LoadedExecutable_GetExecutable_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_GetExecutable, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_addressable_devices(
    args: *mut PJRT_LoadedExecutable_AddressableDevices_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_AddressableDevices, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_delete(
    args: *mut PJRT_LoadedExecutable_Delete_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_Delete, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_is_deleted(
    args: *mut PJRT_LoadedExecutable_IsDeleted_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_IsDeleted, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_execute(
    args: *mut PJRT_LoadedExecutable_Execute_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_Execute, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_fingerprint(
    args: *mut PJRT_LoadedExecutable_Fingerprint_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_Fingerprint, args)
}

pub unsafe extern "C" fn pjrt_loaded_executable_get_device_assignment(
    args: *mut PJRT_LoadedExecutable_GetDeviceAssignment_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_LoadedExecutable_GetDeviceAssignment, args)
}

// ---------------------------------------------------------------------------
// Buffer
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_buffer_destroy(
    args: *mut PJRT_Buffer_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_Destroy, args)
}

pub unsafe extern "C" fn pjrt_buffer_element_type(
    args: *mut PJRT_Buffer_ElementType_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_ElementType, args)
}

pub unsafe extern "C" fn pjrt_buffer_dimensions(
    args: *mut PJRT_Buffer_Dimensions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_Dimensions, args)
}

pub unsafe extern "C" fn pjrt_buffer_unpadded_dimensions(
    args: *mut PJRT_Buffer_UnpaddedDimensions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_UnpaddedDimensions, args)
}

pub unsafe extern "C" fn pjrt_buffer_dynamic_dimension_indices(
    args: *mut PJRT_Buffer_DynamicDimensionIndices_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_DynamicDimensionIndices, args)
}

pub unsafe extern "C" fn pjrt_buffer_get_memory_layout(
    args: *mut PJRT_Buffer_GetMemoryLayout_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_GetMemoryLayout, args)
}

pub unsafe extern "C" fn pjrt_buffer_on_device_size_in_bytes(
    args: *mut PJRT_Buffer_OnDeviceSizeInBytes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_OnDeviceSizeInBytes, args)
}

pub unsafe extern "C" fn pjrt_buffer_device(
    args: *mut PJRT_Buffer_Device_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_Device, args)
}

pub unsafe extern "C" fn pjrt_buffer_memory(
    args: *mut PJRT_Buffer_Memory_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_Memory, args)
}

pub unsafe extern "C" fn pjrt_buffer_delete(
    args: *mut PJRT_Buffer_Delete_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_Delete, args)
}

pub unsafe extern "C" fn pjrt_buffer_is_deleted(
    args: *mut PJRT_Buffer_IsDeleted_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_IsDeleted, args)
}

pub unsafe extern "C" fn pjrt_buffer_copy_to_device(
    args: *mut PJRT_Buffer_CopyToDevice_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_CopyToDevice, args)
}

pub unsafe extern "C" fn pjrt_buffer_to_host_buffer(
    args: *mut PJRT_Buffer_ToHostBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_ToHostBuffer, args)
}

pub unsafe extern "C" fn pjrt_buffer_is_on_cpu(
    args: *mut PJRT_Buffer_IsOnCpu_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_IsOnCpu, args)
}

pub unsafe extern "C" fn pjrt_buffer_ready_event(
    args: *mut PJRT_Buffer_ReadyEvent_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_ReadyEvent, args)
}

pub unsafe extern "C" fn pjrt_buffer_unsafe_pointer(
    args: *mut PJRT_Buffer_UnsafePointer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_UnsafePointer, args)
}

pub unsafe extern "C" fn pjrt_buffer_increase_external_reference_count(
    args: *mut PJRT_Buffer_IncreaseExternalReferenceCount_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_IncreaseExternalReferenceCount, args)
}

pub unsafe extern "C" fn pjrt_buffer_decrease_external_reference_count(
    args: *mut PJRT_Buffer_DecreaseExternalReferenceCount_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_DecreaseExternalReferenceCount, args)
}

pub unsafe extern "C" fn pjrt_buffer_opaque_device_memory_data_pointer(
    args: *mut PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_OpaqueDeviceMemoryDataPointer, args)
}

pub unsafe extern "C" fn pjrt_buffer_copy_to_memory(
    args: *mut PJRT_Buffer_CopyToMemory_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_CopyToMemory, args)
}

pub unsafe extern "C" fn pjrt_buffer_copy_raw_to_host(
    args: *mut PJRT_Buffer_CopyRawToHost_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_CopyRawToHost, args)
}

pub unsafe extern "C" fn pjrt_buffer_copy_raw_to_host_future(
    args: *mut PJRT_Buffer_CopyRawToHostFuture_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_CopyRawToHostFuture, args)
}

pub unsafe extern "C" fn pjrt_buffer_donate_with_control_dependency(
    args: *mut PJRT_Buffer_DonateWithControlDependency_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_Buffer_DonateWithControlDependency, args)
}

// ---------------------------------------------------------------------------
// CopyToDeviceStream
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_copy_to_device_stream_destroy(
    args: *mut PJRT_CopyToDeviceStream_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_CopyToDeviceStream_Destroy, args)
}

pub unsafe extern "C" fn pjrt_copy_to_device_stream_add_chunk(
    args: *mut PJRT_CopyToDeviceStream_AddChunk_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_CopyToDeviceStream_AddChunk, args)
}

pub unsafe extern "C" fn pjrt_copy_to_device_stream_total_bytes(
    args: *mut PJRT_CopyToDeviceStream_TotalBytes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_CopyToDeviceStream_TotalBytes, args)
}

pub unsafe extern "C" fn pjrt_copy_to_device_stream_granule_size(
    args: *mut PJRT_CopyToDeviceStream_GranuleSize_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_CopyToDeviceStream_GranuleSize, args)
}

pub unsafe extern "C" fn pjrt_copy_to_device_stream_current_bytes(
    args: *mut PJRT_CopyToDeviceStream_CurrentBytes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_CopyToDeviceStream_CurrentBytes, args)
}

// ---------------------------------------------------------------------------
// TopologyDescription
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_topology_description_create(
    args: *mut PJRT_TopologyDescription_Create_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_Create, args)
}

pub unsafe extern "C" fn pjrt_topology_description_destroy(
    args: *mut PJRT_TopologyDescription_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_Destroy, args)
}

pub unsafe extern "C" fn pjrt_topology_description_platform_name(
    args: *mut PJRT_TopologyDescription_PlatformName_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_PlatformName, args)
}

pub unsafe extern "C" fn pjrt_topology_description_platform_version(
    args: *mut PJRT_TopologyDescription_PlatformVersion_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_PlatformVersion, args)
}

pub unsafe extern "C" fn pjrt_topology_description_get_device_descriptions(
    args: *mut PJRT_TopologyDescription_GetDeviceDescriptions_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_GetDeviceDescriptions, args)
}

pub unsafe extern "C" fn pjrt_topology_description_serialize(
    args: *mut PJRT_TopologyDescription_Serialize_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_Serialize, args)
}

pub unsafe extern "C" fn pjrt_topology_description_attributes(
    args: *mut PJRT_TopologyDescription_Attributes_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_Attributes, args)
}

pub unsafe extern "C" fn pjrt_topology_description_deserialize(
    args: *mut PJRT_TopologyDescription_Deserialize_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_TopologyDescription_Deserialize, args)
}

// ---------------------------------------------------------------------------
// Compile (standalone, topology-based)
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_compile(args: *mut PJRT_Compile_Args) -> *mut PJRT_Error {
    fwd!(PJRT_Compile, args)
}

// ---------------------------------------------------------------------------
// ExecuteContext
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_execute_context_create(
    args: *mut PJRT_ExecuteContext_Create_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_ExecuteContext_Create, args)
}

pub unsafe extern "C" fn pjrt_execute_context_destroy(
    args: *mut PJRT_ExecuteContext_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_ExecuteContext_Destroy, args)
}

// ---------------------------------------------------------------------------
// AsyncHostToDeviceTransferManager
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_destroy(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_Destroy, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_transfer_data(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_TransferData_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_TransferData, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_retrieve_buffer(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_device(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_Device_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_Device, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_buffer_count(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_BufferCount, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_buffer_size(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_BufferSize, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_set_buffer_error(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_SetBufferError, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_add_metadata(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_AddMetadata, args)
}

pub unsafe extern "C" fn pjrt_async_htod_transfer_manager_transfer_literal(
    args: *mut PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncHostToDeviceTransferManager_TransferLiteral, args)
}

// ---------------------------------------------------------------------------
// AsyncTrackingEvent
// ---------------------------------------------------------------------------

pub unsafe extern "C" fn pjrt_async_tracking_event_destroy(
    args: *mut PJRT_AsyncTrackingEvent_Destroy_Args,
) -> *mut PJRT_Error {
    fwd!(PJRT_AsyncTrackingEvent_Destroy, args)
}
