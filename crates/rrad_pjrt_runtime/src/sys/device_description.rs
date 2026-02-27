use crate::bridge::client::*;
use crate::sys::pjrt::PJRT_NamedValue;
use crate::domain::client::RradClientInternal;
use crate::domain::device::RradDeviceInternal;
use crate::domain::device_description::{PjrtDeviceDescriptionTrait, RradDeviceDescriptionInternal};
use crate::domain::memory::RradMemorySpaceInternal;

#[repr(C)]
pub struct RradDeviceDescription<'client> {
    device_description: &'client RradDeviceDescriptionInternal,
    attributes: Vec<PJRT_NamedValue>
}


#[repr(C)]
pub struct RradDevice<'client> {
    pub device: &'client RradDeviceInternal,
    pub description: RradDeviceDescription<'client>,
    client: &'client RradClientInternal
}

#[repr(C)]
pub struct RradMemorySpace<'client> {
    pub memory_space: &'client RradMemorySpaceInternal,
    client: &'client RradClientInternal
}