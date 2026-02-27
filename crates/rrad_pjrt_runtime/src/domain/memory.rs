use crate::domain::client::RradClientInternal;
use crate::domain::device::RradDeviceInternal;



pub struct RradMemorySpaceInternal {
    pub id: i64,
    pub kind: String,
    pub devices: Vec<RradDeviceInternal>,
    pub client: *mut RradClientInternal,
}

