use std::mem::size_of;

use rrad_xla::pjrt_sys::*;

// Minimal ABI/layout expectations for PJRT C API bindings.
//
// If these fail on some platform/arch, you almost certainly generated bindings
// from a different `pjrt_c_api.h` than the one your plugin was compiled with.
#[test]
fn pjrt_struct_sizes_match_header_constants() {
    assert_eq!(
        size_of::<PJRT_Plugin_Initialize_Args>(),
        PJRT_Plugin_Initialize_Args_STRUCT_SIZE as usize
    );

    assert_eq!(
        size_of::<PJRT_Client_Create_Args>(),
        PJRT_Client_Create_Args_STRUCT_SIZE as usize
    );

    assert_eq!(
        size_of::<PJRT_Client_Destroy_Args>(),
        PJRT_Client_Destroy_Args_STRUCT_SIZE as usize
    );

    assert_eq!(
        size_of::<PJRT_Error_Message_Args>(),
        PJRT_Error_Message_Args_STRUCT_SIZE as usize
    );

    assert_eq!(
        size_of::<PJRT_Error_Destroy_Args>(),
        PJRT_Error_Destroy_Args_STRUCT_SIZE as usize
    );
}

