use rrad_pjrt::pjrt::device::PJRTDevice;
use rrad_pjrt::pjrt::loader::PjrtRuntime;
use std::path::{Path, PathBuf};

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
fn general_hardware_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    for device in raw_devices {
        assert!(!device.is_null(), "raw device should not be null");
        let device_ = PJRTDevice::new(&rt, device);
        let hardware_id = device_.local_hardware_id()?;
        let async_tracking_event = device_.create_async_tracking_event("test")?;
        assert!(hardware_id >= 0, "local hardware id should be non-negative");
        assert!(
            !async_tracking_event.raw().is_null(),
            "async tracking event should not be null"
        );
    }
    Ok(())
}

#[test]
fn device_basic_metadata_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");
    assert!(
        !raw_devices[0].is_null(),
        "first raw device should not be null"
    );

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    assert!(device.id()? >= 0, "device id should be non-negative");
    assert!(
        !device.kind()?.is_empty(),
        "device kind should be non-empty"
    );
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
    assert!(
        !desc.kind()?.is_empty(),
        "description kind should be non-empty"
    );
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

#[test]
fn device_default_memory_in_addressable_memories_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    let default_memory = device.default_memory()?;
    assert!(
        !default_memory.is_null(),
        "default_memory should not be null"
    );

    let memories = device.addressable_memories()?;
    assert!(
        memories.iter().any(|m| *m == default_memory),
        "default memory should be part of addressable memories"
    );
    Ok(())
}

#[test]
fn device_debug_and_process_index_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "expected at least one device");

    let device = PJRTDevice::new(&rt, raw_devices[0]);
    let debug_string = device.debug_string()?;
    let to_string = device.to_string()?;
    let process_index = device.process_index()?;

    assert!(!debug_string.is_empty(), "debug_string should not be empty");
    assert!(!to_string.is_empty(), "to_string should not be empty");
    assert!(process_index >= 0, "process_index should be non-negative");
    Ok(())
}
