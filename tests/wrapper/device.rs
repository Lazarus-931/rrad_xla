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
        eprintln!("Skipping wrapper::device tests: PJRT plugin not found");
        return Ok(None);
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    Ok(Some(rt))
}

#[test]
fn device_basic_metadata_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");
    assert!(!raw_devices[0].is_null(), "first raw device should not be null");

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    assert!(device.id()? >= 0, "device id should be non-negative");
    assert!(!device.kind()?.is_empty(), "device kind should be non-empty");
    Ok(())
}

#[test]
fn device_description_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    let desc = device.description()?;

    assert!(desc.id()? >= 0, "description id should be non-negative");
    assert!(!desc.kind()?.is_empty(), "description kind should be non-empty");
    assert!(
        !desc.to_string()?.is_empty(),
        "description to_string should be non-empty"
    );
    Ok(())
}

#[test]
fn device_is_addressable_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    assert!(
        device.is_addressable()?,
        "first runtime device should be addressable"
    );
    Ok(())
}

