use crate::bridge::client::*;
use crate::sys::device_description::{RradDevice, RradMemorySpace};
use crate::domain::client::*;



#[repr(C)]
pub struct RradClient<'client> {
    client: Box<RradClientInternal>,
    owned_devices: Vec<RradDevice<'client>>,
    // list of devices for this client
    devices: Vec<*mut RradDevice<'client>>,
    addressable_devices: Vec<*mut RradDevice<'client>>,
    owned_memories: Vec<RradMemorySpace<'client>>

}
