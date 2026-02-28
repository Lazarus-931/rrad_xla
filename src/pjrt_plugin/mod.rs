//! Rust PJRT Plugin Provider.
//!
//! This module turns `rrad_xla` into a **PJRT plugin shared library** that
//! JAX, TensorFlow, and any other PJRT-aware framework can load directly.
//!
//! ## How it works
//!
//! 1. The framework `dlopen`s our `librrad_xla.so` and calls `GetPjrtApi()`.
//! 2. We return a static `PJRT_Api` table whose function pointers all forward
//!    (proxy) to an underlying *backend* PJRT plugin.
//! 3. The backend plugin path is read from the `RRAD_PJRT_BACKEND_PLUGIN`
//!    environment variable when `PJRT_Plugin_Initialize` is first called.
//!
//! ## JAX usage
//! ```python
//! import jax._src.xla_bridge as xb
//! xb.register_plugin(
//!     "rrad_cpu",
//!     priority=500,
//!     library_path="/path/to/librrad_xla.so",
//!     options=None,
//! )
//! # Set RRAD_PJRT_BACKEND_PLUGIN=/path/to/pjrt_c_api_cpu_plugin.so
//! ```
//!
//! ## TensorFlow usage
//! ```python
//! import tensorflow as tf
//! tf.config.experimental.register_pjrt_plugin(
//!     "rrad_cpu",
//!     library_path="/path/to/librrad_xla.so",
//! )
//! # Set RRAD_PJRT_BACKEND_PLUGIN=/path/to/pjrt_c_api_cpu_plugin.so
//! ```

pub(crate) mod dispatch;
pub(crate) mod error;
pub(crate) mod state;

use crate::pjrt_sys::*;
use dispatch::*;
use std::ptr;

// ---------------------------------------------------------------------------
// Sync wrapper for PJRT_Api
// ---------------------------------------------------------------------------

/// Newtype that makes `PJRT_Api` (which contains raw pointers) safe to place
/// in a `static`.
///
/// # Safety
/// Every pointer field in `PJRT_Api` is either `null` (the `extension_start`
/// fields) or a C function pointer that is valid for the lifetime of the
/// process.  There is no interior mutability.  Accessing the struct from
/// multiple threads is safe under the PJRT spec, which explicitly requires
/// all function pointers to be thread-safe.
struct StaticApi(PJRT_Api);
unsafe impl Sync for StaticApi {}
unsafe impl Send for StaticApi {}

// ---------------------------------------------------------------------------
// The static proxy dispatch table
// ---------------------------------------------------------------------------

