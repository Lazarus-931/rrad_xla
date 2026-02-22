#[cfg(test)]
mod client_wrapper_tests {
    use std::ptr::null_mut;
    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;
    use rrad_pjrt::pjrt_sys::{
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Error_Code_PJRT_Error_Code_OK,
    };
    use rrad_pjrt::rrad_pjrt::client::PJRTClient;
    use rrad_pjrt::rrad_pjrt::device::PJRTDevice;
    use rrad_pjrt::rrad_pjrt::error::PJRTError;

    const ALIAS_DIMS: [i64; 1] = [4];

    fn first_addressable_device<'a>(
        client: &'a PJRTClient<'a>,
    ) -> Result<PJRTDevice<'a>, PJRTError<'a>> {
        let devices = client.devices()?;
        let first = devices
            .first()
            .ok_or_else(|| client.error("expected at least one device"))?;
        let local_hardware_id = first.local_hardware_id()?;
        client.lookup_addressable_device(local_hardware_id)
    }

    // platform_name + platform_version + process_index
    #[test]
    fn client_basic_metadata_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let platform_name = client.platform_name()?;
            let platform_version = client.platform_version()?;
            let process_index = client.process_index()?;

            assert!(
                !platform_name.is_empty(),
                "platform_name should not be empty"
            );
            assert!(
                !platform_version.is_empty(),
                "platform_version should not be empty"
            );
            assert!(process_index >= 0, "process_index should be non-negative");

            Ok(())
        })
    }

    // topology_description + topology_platform_name + default_device_assignment
    #[test]
    fn client_topology_and_assignment_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let topology = client.topology_description()?;
            let topology_platform_name = topology.platform_name()?;
            let descs = topology.device_descriptions()?;

            assert!(
                !topology_platform_name.is_empty(),
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
        })
    }

    // platform_name + topology_platform_name
    #[test]
    fn client_platform_name_matches_topology_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let platform_name = client.platform_name()?;
            let topology_platform_name = client.topology_platform_name()?;

            assert!(
                !platform_name.is_empty(),
                "platform_name should not be empty"
            );
            assert!(
                !topology_platform_name.is_empty(),
                "topology platform_name should not be empty"
            );
            assert_eq!(
                platform_name, topology_platform_name,
                "client platform_name and topology platform_name should match"
            );

            Ok(())
        })
    }

    // devices + lookup_device + lookup_addressable_device
    #[test]
    fn client_lookup_first_device_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let devices = client.devices()?;
            let first = devices
                .first()
                .ok_or_else(|| client.error("expected at least one device"))?;

            let first_id = first.id()?;
            let local_hardware_id = first.local_hardware_id()?;

            let by_id = client.lookup_device(first_id)?;
            assert!(!by_id.is_null(), "lookup_device returned null");
            assert_eq!(
                by_id.id()?,
                first_id,
                "lookup_device should return the requested id"
            );

            let by_local = client.lookup_addressable_device(local_hardware_id)?;
            assert!(
                !by_local.is_null(),
                "lookup_addressable_device returned null"
            );

            Ok(())
        })
    }

    // addressable_memories
    #[test]
    fn client_addressable_memory_refs_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let memories = client.addressable_memories()?;
            assert!(
                !memories.is_empty(),
                "expected at least one addressable memory"
            );

            for memory in &memories {
                let kind = memory.kind()?;
                let id = memory.id()?;
                assert!(!kind.is_empty(), "memory kind should not be empty");
                assert!(id > 0, "memory id should be positive");
            }

            Ok(())
        })
    }

    // create_alias_buffer + fulfill_alias_buffer
    #[test]
    fn client_fulfill_alias_buffer_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let device = first_addressable_device(&client)?;

            let (_alias_buf, cb) = client.create_alias_buffer(
                &ALIAS_DIMS,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                None,
                None,
            )?;

            let host = [1.0_f32, 2.0, 3.0, 4.0];
            let real = client.buffer_from_host_slice_copy(
                &host,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                &ALIAS_DIMS,
                Some(device.raw()),
            )?;

            client.fulfill_alias_buffer(
                cb,
                Some(real.raw()),
                PJRT_Error_Code_PJRT_Error_Code_OK,
                None,
            )?;

            Ok(())
        })
    }

    // dma_map + dma_unmap (negative + positive cases)
    #[test]
    fn client_dma_map_unmap_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let err = client.dma_map(null_mut(), 16 );
            assert!(err.is_err(), "dma_map should raise error for null ptr");

            let err = client.dma_unmap(null_mut());
            assert!(err.is_err(), "dma_unmap should raise error for null ptr");

            let mut bytes = vec![0u8; 4096];
            let ptr = bytes.as_mut_ptr().cast::<std::ffi::c_void>();

            client.dma_map(ptr, bytes.len())?;
            client.dma_unmap(ptr)?;

            Ok(())
        })
    }

    // create_uninitialized_buffer + buffer_from_host_slice_copy + buffer_from_host_slice_move
    #[test]
    fn client_buffer_creation_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let element_type = PJRT_Buffer_Type_PJRT_Buffer_Type_F32;

            let pjrt_buffer = client.create_uninitialized_buffer(element_type);

            assert!(!pjrt_buffer.is_err(), "should not return an error");

            assert!(!pjrt_buffer.unwrap().raw.is_null(), "buffer's raw should not be null");

            Ok(())
        })
    }

    // buffer_from_host_buffer
    #[test]
    fn buffer_from_host_buffer_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let element_type = PJRT_Buffer_Type_PJRT_Buffer_Type_F32;

            let data: [i8;3] = [1_i8, 2, 3];

            let dims = ALIAS_DIMS;

            let byte_strides: Option<[i64; 1]> = Some(dims);

            for semantics in []
            let (buffer, _event) = client.buffer_from_host_buffer(
                data.as_ptr().cast::<std::ffi::c_void>(),
                element_type,
                &dims,
                byte_strides,


            )
            Ok(())

        })
    }



}
