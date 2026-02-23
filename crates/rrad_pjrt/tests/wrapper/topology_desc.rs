#[cfg(test)]
mod topology_desc_wrapper_tests {
    // Remaining wrapper methods that still need dedicated tests:
    // - PJRTTopologyDescription::create
    // - PJRTTopologyDescription::compile_and_load
    // - PJRTTopologyDescription::compile_and_load_code
    // - PJRTTopologyDescription::raw
    // - PJRTDeviceDescriptionRef::{process_index, debug_string, attributes}

    use std::ptr::null_mut;

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;
    use rrad_pjrt::pjrt_sys::{
        PJRT_Executable_Destroy_Args, PJRT_Executable_Destroy_Args_STRUCT_SIZE, PJRT_Program,
    };
    use rrad_pjrt::topology_desc::PJRTTopologyDescription;

    const MODULE_ADD_ONE: &str = r#"module {
func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2 : tensor<f32>
}}"#;

    #[test]
    fn serialize_deseralize_smoke() -> TestResult {
        with_runtime_and_client(|rt, client| {
            let topology = client.topology_description()?;
            let topology_description = topology.device_descriptions()?;
            let topology_name = topology.platform_name()?;
            let topology_version = topology.platform_version()?;
            let serialized_top = topology.serialize()?;
            let attr_count = topology.attributes()?.len();

            assert!(!topology_description.is_empty());
            assert!(!topology_name.is_empty());
            assert!(
                !serialized_top.is_empty(),
                "serialized topology should not be empty"
            );

            let deserialized = PJRTTopologyDescription::deserialize(rt, &serialized_top)?;
            let _deserialized_description = deserialized.device_descriptions()?;
            let deserialized_name = deserialized.platform_name()?;
            let deserialized_version = deserialized.platform_version()?;
            let deserialized_attr_count = deserialized.attributes()?.len();

            assert_eq!(topology_name, deserialized_name);
            assert_eq!(topology_version, deserialized_version);
            assert_eq!(attr_count, deserialized_attr_count);

            Ok(())
        })
    }

    #[test]
    fn compile_smoke() -> TestResult {
        with_runtime_and_client(|rt, client| {
            let topology = client.topology_description()?;

            let format = "mlir";
            let program = PJRT_Program {
                struct_size: std::mem::size_of::<PJRT_Program>(),
                extension_start: null_mut(),
                code: MODULE_ADD_ONE.as_ptr() as *mut libc::c_char,
                code_size: MODULE_ADD_ONE.len(),
                format: format.as_ptr() as *const libc::c_char,
                format_size: format.len(),
            };

            let executable = topology.compile(client.raw(), &program, &[])?;
            assert!(
                !executable.is_null(),
                "topology.compile should return non-null executable"
            );

            let destroy = rt
                .api()
                .PJRT_Executable_Destroy
                .ok_or("PJRT_Executable_Destroy symbol not found")?;

            let mut args = PJRT_Executable_Destroy_Args {
                struct_size: PJRT_Executable_Destroy_Args_STRUCT_SIZE as usize,
                extension_start: null_mut(),
                executable,
            };

            let err = unsafe { destroy(&mut args) };
            if !err.is_null() {
                return Err("PJRT_Executable_Destroy failed".into());
            }

            Ok(())
        })
    }
}
