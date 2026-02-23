#[cfg(test)]
mod memory_wrapper_tests {
    // Remaining test gaps for `src/rrad_pjrt/memory.rs`:
    // - Negative-path tests for null raw handle on: id, kind, kind_id, to_string,
    //   debug_string, addressable_by_device.
    // - Plugin/symbol-missing behavior is not currently unit-tested.

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;

    // id
    #[test]
    fn memory_id_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            if memories.is_empty() {
                return Err("expected at least one addressable memory".into());
            }

            for memory in &memories {
                let id = memory.id()?;
                assert!(id > 0, "memory id should be positive, got {id}");
            }

            Ok(())
        })
    }

    // kind + kind_id
    #[test]
    fn memory_kind_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            if memories.is_empty() {
                return Err("expected at least one addressable memory".into());
            }

            for memory in &memories {
                let kind = memory.kind()?;
                let kind_id = memory.kind_id()?;
                assert!(!kind.is_empty(), "memory kind should be non-empty");
                assert!(kind_id >= 0, "memory kind_id should be non-negative");
            }

            Ok(())
        })
    }

    // to_string
    #[test]
    fn memory_to_string_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            if memories.is_empty() {
                return Err("expected at least one addressable memory".into());
            }

            for memory in &memories {
                let value = memory.to_string()?;
                assert!(!value.is_empty(), "memory to_string should be non-empty");
            }

            Ok(())
        })
    }

    // debug_string
    #[test]
    fn memory_debug_string_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            if memories.is_empty() {
                return Err("expected at least one addressable memory".into());
            }

            for memory in &memories {
                let value = memory.debug_string()?;
                assert!(!value.is_empty(), "memory debug_string should be non-empty");
            }

            Ok(())
        })
    }

    // addressable_by_device
    #[test]
    fn memory_addressable_by_device_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            if memories.is_empty() {
                return Err("expected at least one addressable memory".into());
            }

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
        })
    }
}
