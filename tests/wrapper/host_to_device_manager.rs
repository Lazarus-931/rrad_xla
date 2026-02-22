use std::ptr::{null, null_mut};

use super::tools::{runtime_or_skip, TestResult};
use rrad_pjrt::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use rrad_pjrt::rrad_pjrt::client::PJRTClient;
use rrad_pjrt::rrad_pjrt::error::PJRTError;
use rrad_pjrt::rrad_pjrt::host_to_device_manager::PjrtHtoDeviceManager;
use rrad_pjrt::rrad_pjrt::utils::PjrtShapeSpec;

const DIMS_1D: [i64; 1] = [1];

fn make_manager<'a>(client: &'a PJRTClient<'a>) -> Result<PjrtHtoDeviceManager<'a>, PJRTError<'a>> {
    let shape_specs = [PjrtShapeSpec::new(
        DIMS_1D.to_vec(),
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
    )];
    let mut device_layouts = [null_mut()];

    client.create_buffers_for_async_host_to_device_specs(&shape_specs, &mut device_layouts, None)
}

#[test]
fn manager_create_and_query_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
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
}

#[test]
fn transfer_data_negative_offset_rejected() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
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
}

#[test]
fn transfer_literal_null_data_rejected() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
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
}

#[test]
fn partial_buffer_transfer_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;

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
}
