use crate::internal::client::RradClient;
use crate::internal::device::RradDeviceInternal;

pub struct RradMemorySpaceInternal {
    pub device
}

pub struct RradMemoryInternal {
    pub memory_space: *mut RradMemorySpace,
    pub devices: Vec<RradDeviceInternal>,
    pub client: *mut RradClient,
}

