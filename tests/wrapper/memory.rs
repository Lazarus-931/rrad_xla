use rrad_pjrt::rrad_pjrt::error::PJRTError;
use super::tools::{runtime_or_skip, TestResult};


#[test]
fn memory_id_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()
        .map_err(|e| PJRTError::invalid_arg(&rt, "Failed to create client"))?;
    let memories = client.addressable_memories()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let id = memory.id()?;
        let kind = memory.kind()?;
        assert!(id > 0, "memory id should be positive, got {id}");
        assert!(!kind.is_empty(), "memory kind should be non-empty");
    }

    Ok(())
}

#[test]
fn memory_kind_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()
        .map_err(|e| PJRTError::invalid_arg(&rt, "Failed to create client"))?;
    let memories = client.addressable_memories()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let kind = memory.kind()?;
        let kind_id = memory.kind_id()?;
        assert!(!kind.is_empty(), "memory kind should be non-empty");
        assert!(kind_id >= 0, "memory kind_id should be non-negative");
    }

    Ok(())
}

#[test]
fn memory_to_string_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()
        .map_err(|e| PJRTError::invalid_arg(&rt, "Failed to create client"))?;
    let memories = client.addressable_memories()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let to_string = memory.to_string()?;
        assert!(
            !to_string.is_empty(),
            "memory to_string should be non-empty"
        );
    }

    Ok(())
}

#[test]
fn memory_debug_string_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()
        .map_err(|e| PJRTError::invalid_arg(&rt, "Failed to create client"))?;
    let memories = client.addressable_memories()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let debug_string = memory.debug_string()?;
        assert!(
            !debug_string.is_empty(),
            "memory debug_string should be non-empty"
        );
    }

    Ok(())
}

#[test]
fn memory_addressable_by_device_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()
        .map_err(|e| PJRTError::invalid_arg(&rt, "Failed to create client"))?;
    let memories = client.addressable_memories()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let devices = memory.addressable_by_device()?;
        assert!(
            !devices.is_empty(),
            "memory should be addressable by at least one device"
        );
        for device in &devices {
            assert!(
                device.is_addressable()?,
                "device returned by memory.addressable_by_device should be addressable"
            );
        }
    }

    Ok(())
}
