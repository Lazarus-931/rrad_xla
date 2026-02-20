use rrad_pjrt::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use rrad_pjrt::rrad_pjrt::topology_desc::PJRTTopologyDescription;

use super::tools::runtime_or_skip;

const MODULE_ADD_ONE: &str = r#"module {
func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2 : tensor<f32>
}}"#;

#[test]
fn unified_topology_serialize_roundtrip_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let topology = client.topology_description()?;
    let before_name = topology.platform_name()?;
    let serialized = topology.serialize()?;
    assert!(
        !serialized.is_empty(),
        "serialized topology should be non-empty"
    );

    let deserialized = PJRTTopologyDescription::deserialize(&rt, &serialized)?;
    let after_name = deserialized.platform_name()?;
    assert_eq!(
        before_name, after_name,
        "platform name should round-trip through topology serialization"
    );
    Ok(())
}

#[test]
fn unified_compile_execute_metadata_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let executable = client
        .compile(MODULE_ADD_ONE, "mlir", &[])
        .map_err(|e| e.to_string())?;
    assert!(
        executable.num_replicas()? >= 1,
        "num_replicas should be >= 1"
    );
    assert!(
        executable.num_partitions()? >= 1,
        "num_partitions should be >= 1"
    );
    let output_types = executable.output_element_types()?;
    assert_eq!(
        output_types,
        vec![PJRT_Buffer_Type_PJRT_Buffer_Type_F32],
        "single-output add-one program should return one F32 output"
    );

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let input = [3.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(raw_devices[0].raw()),
    )?;

    let (outputs, done) = executable.execute(&[&input_buffer])?;
    done.ok()?;
    assert_eq!(outputs.len(), 1, "expected one output buffer");

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 4.0).abs() > 1e-6 {
        return Err(format!("expected 4.0, got {out}"));
    }
    Ok(())
}
