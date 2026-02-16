use std::mem;
use std::ptr;
use std::slice::from_raw_parts;

use crate::pjrt::loader::{error_to_string, PjrtRuntime};
use crate::pjrt_sys::*;


pub struct PJRTTopologyDescription<'a> {
    rt: &'a PjrtRuntime,
    raw: *mut PJRT_TopologyDescription,
}



impl<'a> PJRTTopologyDescription<'a> {
    pub(crate) fn new(rt: &'a PjrtRuntime, raw: *mut PJRT_TopologyDescription) -> Self {
        Self { rt, raw }
    }

    fn raw_checked(&self) -> Result<*mut PJRT_TopologyDescription, String> {
        if self.raw.is_null() {
            Err("PJRT_Buffer is null".to_string())
        } else {
            Ok(self.raw)
        }
    }

    pub fn create(self) -> Result<(), String> {
        self.raw_checked()?;

        let create = self.rt
            .api().PJRT_TopologyDescription_Create
            .ok_or("Unable to ceate PJRT_TopplogyDescription Instance".to_string())?;

        let mut args = PJRT_TopologyDescription_Create_Args {
            struct_size: PJRT_TopologyDescription_Create_Args_STRUCT_SIZE as usize,
            extension_start: ptr::null_mut(),
            topology_name: ptr::null(),
            topology_name_size: 0,
            create_options: ptr::null(),
            num_options: 0,
            topology: ptr::null_mut(),
        };

        let err = unsafe { create(&mut args) };

        if !err.is_null() {
            Err(error_to_string(self.rt.api(), err).to_string())
        } else {
            Ok(())
        }
    }

}