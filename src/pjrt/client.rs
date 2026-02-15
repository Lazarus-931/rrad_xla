use std::ptr;
use std::slice::from_raw_parts;
use crate::pjrt::loader::PjrtRuntime;
use crate::pjrt_sys::*;

//raii wrapper for PJRT_Client

pub struct PJRTClient<'a> {
    rt: &'a PjrtRuntime,
    raw_client: *mut PJRT_Client,
}


impl PJRTClient<'_> {

    pub fn devices(&self) -> Result<Vec<*mut PJRT_Device>, String> {
        self.rt.client_devices(self.raw_client)
    }

    pub fn raw(&self) -> *mut PJRT_Client {
        self.raw_client
    }

    // destory errors
    pub fn close(self) -> Result<(), String> {
        let raw = self.raw_client;
        let rt = self.rt;
        std::mem::forget(self);
        rt.destroy_client(raw)
    }
}

impl Drop for PJRTClient<'_> {
    fn drop(&mut self) {
        self.rt.destroy_client(self.raw_client).unwrap();
    }
}