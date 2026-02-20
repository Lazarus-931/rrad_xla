use std::mem::size_of;
use std::path::Path;

use rrad_pjrt::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use rrad_pjrt::rrad_pjrt::error::PJRTError;
use rrad_pjrt::rrad_pjrt::loader::PjrtRuntime;

fn plugin_path_from_env() -> Option<String> {
    std::env::var("PJRT_PLUGIN").ok().filter(|v| !v.is_empty())
}

#[test]
fn cpu_runtime_smoke() -> Result<(), PJRTError<'static>> {
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

    // Leak runtime to satisfy the lifetime carried by PJRTError in this test signature.
    let rt: &'static PjrtRuntime = match PjrtRuntime::load(Path::new(&plugin_path)) {
        Ok(rt) => Box::leak(Box::new(rt)),
        Err(e) => panic!("Failed to load PJRT runtime from PJRT_PLUGIN: {e}"),
    };

    let to_pjrt_err = |msg: String| PJRTError::invalid_arg(rt, msg);

    rt.initialize_plugin().map_err(to_pjrt_err)?;

    let client = rt.create_client().map_err(to_pjrt_err)?;
    let platform_name = client.platform_name().map_err(to_pjrt_err)?;
    let platform_version = client.platform_version().map_err(to_pjrt_err)?;
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

    let first = &raw_devices[0];
    let first_id = first.id().map_err(to_pjrt_err)?;
    let first_kind = first.kind().map_err(to_pjrt_err)?;
    assert!(first_id >= 0, "expected non-negative device id");
    assert!(!first_kind.is_empty(), "expected non-empty device kind");

    let topology = client.topology_description().map_err(to_pjrt_err)?;
    let descs = topology.device_descriptions().map_err(to_pjrt_err)?;
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
    let buffer = client
        .buffer_from_host_slice_copy(
            &host,
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
            &[host.len() as i64],
            Some(raw_devices[0].raw()),
        )
        .map_err(to_pjrt_err)?;

    let dims = buffer.dimensions()?;
    assert_eq!(dims, vec![host.len() as i64]);

    assert_eq!(
        buffer.element_type()?,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        "expected f32 element type"
    );

    assert!(
        buffer.on_device_size_in_bytes().map_err(to_pjrt_err)? >= host.len() * size_of::<f32>(),
        "unexpectedly small on-device size"
    );

    Ok(())
}
