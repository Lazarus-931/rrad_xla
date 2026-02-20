use super::tools::runtime_or_skip;
use rrad_pjrt::pjrt_sys::{
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
};
use rrad_pjrt::rrad_pjrt::buffer::PJRTBuffer;
use rrad_pjrt::rrad_pjrt::client::PJRTClient;
use rrad_pjrt::rrad_pjrt::error::PJRTError;

fn make_test_buffer<'a>(client: &'a PJRTClient<'a>) -> Result<PJRTBuffer<'a>, PJRTError<'a>> {
    let device = client.lookup_addressable_device(0)?;
    let host = [1.0_f32, 2.0, 3.0, 4.0];
    client.buffer_from_host_slice_copy(
        &host,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[host.len() as i64],
        Some(device.raw),
    )
}

#[test]
fn buffer_delete_smoke() -> Result<(), PJRTError> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let buffer = make_test_buffer(&client)?;

    assert!(
        !buffer.is_deleted().map_err(|e| e.to_string())?,
        "newly-created buffer should not be deleted"
    );
    buffer.delete().map_err(|e| e.to_string())?;
    assert!(
        buffer.is_deleted().map_err(|e| e.to_string())?,
        "buffer should report deleted after delete"
    );
    Ok(())
}

#[test]
fn buffer_get_memory_layout_smoke() -> Result<(), PJRTError<'a>> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let buffer = make_test_buffer(&client)?;

    let layout = buffer.get_memory_layout()?;
    let memory = buffer.memory()?;
    assert!(!memory.raw.is_null(), "buffer memory should not be null");
    assert!(
        layout.type_ == PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled
            || layout.type_ == PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
        "memory layout type should be tiled or strides"
    );
    Ok(())
}

#[test]
fn buffer_dynamic_dims_smoke() -> Result<(), PJRTError<'a>> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let device = client.lookup_addressable_device(0)?;
    let host = [1.0_f32, 2.0, 3.0, 4.0];
    let dims = [2_i64, 2_i64];
    let buffer = client.buffer_from_host_slice_copy(
        &host,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &dims,
        Some(device.raw),
    )?;

    let dynamic_dims = buffer.dynamic_dimension_indices()?;
    assert!(
        dynamic_dims.iter().all(|idx| *idx < dims.len()),
        "dynamic dim indices should be within rank bounds"
    );
    Ok(())
}

#[test]
fn buffer_external_references_smoke() -> Result<(), PJRTError<'a>> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let buffer = make_test_buffer(&client)?;
    buffer.increase_external_ref()?;
    buffer.decrease_external_ref()?;
    Ok(())
}

#[test]
fn buffer_on_device_size_and_element_type_smoke() -> Result<(), PJRTError<'a>> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let buffer = make_test_buffer(&client)?;

    let element_type = buffer.element_type().map_err(|e| e.to_string())?;
    let dims = buffer.dimensions().map_err(|e| e.to_string())?;
    let size = buffer.on_device_size_in_bytes()?;

    assert_eq!(
        element_type, PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        "element type should be F32"
    );
    assert_eq!(dims, vec![4], "dimensions should match upload shape");
    assert!(
        size >= 4 * std::mem::size_of::<f32>(),
        "on-device size should be at least payload size"
    );
    Ok(())
}

#[test]
fn buffer_to_host_async_roundtrip_smoke() -> Result<(), PJRTError<'a>> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let buffer = make_test_buffer(&client)?;
    let mut out_bytes = [0u8; 4 * std::mem::size_of::<f32>()];

    let done = buffer.to_host_buffer_async(&mut out_bytes)?;
    done.await_ready().map_err(|e| e.to_string())?;
    done.ok()?;

    let mut out = [0.0_f32; 4];
    for (i, chunk) in out_bytes
        .chunks_exact(std::mem::size_of::<f32>())
        .enumerate()
    {
        out[i] = f32::from_le_bytes(chunk.try_into().map_err(|_| "invalid output chunk size")?);
    }

    assert_eq!(out, [1.0, 2.0, 3.0, 4.0], "roundtrip values should match");
    Ok(())
}
