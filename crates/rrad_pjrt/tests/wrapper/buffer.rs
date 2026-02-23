#[cfg(test)]
mod buffer_wrapper_tests {
        use super::super::tools::{runtime_or_skip, TestResult};
    use rrad_pjrt::pjrt_sys::{
        PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
        PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
    };
    use rrad_pjrt::buffer::PJRTBuffer;
    use rrad_pjrt::client::PJRTClient;
    use rrad_pjrt::error::PJRTError;
    use rrad_pjrt::loader::PjrtRuntime;

    // Local setup helper for this file so tests stay concise.
    fn with_rt_client(
        test_body: impl for<'a> FnOnce(&'a PjrtRuntime, PJRTClient<'a>) -> TestResult,
    ) -> TestResult {
        let Some(rt) = runtime_or_skip()? else {
            return Ok(());
        };
        let client = rt.create_client()?;
        test_body(&rt, client)
    }

    fn make_test_buffer<'a>(client: &'a PJRTClient<'a>) -> Result<PJRTBuffer<'a>, PJRTError<'a>> {
        let _device = client.lookup_addressable_device(0)?;
        let host = [1.0_f32, 2.0, 3.0, 4.0];
        client.buffer_from_host_slice_copy(
            &host,
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
            &[host.len() as i64],
            None,
        )
    }

    fn decode_f32_vec(bytes: &[u8]) -> TestResult<Vec<f32>> {
        if bytes.len() % size_of::<f32>() != 0 {
            return Err("output byte length is not a multiple of f32 size".into());
        }

        let values = bytes
            .chunks_exact(size_of::<f32>())
            .map(|chunk| {
                let arr: [u8; 4] = chunk
                    .try_into()
                    .map_err(|_| "failed converting output chunk to [u8; 4]")?;
                Ok(f32::from_le_bytes(arr))
            })
            .collect::<Result<Vec<_>, &str>>()
            .map_err(|e| e.to_string())?;

        Ok(values)
    }

    // delete + is_deleted
    #[test]
    fn buffer_delete_and_is_deleted_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;

            assert!(!buffer.is_deleted()?, "new buffer should not be deleted");
            buffer.delete()?;
            assert!(buffer.is_deleted()?, "buffer should be deleted after delete() call");
            Ok(())
        })
    }

    // element_type + dimensions + unpadded_dimensions
    #[test]
    fn buffer_shape_and_type_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;

            assert_eq!(
                buffer.element_type()?,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                "element type should be F32"
            );
            assert_eq!(buffer.dimensions()?, vec![4], "dimensions should match input shape");
            assert_eq!(
                buffer.unpadded_dimensions()?,
                vec![4],
                "unpadded dimensions should match input shape"
            );
            Ok(())
        })
    }

    // dynamic_dimension_indices
    #[test]
    fn buffer_dynamic_dimension_indices_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let dims = buffer.dimensions()?;
            let dynamic = buffer.dynamic_dimension_indices()?;

            assert!(
                dynamic.iter().all(|idx| *idx < dims.len()),
                "dynamic indices should be within rank bounds"
            );
            Ok(())
        })
    }

    // on_device_size_in_bytes
    #[test]
    fn buffer_on_device_size_in_bytes_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let size = buffer.on_device_size_in_bytes()?;

            assert!(
                size >= 4 * std::mem::size_of::<f32>(),
                "on-device size should be at least payload size"
            );
            Ok(())
        })
    }

    // get_memory_layout
    #[test]
    fn buffer_get_memory_layout_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let layout = buffer.get_memory_layout()?;

            assert!(
                layout.type_ == PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides
                    || layout.type_ == PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
                "layout should be strides or tiled"
            );
            Ok(())
        })
    }

    // ready_event
    #[test]
    fn buffer_ready_event_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let event = buffer.ready_event()?;
            event.await_ready()?;
            event.ok()?;
            Ok(())
        })
    }

    // to_host_buffer_async + to_host_buffer_blocking
    #[test]
    fn buffer_to_host_paths_roundtrip_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;

            let mut async_bytes = vec![0u8; 4 * std::mem::size_of::<f32>()];
            let done = buffer.to_host_buffer_async(&mut async_bytes)?;
            done.await_ready()?;
            done.ok()?;
            let async_vals = decode_f32_vec(&async_bytes)?;
            assert_eq!(async_vals, vec![1.0, 2.0, 3.0, 4.0]);

            let mut blocking_bytes = vec![0u8; 4 * std::mem::size_of::<f32>()];
            buffer.to_host_buffer_blocking(&mut blocking_bytes)?;
            let blocking_vals = decode_f32_vec(&blocking_bytes)?;
            assert_eq!(blocking_vals, vec![1.0, 2.0, 3.0, 4.0]);

            Ok(())
        })
    }

    // is_on_cpu
    #[test]
    fn buffer_is_on_cpu_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            assert!(buffer.is_on_cpu()?, "cpu plugin buffer should be on CPU");
            Ok(())
        })
    }

    // memory
    #[test]
    fn buffer_memory_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let memory = buffer.memory()?;
            assert!(!memory.raw.is_null(), "buffer memory handle should not be null");
            Ok(())
        })
    }

    // increase_external_ref + decrease_external_ref
    #[test]
    fn buffer_external_reference_count_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            buffer.increase_external_ref()?;
            buffer.decrease_external_ref()?;
            Ok(())
        })
    }

    #[test]
    fn unsafe_pointer_smoke() -> TestResult {
        with_rt_client(|_rt, client| {
            let buffer = make_test_buffer(&client)?;
            let pointer = buffer.unsafe_pointer()?;
            assert_eq!(!pointer, 0, "pointer should not be null, but 0");
            Ok(())
        })
    }

}
