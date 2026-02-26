use crate::internal::client::{PjRtCApiClient, RradClient};
use crate::internal::device_description::{PjRtCApiDeviceDescription, RradDeviceDescriptionInternal};
use crate::c::device_description::{RradDeviceDescription, RradMemory, RradMemorySpace};
use crate::internal::device::{PjRtCApiDevice, RradDeviceInternal};

#[repr(C)]
pub struct PjrtClient {
    _private: [u8; 0],
}

#[repr(C)]
pub struct PjrtDevice {
    _private: [u8; 0],
}




pub struct PjrtCApiMemorySpace {
    c_memory: *mut PjrtDevice,
    client: *mut PjRtCApiClient
}

pub struct PjrtCApiDevice {
    c_device: *mut PjrtDevice,
    client: *mut PjRtCApiClient
}


pub struct RradCApiDevice<'client> {
    c_api: *mut Rrad_Api,
    device: &'client RradDeviceInternal
}

pub trait RradDeviceTrait {
    fn id(&self) -> i64;
    fn is_addressable(&self) -> bool;
    fn description(&self) -> &dyn RradDeviceDescriptionTrait;
}

pub struct RradCApiDeviceDescription<'client> {
    c_api: *mut Rrad_Api,
    description: &'client RradDeviceDescription<'client>
}

pub trait RradDeviceDescriptionTrait {
    fn id(&self) -> i64;
    fn process_index(&self)  -> i64;

    fn device_kind(&self) -> &str;
    fn debug_string(&self) -> &str;
    fn string(&self) -> String;
}

pub struct RradCApiMemory<'client> {
    c_api: *mut Rrad_Api,
    memory: &'client RradMemorySpace<'client>
}

pub trait RradMemorySpaceTrait {
    fn client(&self) -> *mut RradClient;
    fn device(&self) -> Vec<RradDeviceInternal>;
    fn id(&self) -> i64;
    fn kind(&self) -> &str;
    fn debug_string(&self) -> &str;
    fn string(&self) -> String;



}