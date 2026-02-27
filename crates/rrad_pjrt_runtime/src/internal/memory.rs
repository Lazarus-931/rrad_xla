use crate::internal::client::RradClientInternal;
use crate::internal::device::RradDeviceInternal;



pub struct RradMemorySpaceInternal {
    pub id: i64,
    pub kind: String,
    pub devices: Vec<RradDeviceInternal>,
    pub client: *mut RradClientInternal,
}

