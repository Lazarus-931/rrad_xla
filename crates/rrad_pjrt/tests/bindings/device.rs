#[cfg(test)]
mod device_bindings_tests {
    // Remaining bindings methods that still need dedicated tests:
    // - PJRTDevice::memory_stats
    // - PJRTDevice::poison_execution
    // - PJRTDevice::default_memory_ref
    // - PJRTDevice::attributes
    // - PJRTDevice::debug_error
    // - PJRTDeviceDescriptionRef::{process_index, debug_string, attributes}

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;

    #[test]
    fn general_hardware_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            for device in raw_devices {
                assert!(!device.is_null(), "raw device should not be null");
                let hardware_id = device.local_hardware_id()?;
                let async_tracking_event = device.create_async_tracking_event("test")?;
                assert!(hardware_id >= 0, "local hardware id should be non-negative");
                assert!(
                    !async_tracking_event.raw().is_null(),
                    "async tracking event should not be null"
                );
            }
            Ok(())
        })
    }

    #[test]
    fn device_basic_metadata_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            assert!(!raw_devices.is_empty(), "expected at least one device");
            assert!(
                !raw_devices[0].is_null(),
                "first raw device should not be null"
            );

            let device = &raw_devices[0];
            assert!(device.id()? >= 0, "device id should be non-negative");
            assert!(
                !device.kind()?.is_empty(),
                "device kind should be non-empty"
            );
            Ok(())
        })
    }

    #[test]
    fn device_description_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            assert!(!raw_devices.is_empty(), "expected at least one device");

            let device = &raw_devices[0];
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
        })
    }

    #[test]
    fn device_is_addressable_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            assert!(!raw_devices.is_empty(), "expected at least one device");

            let device = &raw_devices[0];
            assert!(
                device.is_addressable()?,
                "first runtime device should be addressable"
            );
            Ok(())
        })
    }

    #[test]
    fn device_default_memory_in_addressable_memories_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            assert!(!raw_devices.is_empty(), "expected at least one device");

            let device = &raw_devices[0];
            let default_memory = device.default_memory()?;
            assert!(
                !default_memory.raw.is_null(),
                "default_memory should not be null"
            );

            let memories = device.addressable_memories()?;
            assert!(
                memories.iter().any(|m| m.raw == default_memory.raw),
                "default memory should be part of addressable memories"
            );
            Ok(())
        })
    }

    #[test]
    fn device_debug_and_process_index_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let raw_devices = client.devices().map_err(|e| e.to_string())?;
            assert!(!raw_devices.is_empty(), "expected at least one device");

            let device = &raw_devices[0];
            let debug_string = device.debug_string()?;
            let to_string = device.to_string()?;
            let process_index = device.process_index()?;

            assert!(!debug_string.is_empty(), "debug_string should not be empty");
            assert!(!to_string.is_empty(), "to_string should not be empty");
            assert!(process_index >= 0, "process_index should be non-negative");
            Ok(())
        })
    }
}
