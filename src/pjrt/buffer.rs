use std::ptr;
use std::slice::from_raw_parts;
use crate::pjrt::loader::PjrtRuntime;
use crate::pjrt_sys::*;


pub struct PJRTBuffer<'a> {
    rt: &'a PjrtRuntime,
    raw_buffer: *mut PJRT_Buffer,
}

impl<'a> PJRTBuffer<'a> {
    pub fn new(rt: &'a PjrtRuntime, raw_buffer: *mut PJRT_Buffer) -> Self {
        Self { rt, raw_buffer }
    }

    pub fn upload(&self) -> Result<*mut PJRT_Buffer,String> {
        if self.raw_buffer.is_null() {
            return Err("PJRT_Buffer is null".to_string());
        };

        let buffer_host = self.rt.api()
            .PJRT_Client_BufferFromHostBuffer
            .ok_or("PJRT_Client_BufferFromHostBuffer symbol not found")?;

        let mut buffer_args = PJRT_Client_BufferFromHostBuffer_Args {
            struct_size: PJRT_Client_BufferFromHostBuffer_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            client: self.rt.create_client()?,
            data: ptr::null_mut(),
            type_: 0,
            dims: ptr::null(),
            num_dims: 0,
            byte_strides: ptr::null(),
            num_byte_strides: 0,
            host_buffer_semantics: 0,
            device: ptr::null_mut(),
            memory: ptr::null_mut(),
            device_layout: ptr::null_mut(),
            done_with_host_buffer: ptr::null_mut(),
            buffer: ptr::null_mut(),
        };

        let err = unsafe {
            buffer_host(&mut buffer_args)
        };

        if !err.is_null() {
            Err(crate::pjrt::loader::error_to_string(self.rt.api(), err))
        } else {
            Ok(buffer_args.buffer)
        }

    }
}