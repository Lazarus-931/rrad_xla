use crate::api_wrapper::client::*;
use crate::c::pjrt::PJRT_NamedValue;
use crate::internal::client::RradClientInternal;
use crate::internal::device::RradDeviceInternal;
use crate::internal::device_description::{PjrtDeviceDescriptionTrait, RradDeviceDescriptionInternal};
use crate::internal::memory::RradMemorySpaceInternal;

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