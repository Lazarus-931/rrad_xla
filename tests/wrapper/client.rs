use std::path::{Path, PathBuf};

use rrad_xla::pjrt::device::PJRTDevice;
use rrad_xla::pjrt::loader::PjrtRuntime;

fn resolve_plugin_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PJRT_PLUGIN") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }

    let candidates = [
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.so",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.dylib",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin",
    ];
    for candidate in candidates {
        let p = Path::new(candidate).to_path_buf();
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

fn runtime_or_skip() -> Result<Option<PjrtRuntime>, String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping wrapper::client tests: PJRT plugin not found");
        return Ok(None);
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    Ok(Some(rt))
}

#[test]
fn client_basic_metadata_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let platform_name = client.platform_name()?;
    let platform_version = client.platform_version()?;
    let process_index = client.process_index()?;

    assert!(!platform_name.is_empty(), "platform_name should not be empty");
    assert!(
        !platform_version.is_empty(),
        "platform_version should not be empty"
    );
    assert!(process_index >= 0, "process_index should be non-negative");
    Ok(())
}

#[test]
fn client_lookup_first_device_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");

    let first_device_ref = PJRTDevice::new(&rt, raw_devices[0]);
    let first_id = first_device_ref.id()?;
    let local_hardware_id = first_device_ref.local_hardware_id()?;

    let by_id = client.lookup_device(first_id)?;
    assert!(!by_id.is_null(), "lookup_device returned null");

    let by_local = client.lookup_addressable_device(local_hardware_id)?;
    assert!(!by_local.is_null(), "lookup_addressable_device returned null");
    Ok(())
}

#[test]
fn client_topology_and_assignment_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let topology = client.topology_description()?;
    let platform_name = topology.platform_name()?;
    let descs = topology.device_descriptions()?;

    assert!(
        !platform_name.is_empty(),
        "topology platform_name should not be empty"
    );
    assert!(
        !descs.is_empty(),
        "topology should include at least one device description"
    );

    let assignment = client.default_device_assignment(1, 1)?;
    assert!(
        !assignment.is_empty(),
        "default device assignment for 1x1 should not be empty"
    );
    Ok(())
}


