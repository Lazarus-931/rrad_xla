#[cfg(test)]
mod event_bindings_tests {
    use std::ptr::null_mut;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::{runtime_or_skip, TestResult};
    use rrad_pjrt::pjrt_sys::{
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Event_Destroy_Args,
        PJRT_Event_Destroy_Args_STRUCT_SIZE,
    };
    use rrad_pjrt::event::PJRTEvent;
    use rrad_pjrt::loader::PjrtRuntime;

    // Local helper for tests that only need runtime (no client setup required).
    fn with_runtime(test_body: impl for<'a> FnOnce(&'a PjrtRuntime) -> TestResult) -> TestResult {
        let Some(rt) = runtime_or_skip()? else {
            return Ok(());
        };
        test_body(&rt)
    }

    #[test]
    fn event_create_and_is_ready_smoke() -> TestResult {
        with_runtime(|rt| {
            let event = PJRTEvent::create(rt)?;
            assert!(!event.raw().is_null(), "created event should not be null");

            // We only validate this call succeeds; readiness value may vary by backend.
            let _ready = event.is_ready()?;
            Ok(())
        })
    }

    #[test]
    fn event_on_ready_requires_callback() -> TestResult {
        with_runtime(|rt| {
            let event = PJRTEvent::create(rt)?;
            let err = event
                .on_ready(None, null_mut())
                .expect_err("on_ready(None, ..) should return an error");
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("callback"),
                "expected callback validation error, got: {err_msg}"
            );
            Ok(())
        })
    }

    #[test]
    fn event_from_buffer_ready_event_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let _device = client.lookup_addressable_device(0)?;
            let host = [1.0f32, 2.0f32, 3.0f32, 4.0f32];
            let buffer = client.buffer_from_host_slice_copy(
                &host,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                &[host.len() as i64],
                None,
            )?;

            let event = buffer.ready_event()?;
            event.await_ready()?;
            event.ok()?;
            assert!(event.is_ready()?, "ready_event should be ready after await");

            Ok(())
        })
    }

    unsafe extern "C" fn mark_event_ready(
        _error: *mut rrad_pjrt::pjrt_sys::PJRT_Error,
        user_arg: *mut libc::c_void,
    ) {
        let flag = user_arg as *const AtomicBool;
        if !flag.is_null() {
            unsafe { (&*flag).store(true, Ordering::SeqCst) };
        }
    }

    #[test]
    fn event_on_ready_callback_invoked_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let _device = client.lookup_addressable_device(0)?;
            let host = [10.0f32, 20.0f32];
            let buffer = client.buffer_from_host_slice_copy(
                &host,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                &[host.len() as i64],
                None,
            )?;
            let event = buffer.ready_event()?;

            let called = AtomicBool::new(false);
            event.on_ready(
                Some(mark_event_ready),
                (&called as *const AtomicBool).cast_mut().cast(),
            )?;
            event.await_ready()?;
            event.ok()?;

            assert!(
                called.load(Ordering::SeqCst),
                "ready callback should be invoked"
            );
            Ok(())
        })
    }

    #[test]
    fn event_into_raw_manual_destroy_smoke() -> TestResult {
        with_runtime(|rt| {
            let event = PJRTEvent::create(rt)?;
            let raw_event = event.into_raw();
            assert!(!raw_event.is_null(), "raw event should not be null");

            let destroy = rt
                .api()
                .PJRT_Event_Destroy
                .ok_or("PJRT_Event_Destroy symbol not found")?;
            let mut args = PJRT_Event_Destroy_Args {
                struct_size: PJRT_Event_Destroy_Args_STRUCT_SIZE as usize,
                extension_start: null_mut(),
                event: raw_event,
            };
            let err = unsafe { destroy(&mut args) };
            if !err.is_null() {
                return Err("PJRT_Event_Destroy failed for raw event".into());
            }
            Ok(())
        })
    }
}
