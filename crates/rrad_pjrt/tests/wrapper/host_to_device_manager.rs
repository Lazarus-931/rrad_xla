#[cfg(test)]
mod host_to_device_manager_wrapper_tests {

    use std::ptr::{null, null_mut};

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;
    use rrad_pjrt::pjrt_sys::{PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Error_Code_PJRT_Error_Code_OK, PJRT_NamedValue, PJRT_NamedValue_STRUCT_SIZE, PJRT_NamedValue_Type_PJRT_NamedValue_kString, PJRT_NamedValue__bindgen_ty_1};
    use rrad_pjrt::client::PJRTClient;
    use rrad_pjrt::error::PJRTError;
    use rrad_pjrt::host_to_device_manager::PjrtHtoDeviceManager;
    use rrad_pjrt::utils::PJRTShapeSpec;

    const DIMS_1D: [i64; 1] = [1];

    fn make_manager<'a>(
        client: &'a PJRTClient<'a>,
    ) -> Result<PjrtHtoDeviceManager<'a>, PJRTError<'a>> {
        let shape_specs = [PJRTShapeSpec::new(
            DIMS_1D.to_vec(),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        )];
        let mut device_layouts = [null_mut()];

        client.create_buffers_for_async_host_to_device_specs(
            &shape_specs,
            &mut device_layouts,
            None,
        )
    }

    #[test]
    fn manager_create_and_query_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            assert!(
                !manager.raw().is_null(),
                "transfer manager should not be null"
            );

            let count = manager.buffer_count()?;
            assert_eq!(count, 1, "expected one staged buffer");

            let size = manager.buffer_size(0)?;
            assert!(size > 0, "buffer_size(0) should be positive");

            let device = manager.device()?;
            assert!(device.id()? >= 0, "device id should be non-negative");
            assert!(
                device.is_addressable()?,
                "manager device should be addressable"
            );

            Ok(())
        })
    }

    #[test]
    fn transfer_data_negative_offset_rejected() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let msg = match manager.transfer_data(0, &[1_u8, 2, 3, 4], -1, true) {
                Ok(_) => return Err("negative offset should be rejected".into()),
                Err(err) => err.to_string(),
            };
            assert!(
                msg.contains("offset") || msg.contains("must be >= 0"),
                "expected offset validation error, got: {msg}"
            );

            Ok(())
        })
    }

    #[test]
    fn transfer_literal_null_data_rejected() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let msg = match manager.transfer_literal(
                0,
                null(),
                &DIMS_1D,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                None,
            ) {
                Ok(_) => return Err("null literal pointer should be rejected".into()),
                Err(err) => err.to_string(),
            };
            assert!(
                msg.contains("data is null"),
                "expected null-data validation error, got: {msg}"
            );

            Ok(())
        })
    }

    #[test]
    fn partial_buffer_transfer_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let payload = [1_u8, 2, 3, 4];

            if let Some(ev) = manager.transfer_data(0, &payload[..2], 0, false)? {
                ev.await_ready()?;
                ev.ok()?;
            }

            if let Some(ev) = manager.transfer_data(0, &payload[2..], 2, false)? {
                ev.await_ready()?;
                ev.ok()?;
            }

            let buffer = manager.retrieve_buffer(0)?;

            let mut out = [0_u8; 4];
            buffer.to_host_buffer_blocking(&mut out)?;

            assert_eq!(out, payload, "expected payload to be transferred");

            Ok(())
        })
    }

    #[test]
    fn add_metadata_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let key = b"test_key";
            let value = b"test_value";
            let metadata = [PJRT_NamedValue {
                struct_size: PJRT_NamedValue_STRUCT_SIZE as usize,
                extension_start: null_mut(),
                name: key.as_ptr() as *const libc::c_char,
                name_size: key.len(),
                type_: PJRT_NamedValue_Type_PJRT_NamedValue_kString,
                __bindgen_anon_1: PJRT_NamedValue__bindgen_ty_1 {
                    string_value: value.as_ptr() as *const libc::c_char,
                },
                value_size: value.len(),
            }];

            manager.add_metadata(&metadata)?;
            Ok(())
        })
    }

    #[test]
    fn set_buffer_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let buffer_index: i32 = 3;
            let error_code = PJRT_Error_Code_PJRT_Error_Code_OK;
            let error_message = "test error message";

            manager.set_buffer_error(buffer_index, error_code, error_message)?;

            Ok(())
        })
    }

    #[test]
    fn transfer_data_errors_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let manager = make_manager(&client)?;

            let buffer_index: i32 = 3;

            let data = [1_u8, 2, 3, 4];

            let correct_offset = 0;

            let is_last_transfer = true;

            return match manager.transfer_data(buffer_index, &data[..], correct_offset, is_last_transfer) {
                Ok(_) => Ok(()),
                Err(_) => Err("transfer_data should have succeeded".into())
            };


        })
    }
}
