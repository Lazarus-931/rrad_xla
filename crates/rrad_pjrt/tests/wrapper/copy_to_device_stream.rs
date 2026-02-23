#[cfg(test)]
mod copy_to_device_stream_wrapper_tests {
        use std::ptr::null_mut;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;
    use rrad_pjrt::pjrt_sys::{PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Chunk};
    use rrad_pjrt::client::PJRTClient;
    use rrad_pjrt::copy_to_device_stream::PJRTCopyToDeviceStreamRef;
    use rrad_pjrt::executable::{
        PJRTCallbackError, PJRTRecvCallbackInvocation, PJRTRecvCallbackRegistration,
        PJRTSendCallbackRegistration,
    };
    use rrad_pjrt::loader::PjrtRuntime;

    const RECV_CHANNEL_ID: i64 = 1;

    const MODULE_RECV_ONE: &str = r#"module {
  func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
    %tok0 = mhlo.create_token : !mhlo.token
    %recv, %tok1 = "mhlo.recv"(%tok0) {
      channel_handle = #mhlo.channel_handle<handle = 1, type = 3>,
      is_host_transfer = true
    } : (!mhlo.token) -> (tensor<f32>, !mhlo.token)

    %out = mhlo.add %arg0, %recv : tensor<f32>
    return %out : tensor<f32>
  }
}"#;

    static RUNTIME_PTR: AtomicUsize = AtomicUsize::new(0);
    static RECV_CALLBACK_HIT: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn chunk_deleter(data: *mut libc::c_void, _arg: *mut libc::c_void) {
        if data.is_null() {
            return;
        }
        let _ = unsafe { Box::from_raw(data as *mut [u8; 4]) };
    }

    fn recv_chunk_callback(inv: PJRTRecvCallbackInvocation) -> Result<(), PJRTCallbackError> {
        RECV_CALLBACK_HIT.store(true, Ordering::SeqCst);

        let rt_ptr = RUNTIME_PTR.load(Ordering::SeqCst) as *const PjrtRuntime;
        if rt_ptr.is_null() {
            return Err(PJRTCallbackError::new("runtime pointer not initialized"));
        }
        let rt = unsafe { &*rt_ptr };

        let stream = PJRTCopyToDeviceStreamRef::new(rt, inv.stream);

        let total_bytes = stream
            .total_bytes()
            .map_err(|e| PJRTCallbackError::new(e.to_string()))?;
        let current_bytes = stream
            .current_bytes()
            .map_err(|e| PJRTCallbackError::new(e.to_string()))?;
        let granule_size = stream
            .granule_size()
            .map_err(|e| PJRTCallbackError::new(e.to_string()))?;

        if total_bytes <= 0 {
            return Err(PJRTCallbackError::new(format!(
                "expected total_bytes > 0, got {total_bytes}"
            )));
        }
        if current_bytes < 0 || current_bytes > total_bytes {
            return Err(PJRTCallbackError::new(format!(
                "invalid current_bytes {current_bytes} for total_bytes {total_bytes}"
            )));
        }
        if granule_size <= 0 {
            return Err(PJRTCallbackError::new(format!(
                "expected granule_size > 0, got {granule_size}"
            )));
        }

        let data = Box::new(1.0f32.to_le_bytes());
        let data_ptr = Box::into_raw(data) as *mut libc::c_void;

        let mut chunk = PJRT_Chunk {
            data: data_ptr,
            size: 4,
            deleter: Some(chunk_deleter),
            deleter_arg: null_mut(),
        };

        stream
            .add_chunk(&mut chunk, None)
            .map_err(|e| PJRTCallbackError::new(format!("add_chunk failed: {e}")))
    }

    fn run_recv_execute(rt: &PjrtRuntime, client: &PJRTClient<'_>) -> TestResult {
        let executable = client.compile_on_topology_code(MODULE_RECV_ONE, "mlir", &[], None)?;

        let input_buffer = client.buffer_from_host_slice_copy(
            &[41.0f32],
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
            &[],
            None,
        )?;

        RUNTIME_PTR.store((rt as *const PjrtRuntime) as usize, Ordering::SeqCst);
        RECV_CALLBACK_HIT.store(false, Ordering::SeqCst);

        let send_callbacks: [PJRTSendCallbackRegistration; 0] = [];
        let recv_callbacks = [PJRTRecvCallbackRegistration {
            channel_id: RECV_CHANNEL_ID,
            callback: recv_chunk_callback,
        }];

        let (outputs, done) = executable.execute_with_options(
            &[&input_buffer],
            None,
            0,
            1,
            0,
            &[],
            null_mut(),
            &send_callbacks,
            &recv_callbacks,
        )?;

        done.ok()?;

        assert!(
            RECV_CALLBACK_HIT.load(Ordering::SeqCst),
            "recv callback should be invoked"
        );

        if outputs.len() != 1 {
            RUNTIME_PTR.store(0, Ordering::SeqCst);
            return Err(
                format!("expected exactly one output buffer, got {}", outputs.len()).into(),
            );
        }

        let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
        outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
        let out = f32::from_le_bytes(out_bytes);
        if (out - 42.0).abs() > 1e-6 {
            RUNTIME_PTR.store(0, Ordering::SeqCst);
            return Err(format!("expected 42.0, got {out}").into());
        }

        RUNTIME_PTR.store(0, Ordering::SeqCst);
        Ok(())
    }

    #[test]
    fn add_chunk_smoke() -> TestResult {
        with_runtime_and_client(|rt, client| run_recv_execute(rt, &client))
    }

    #[test]
    fn drop_smoke() -> TestResult {
        with_runtime_and_client(|rt, _client| {
            let executable = _client.compile_on_topology_code(MODULE_RECV_ONE, "mlir", &[], None)?;

            let input_buffer = _client.buffer_from_host_slice_copy(
                &[41.0f32],
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                &[],
                None,
            )?;

            RUNTIME_PTR.store((rt as *const PjrtRuntime) as usize, Ordering::SeqCst);

            let recv_callbacks = [PJRTRecvCallbackRegistration {
                channel_id: RECV_CHANNEL_ID,
                callback: recv_chunk_callback
            }];

            let send_callbacks: [PJRTSendCallbackRegistration; 0] = [];

            {
                let (_outputs, done) = executable.execute_with_options(
                    &[&input_buffer],
                    None,
                    0,
                    1,
                    0,
                    &[],
                    null_mut(),
                    &send_callbacks,
                    &recv_callbacks,
                )?;
                done.ok()?;
            }

            let platform = _client.platform_name()?;
            assert!(!platform.is_empty(), "expected drop");
            Ok(())
        })
    }
}
