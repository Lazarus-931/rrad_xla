use core::ffi::c_void;

use crate::internal::device::PjrtDevice;
use crate::runtime_util::PJRTRuntimeError;
use crate::internal::device::RradDeviceInternal;
use crate::internal::memory::RradMemorySpaceInternal;

pub struct RradClientInternal {
    id: i32,
    platform_name: String,
    process_index: usize,
    devices: Vec<RradDeviceInternal>,
    memories: Vec<RradMemorySpaceInternal>,

}

pub struct RradBuffer<'client> {
    _phantom: std::marker::PhantomData<&'client ()>,
}

pub struct RradExecutable<'client> {
    _phantom: std::marker::PhantomData<&'client ()>,
}



pub trait PjrtClient {
    fn process_index(&self) -> usize;
    fn device_count(&self) -> usize;
    fn addressable_device_count(&self) -> usize;
    fn platform_name(&self) -> &str;
    fn platform_version(&self) -> &str;
    fn lookup_device(&self, id: i64) -> Result<&(dyn PjrtDevice + Send + Sync), PJRTRuntimeError>;
}

pub struct PjRtCApiClient {
    raw_client: *mut c_void,
    process_index: usize,
    platform_name: String,
    platform_version: String,
    devices: Vec<Box<dyn PjrtDevice + Send + Sync>>,
}

impl PjRtCApiClient {
    pub fn new(
        raw_client: *mut c_void,
        process_index: usize,
        platform_name: impl Into<String>,
        platform_version: impl Into<String>,
        devices: Vec<Box<dyn PjrtDevice + Send + Sync>>,
    ) -> Self {
        Self {
            raw_client,
            process_index,
            platform_name: platform_name.into(),
            platform_version: platform_version.into(),
            devices,
        }
    }

    pub fn raw_client(&self) -> *mut c_void {
        self.raw_client
    }
}

impl PjrtClient for PjRtCApiClient {
    fn process_index(&self) -> usize {
        self.process_index
    }

    fn device_count(&self) -> usize {
        self.devices.len()
    }

    fn addressable_device_count(&self) -> usize {
        self.devices.iter().filter(|d| d.is_addressable()).count()
    }

    fn platform_name(&self) -> &str {
        &self.platform_name
    }

    fn platform_version(&self) -> &str {
        &self.platform_version
    }

    fn lookup_device(&self, id: i64) -> Result<&(dyn PjrtDevice + Send + Sync), PJRTRuntimeError> {
        self.devices
            .iter()
            .find(|d| d.id() == id)
            .map(|d| d.as_ref())
            .ok_or_else(|| PJRTRuntimeError::not_found_for(format!("ClientDevice({id})")))
    }
}