static PROXY_API: StaticApi = StaticApi(PJRT_Api {
    struct_size: PJRT_Api_STRUCT_SIZE as usize,
    extension_start: ptr::null_mut(),

    pjrt_api_version: PJRT_Api_Version {
        struct_size: PJRT_Api_Version_STRUCT_SIZE as usize,
        extension_start: ptr::null_mut(),
        major_version: PJRT_API_MAJOR as i32,
        minor_version: PJRT_API_MINOR as i32,
    },

    // --- Error ----------------------------------------------------------
    PJRT_Error_Destroy: Some(pjrt_error_destroy),
    PJRT_Error_Message: Some(pjrt_error_message),
    PJRT_Error_GetCode: Some(pjrt_error_get_code),

    // --- Plugin ---------------------------------------------------------
    PJRT_Plugin_Initialize: Some(pjrt_plugin_initialize),
    PJRT_Plugin_Attributes: Some(pjrt_plugin_attributes),

    // --- Event ----------------------------------------------------------
    PJRT_Event_Destroy: Some(pjrt_event_destroy),
    PJRT_Event_IsReady: Some(pjrt_event_is_ready),
    PJRT_Event_Error: Some(pjrt_event_error),
    PJRT_Event_Await: Some(pjrt_event_await),
    PJRT_Event_OnReady: Some(pjrt_event_on_ready),
    PJRT_Event_Create: Some(pjrt_event_create),
    PJRT_Event_Set: Some(pjrt_event_set),

    // --- Client ---------------------------------------------------------
    PJRT_Client_Create: Some(pjrt_client_create),
    PJRT_Client_Destroy: Some(pjrt_client_destroy),
    PJRT_Client_PlatformName: Some(pjrt_client_platform_name),
    PJRT_Client_ProcessIndex: Some(pjrt_client_process_index),
    PJRT_Client_PlatformVersion: Some(pjrt_client_platform_version),
    PJRT_Client_Devices: Some(pjrt_client_devices),
    PJRT_Client_AddressableDevices: Some(pjrt_client_addressable_devices),
    PJRT_Client_LookupDevice: Some(pjrt_client_lookup_device),
    PJRT_Client_LookupAddressableDevice: Some(pjrt_client_lookup_addressable_device),
    PJRT_Client_AddressableMemories: Some(pjrt_client_addressable_memories),
    PJRT_Client_Compile: Some(pjrt_client_compile),
    PJRT_Client_DefaultDeviceAssignment: Some(pjrt_client_default_device_assignment),
    PJRT_Client_BufferFromHostBuffer: Some(pjrt_client_buffer_from_host_buffer),
    PJRT_Client_CreateViewOfDeviceBuffer: Some(pjrt_client_create_view_of_device_buffer),
    PJRT_Client_TopologyDescription: Some(pjrt_client_topology_description),
    PJRT_Client_CreateBuffersForAsyncHostToDevice: Some(
        pjrt_client_create_buffers_for_async_htod,
    ),
    PJRT_Client_DmaMap: Some(pjrt_client_dma_map),
    PJRT_Client_DmaUnmap: Some(pjrt_client_dma_unmap),
    PJRT_Client_CreateUninitializedBuffer: Some(pjrt_client_create_uninitialized_buffer),
    PJRT_Client_UpdateGlobalProcessInfo: Some(pjrt_client_update_global_process_info),
    PJRT_Client_CreateAliasBuffer: Some(pjrt_client_create_alias_buffer),
    PJRT_Client_FulfillAliasBuffer: Some(pjrt_client_fulfill_alias_buffer),
    PJRT_Client_CreateErrorBuffer: Some(pjrt_client_create_error_buffer),

    // --- DeviceDescription -----------------------------------------------
    PJRT_DeviceDescription_Id: Some(pjrt_device_description_id),
    PJRT_DeviceDescription_ProcessIndex: Some(pjrt_device_description_process_index),
    PJRT_DeviceDescription_Attributes: Some(pjrt_device_description_attributes),
    PJRT_DeviceDescription_Kind: Some(pjrt_device_description_kind),
    PJRT_DeviceDescription_DebugString: Some(pjrt_device_description_debug_string),
    PJRT_DeviceDescription_ToString: Some(pjrt_device_description_to_string),

    // --- Device ----------------------------------------------------------
    PJRT_Device_GetDescription: Some(pjrt_device_get_description),
    PJRT_Device_IsAddressable: Some(pjrt_device_is_addressable),
    PJRT_Device_LocalHardwareId: Some(pjrt_device_local_hardware_id),
    PJRT_Device_AddressableMemories: Some(pjrt_device_addressable_memories),
    PJRT_Device_DefaultMemory: Some(pjrt_device_default_memory),
    PJRT_Device_MemoryStats: Some(pjrt_device_memory_stats),
    PJRT_Device_PoisonExecution: Some(pjrt_device_poison_execution),
    PJRT_Device_CreateAsyncTrackingEvent: Some(pjrt_device_create_async_tracking_event),

    // --- Memory ----------------------------------------------------------
    PJRT_Memory_Id: Some(pjrt_memory_id),
    PJRT_Memory_Kind: Some(pjrt_memory_kind),
    PJRT_Memory_DebugString: Some(pjrt_memory_debug_string),
    PJRT_Memory_ToString: Some(pjrt_memory_to_string),
    PJRT_Memory_AddressableByDevices: Some(pjrt_memory_addressable_by_devices),
    PJRT_Memory_Kind_Id: Some(pjrt_memory_kind_id),

    // --- Executable / LoadedExecutable -----------------------------------
    PJRT_Executable_Destroy: Some(pjrt_executable_destroy),
    PJRT_Executable_Name: Some(pjrt_executable_name),
    PJRT_Executable_NumReplicas: Some(pjrt_executable_num_replicas),
    PJRT_Executable_NumPartitions: Some(pjrt_executable_num_partitions),
    PJRT_Executable_NumOutputs: Some(pjrt_executable_num_outputs),
    PJRT_Executable_SizeOfGeneratedCodeInBytes: Some(
        pjrt_executable_size_of_generated_code_in_bytes,
    ),
    PJRT_Executable_GetCostAnalysis: Some(pjrt_executable_get_cost_analysis),
    PJRT_Executable_OutputMemoryKinds: Some(pjrt_executable_output_memory_kinds),
    PJRT_Executable_OptimizedProgram: Some(pjrt_executable_optimized_program),
    PJRT_Executable_Serialize: Some(pjrt_executable_serialize),
    PJRT_Executable_DeserializeAndLoad: Some(pjrt_executable_deserialize_and_load),
    PJRT_Executable_Fingerprint: Some(pjrt_executable_fingerprint),
    PJRT_Executable_OutputElementTypes: Some(pjrt_executable_output_element_types),
    PJRT_Executable_OutputDimensions: Some(pjrt_executable_output_dimensions),
    PJRT_Executable_GetCompiledMemoryStats: Some(pjrt_executable_get_compiled_memory_stats),
    PJRT_Executable_GetCompileOptions: Some(pjrt_executable_get_compile_options),
    PJRT_LoadedExecutable_Destroy: Some(pjrt_loaded_executable_destroy),
    PJRT_LoadedExecutable_GetExecutable: Some(pjrt_loaded_executable_get_executable),
    PJRT_LoadedExecutable_AddressableDevices: Some(pjrt_loaded_executable_addressable_devices),
    PJRT_LoadedExecutable_Delete: Some(pjrt_loaded_executable_delete),
    PJRT_LoadedExecutable_IsDeleted: Some(pjrt_loaded_executable_is_deleted),
    PJRT_LoadedExecutable_Execute: Some(pjrt_loaded_executable_execute),
    PJRT_LoadedExecutable_Fingerprint: Some(pjrt_loaded_executable_fingerprint),
    PJRT_LoadedExecutable_GetDeviceAssignment: Some(pjrt_loaded_executable_get_device_assignment),

    // --- Buffer ----------------------------------------------------------
    PJRT_Buffer_Destroy: Some(pjrt_buffer_destroy),
    PJRT_Buffer_ElementType: Some(pjrt_buffer_element_type),
    PJRT_Buffer_Dimensions: Some(pjrt_buffer_dimensions),
    PJRT_Buffer_UnpaddedDimensions: Some(pjrt_buffer_unpadded_dimensions),
    PJRT_Buffer_DynamicDimensionIndices: Some(pjrt_buffer_dynamic_dimension_indices),
    PJRT_Buffer_GetMemoryLayout: Some(pjrt_buffer_get_memory_layout),
    PJRT_Buffer_OnDeviceSizeInBytes: Some(pjrt_buffer_on_device_size_in_bytes),
    PJRT_Buffer_Device: Some(pjrt_buffer_device),
    PJRT_Buffer_Memory: Some(pjrt_buffer_memory),
    PJRT_Buffer_Delete: Some(pjrt_buffer_delete),
    PJRT_Buffer_IsDeleted: Some(pjrt_buffer_is_deleted),
    PJRT_Buffer_CopyToDevice: Some(pjrt_buffer_copy_to_device),
    PJRT_Buffer_ToHostBuffer: Some(pjrt_buffer_to_host_buffer),
    PJRT_Buffer_IsOnCpu: Some(pjrt_buffer_is_on_cpu),
    PJRT_Buffer_ReadyEvent: Some(pjrt_buffer_ready_event),
    PJRT_Buffer_UnsafePointer: Some(pjrt_buffer_unsafe_pointer),
    PJRT_Buffer_IncreaseExternalReferenceCount: Some(
        pjrt_buffer_increase_external_reference_count,
    ),
    PJRT_Buffer_DecreaseExternalReferenceCount: Some(
        pjrt_buffer_decrease_external_reference_count,
    ),
    PJRT_Buffer_OpaqueDeviceMemoryDataPointer: Some(
        pjrt_buffer_opaque_device_memory_data_pointer,
    ),
    PJRT_Buffer_CopyToMemory: Some(pjrt_buffer_copy_to_memory),
    PJRT_Buffer_CopyRawToHost: Some(pjrt_buffer_copy_raw_to_host),
    PJRT_Buffer_CopyRawToHostFuture: Some(pjrt_buffer_copy_raw_to_host_future),
    PJRT_Buffer_DonateWithControlDependency: Some(pjrt_buffer_donate_with_control_dependency),

    // --- CopyToDeviceStream ----------------------------------------------
    PJRT_CopyToDeviceStream_Destroy: Some(pjrt_copy_to_device_stream_destroy),
    PJRT_CopyToDeviceStream_AddChunk: Some(pjrt_copy_to_device_stream_add_chunk),
    PJRT_CopyToDeviceStream_TotalBytes: Some(pjrt_copy_to_device_stream_total_bytes),
    PJRT_CopyToDeviceStream_GranuleSize: Some(pjrt_copy_to_device_stream_granule_size),
    PJRT_CopyToDeviceStream_CurrentBytes: Some(pjrt_copy_to_device_stream_current_bytes),

    // --- TopologyDescription ---------------------------------------------
    PJRT_TopologyDescription_Create: Some(pjrt_topology_description_create),
    PJRT_TopologyDescription_Destroy: Some(pjrt_topology_description_destroy),
    PJRT_TopologyDescription_PlatformName: Some(pjrt_topology_description_platform_name),
    PJRT_TopologyDescription_PlatformVersion: Some(pjrt_topology_description_platform_version),
    PJRT_TopologyDescription_GetDeviceDescriptions: Some(
        pjrt_topology_description_get_device_descriptions,
    ),
    PJRT_TopologyDescription_Serialize: Some(pjrt_topology_description_serialize),
    PJRT_TopologyDescription_Attributes: Some(pjrt_topology_description_attributes),
    PJRT_TopologyDescription_Deserialize: Some(pjrt_topology_description_deserialize),

    // --- Compile (standalone) -------------------------------------------
    PJRT_Compile: Some(pjrt_compile),

    // --- ExecuteContext --------------------------------------------------
    PJRT_ExecuteContext_Create: Some(pjrt_execute_context_create),
    PJRT_ExecuteContext_Destroy: Some(pjrt_execute_context_destroy),

    // --- AsyncHostToDeviceTransferManager --------------------------------
    PJRT_AsyncHostToDeviceTransferManager_Destroy: Some(
        pjrt_async_htod_transfer_manager_destroy,
    ),
    PJRT_AsyncHostToDeviceTransferManager_TransferData: Some(
        pjrt_async_htod_transfer_manager_transfer_data,
    ),
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer: Some(
        pjrt_async_htod_transfer_manager_retrieve_buffer,
    ),
    PJRT_AsyncHostToDeviceTransferManager_Device: Some(pjrt_async_htod_transfer_manager_device),
    PJRT_AsyncHostToDeviceTransferManager_BufferCount: Some(
        pjrt_async_htod_transfer_manager_buffer_count,
    ),
    PJRT_AsyncHostToDeviceTransferManager_BufferSize: Some(
        pjrt_async_htod_transfer_manager_buffer_size,
    ),
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError: Some(
        pjrt_async_htod_transfer_manager_set_buffer_error,
    ),
    PJRT_AsyncHostToDeviceTransferManager_AddMetadata: Some(
        pjrt_async_htod_transfer_manager_add_metadata,
    ),
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral: Some(
        pjrt_async_htod_transfer_manager_transfer_literal,
    ),

    // --- AsyncTrackingEvent ----------------------------------------------
    PJRT_AsyncTrackingEvent_Destroy: Some(pjrt_async_tracking_event_destroy),
});

// ---------------------------------------------------------------------------
// C-ABI entry point — exported from the shared library
// ---------------------------------------------------------------------------

/// The single symbol that PJRT consumers (JAX, TensorFlow, …) look for when
/// loading a PJRT plugin `.so`.
///
/// Returns a pointer to the static proxy dispatch table.
#[no_mangle]
pub extern "C" fn GetPjrtApi() -> *const PJRT_Api {
    &raw const PROXY_API.0
}

// ---------------------------------------------------------------------------
// TensorFlow plugin entry point
// ---------------------------------------------------------------------------

/// TensorFlow `tf_pjrt_plugin` discovery symbol.
///
/// TF 2.14+ can discover PJRT plugins via this symbol in addition to (or
/// instead of) `GetPjrtApi`.  It simply delegates to `GetPjrtApi`.
#[no_mangle]
pub extern "C" fn GetPjrtApiForTf() -> *const PJRT_Api {
    GetPjrtApi()
}
