use std::mem::size_of;
use std::path::Path;

use rrad_pjrt::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use rrad_pjrt::rrad_pjrt::loader::PjrtRuntime;

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
        eprintln!(
            "Skipping cpu_runtime_smoke: PJRT_PLUGIN does not point to a file: {plugin_path}"
        );
        return Ok(());
    }

    let rt = PjrtRuntime::load(Path::new(&plugin_path))?;
    rt.initialize_plugin()?;

    let client = rt.create_client().map_err(|e| e.to_string())?;
    let platform_name = client.platform_name().map_err(|e| e.to_string())?;
    let platform_version = client.platform_version().map_err(|e| e.to_string())?;
    assert!(!platform_name.is_empty(), "expected non-empty platform name");
    assert!(
        !platform_version.is_empty(),
        "expected non-empty platform version"
    );

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(
        !raw_devices.is_empty(),
        "expected at least one addressable device"
    );

    let first = &raw_devices[0];
    let first_id = first.id().map_err(|e| e.to_string())?;
    let first_kind = first.kind().map_err(|e| e.to_string())?;
    assert!(first_id >= 0, "expected non-negative device id");
    assert!(!first_kind.is_empty(), "expected non-empty device kind");

    let topology = client.topology_description().map_err(|e| e.to_string())?;
    let descs = topology.device_descriptions().map_err(|e| e.to_string())?;
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

    let host = [1.0f32, 2.0, 3.0, 4.0];
    let buffer = client
        .buffer_from_host_slice_copy(
            &host,
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
            &[host.len() as i64],
            Some(raw_devices[0].raw()),
        )
        .map_err(|e| e.to_string())?;

    let dims = buffer.dimensions().map_err(|e| e.to_string())?;
    assert_eq!(dims, vec![host.len() as i64]);

    assert_eq!(
        buffer.element_type().map_err(|e| e.to_string())?,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        "expected f32 element type"
    );

    assert!(
        buffer
            .on_device_size_in_bytes()
            .map_err(|e| e.to_string())?
            >= host.len() * size_of::<f32>(),
        "unexpectedly small on-device size"
    );

    Ok(())
}
