#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_char;

pub const PJRT_API_MAJOR: u32 = 0;
pub const PJRT_API_MINOR: u32 = 91;

pub type PJRT_Extension_Type = u32;
pub type PJRT_Error_Code = u32;

pub const PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT: PJRT_Error_Code = 3;
pub const PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND: PJRT_Error_Code = 5;
pub const PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS: PJRT_Error_Code = 6;
pub const PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION: PJRT_Error_Code = 9;
pub const PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED: PJRT_Error_Code = 12;
pub const PJRT_Error_Code_PJRT_Error_Code_INTERNAL: PJRT_Error_Code = 13;
pub const PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE: PJRT_Error_Code = 14;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Extension_Base {
    pub struct_size: usize,
    pub type_: PJRT_Extension_Type,
    pub next: *mut PJRT_Extension_Base,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Api_Version {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub major_version: i32,
    pub minor_version: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Error {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Client {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Executable {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_LoadedExecutable {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Buffer {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Event {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_NamedValue {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Error_Destroy_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub error: *mut PJRT_Error,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Error_Message_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub error: *const PJRT_Error,
    pub message: *const c_char,
    pub message_size: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Error_GetCode_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub error: *const PJRT_Error,
    pub code: PJRT_Error_Code,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Plugin_Initialize_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Plugin_Attributes_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub attributes: *const PJRT_NamedValue,
    pub num_attributes: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Client_Create_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub client: *mut PJRT_Client,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Client_Destroy_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub client: *mut PJRT_Client,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Client_Compile_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub client: *mut PJRT_Client,
    pub executable: *mut *mut PJRT_LoadedExecutable,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_LoadedExecutable_Execute_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub executable: *mut PJRT_LoadedExecutable,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Buffer_Destroy_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub buffer: *mut PJRT_Buffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PJRT_Event_Destroy_Args {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub event: *mut PJRT_Event,
}

pub type PJRT_Error_Destroy = Option<unsafe extern "C" fn(*mut PJRT_Error_Destroy_Args)>;
pub type PJRT_Error_Message = Option<unsafe extern "C" fn(*mut PJRT_Error_Message_Args)>;
pub type PJRT_Error_GetCode =
    Option<unsafe extern "C" fn(*mut PJRT_Error_GetCode_Args) -> *mut PJRT_Error>;
pub type PJRT_Plugin_Initialize =
    Option<unsafe extern "C" fn(*mut PJRT_Plugin_Initialize_Args) -> *mut PJRT_Error>;
pub type PJRT_Plugin_Attributes =
    Option<unsafe extern "C" fn(*mut PJRT_Plugin_Attributes_Args) -> *mut PJRT_Error>;
pub type PJRT_Client_Create =
    Option<unsafe extern "C" fn(*mut PJRT_Client_Create_Args) -> *mut PJRT_Error>;
pub type PJRT_Client_Destroy =
    Option<unsafe extern "C" fn(*mut PJRT_Client_Destroy_Args) -> *mut PJRT_Error>;
pub type PJRT_Client_Compile =
    Option<unsafe extern "C" fn(*mut PJRT_Client_Compile_Args) -> *mut PJRT_Error>;
pub type PJRT_LoadedExecutable_Execute =
    Option<unsafe extern "C" fn(*mut PJRT_LoadedExecutable_Execute_Args) -> *mut PJRT_Error>;
pub type PJRT_Buffer_Destroy =
    Option<unsafe extern "C" fn(*mut PJRT_Buffer_Destroy_Args) -> *mut PJRT_Error>;
pub type PJRT_Event_Destroy =
    Option<unsafe extern "C" fn(*mut PJRT_Event_Destroy_Args) -> *mut PJRT_Error>;

type PJRT_Function = Option<unsafe extern "C" fn()>;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct PJRT_Api {
    pub struct_size: usize,
    pub extension_start: *mut PJRT_Extension_Base,
    pub pjrt_api_version: PJRT_Api_Version,

    pub PJRT_Error_Destroy: PJRT_Error_Destroy,
    pub PJRT_Error_Message: PJRT_Error_Message,
    pub PJRT_Error_GetCode: PJRT_Error_GetCode,
    pub PJRT_Plugin_Initialize: PJRT_Plugin_Initialize,
    pub PJRT_Plugin_Attributes: PJRT_Plugin_Attributes,

    pub PJRT_Event_Destroy: PJRT_Event_Destroy,
    pub PJRT_Event_IsReady: PJRT_Function,
    pub PJRT_Event_Error: PJRT_Function,
    pub PJRT_Event_Await: PJRT_Function,
    pub PJRT_Event_OnReady: PJRT_Function,

    pub PJRT_Client_Create: PJRT_Client_Create,
    pub PJRT_Client_Destroy: PJRT_Client_Destroy,
    pub PJRT_Client_PlatformName: PJRT_Function,
    pub PJRT_Client_ProcessIndex: PJRT_Function,
    pub PJRT_Client_PlatformVersion: PJRT_Function,
    pub PJRT_Client_Devices: PJRT_Function,
    pub PJRT_Client_AddressableDevices: PJRT_Function,
    pub PJRT_Client_LookupDevice: PJRT_Function,
    pub PJRT_Client_LookupAddressableDevice: PJRT_Function,
    pub PJRT_Client_AddressableMemories: PJRT_Function,
    pub PJRT_Client_Compile: PJRT_Client_Compile,
    pub PJRT_Client_DefaultDeviceAssignment: PJRT_Function,
    pub PJRT_Client_BufferFromHostBuffer: PJRT_Function,

    pub PJRT_DeviceDescription_Id: PJRT_Function,
    pub PJRT_DeviceDescription_ProcessIndex: PJRT_Function,
    pub PJRT_DeviceDescription_Attributes: PJRT_Function,
    pub PJRT_DeviceDescription_Kind: PJRT_Function,
    pub PJRT_DeviceDescription_DebugString: PJRT_Function,
    pub PJRT_DeviceDescription_ToString: PJRT_Function,

    pub PJRT_Device_GetDescription: PJRT_Function,
    pub PJRT_Device_IsAddressable: PJRT_Function,
    pub PJRT_Device_LocalHardwareId: PJRT_Function,
    pub PJRT_Device_AddressableMemories: PJRT_Function,
    pub PJRT_Device_DefaultMemory: PJRT_Function,
    pub PJRT_Device_MemoryStats: PJRT_Function,

    pub PJRT_Memory_Id: PJRT_Function,
    pub PJRT_Memory_Kind: PJRT_Function,
    pub PJRT_Memory_DebugString: PJRT_Function,
    pub PJRT_Memory_ToString: PJRT_Function,
    pub PJRT_Memory_AddressableByDevices: PJRT_Function,

    pub PJRT_Executable_Destroy: PJRT_Function,
    pub PJRT_Executable_Name: PJRT_Function,
    pub PJRT_Executable_NumReplicas: PJRT_Function,
    pub PJRT_Executable_NumPartitions: PJRT_Function,
    pub PJRT_Executable_NumOutputs: PJRT_Function,
    pub PJRT_Executable_SizeOfGeneratedCodeInBytes: PJRT_Function,
    pub PJRT_Executable_GetCostAnalysis: PJRT_Function,
    pub PJRT_Executable_OutputMemoryKinds: PJRT_Function,
    pub PJRT_Executable_OptimizedProgram: PJRT_Function,
    pub PJRT_Executable_Serialize: PJRT_Function,

    pub PJRT_LoadedExecutable_Destroy: PJRT_Function,
    pub PJRT_LoadedExecutable_GetExecutable: PJRT_Function,
    pub PJRT_LoadedExecutable_AddressableDevices: PJRT_Function,
    pub PJRT_LoadedExecutable_Delete: PJRT_Function,
    pub PJRT_LoadedExecutable_IsDeleted: PJRT_Function,
    pub PJRT_LoadedExecutable_Execute: PJRT_LoadedExecutable_Execute,
    pub PJRT_Executable_DeserializeAndLoad: PJRT_Function,
    pub PJRT_LoadedExecutable_Fingerprint: PJRT_Function,

    pub PJRT_Buffer_Destroy: PJRT_Buffer_Destroy,
}
