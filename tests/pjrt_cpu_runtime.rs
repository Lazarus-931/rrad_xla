use std::path::Path;

use rrad_xla::pjrt::device::PJRTDevice;
use rrad_xla::pjrt::loader::PjrtRuntime;
use rrad_xla::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;

fn plugin_path_from_env() -> Option<String> {
    std::env::var("PJRT_PLUGIN").ok().filter(|v| !v.is_empty())
}

#[test]
fn cpu_runtime_smoke() -> Result<(), String> {
    let Some(plugin_path) = plugin_path_from_env() else {
        eprintln!("Skipping cpu_runtime_smoke: PJRT_PLUGIN is not set");
        return Ok(());
    };

    if !Path::new(&plugin_path).is_file() {
        return Err(format!(
            "PJRT_PLUGIN does not point to a file: {}",
            plugin_path
        ));
    }

    let rt = PjrtRuntime::load(Path::new(&plugin_path))?;
    rt.initialize_plugin()?;

    let client = rt.create_client_raii()?;
    let platform_name = client.platform_name()?;
    let platform_version = client.platform_version()?;
    assert!(
        !platform_name.is_empty(),
        "expected non-empty platform name"
    );
    assert!(
        !platform_version.is_empty(),
        "expected non-empty platform version"
    );

    let raw_devices = client.devices()?;
    assert!(
        !raw_devices.is_empty(),
        "expected at least one addressable device"
    );

    let first = PJRTDevice::new(&rt, raw_devices[0]);
    let first_id = first.id()?;
    let first_kind = first.kind()?;
    assert!(first_id >= 0, "expected non-negative device id");
    assert!(!first_kind.is_empty(), "expected non-empty device kind");

    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to contain device descriptions"
    );
    assert!(
        !descs
            .iter()
            .all(|d| d.kind().unwrap_or_default().is_empty()),
        "expected at least one non-empty device kind in topology descriptions"
    );

    // Buffer smoke: host->device upload and basic metadata checks.
    let host = [1.0f32, 2.0, 3.0, 4.0];
    let buffer = client.buffer_from_host_slice_copy(
        &host,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[host.len() as i64],
        Some(raw_devices[0]),
    )?;
    let dims = buffer.dimensions()?;
    assert_eq!(dims, vec![host.len() as i64]);
    assert_eq!(
        buffer.element_type()?,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        "expected f32 element type"
    );
    assert!(
        buffer.on_device_size_in_bytes()? >= host.len() * std::mem::size_of::<f32>(),
        "unexpectedly small on-device size"
    );

    Ok(())
}
