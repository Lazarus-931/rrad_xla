use crate::internal::client::{PjRtCApiClient, RradClientInternal};
use crate::internal::device_description::RradDeviceDescriptionInternal;
use crate::c::device_description::{RradDeviceDescription, RradMemorySpace};
use crate::internal::device::{PjRtCApiDevice, RradDeviceInternal};
use super::rrad_api::RradApi;

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
    c_api: *mut RradApi,
    device: &'client RradDeviceInternal
}

pub trait RradDeviceTrait {
    fn id(&self) -> i64;
    fn is_addressable(&self) -> bool;
    fn description(&self) -> &dyn RradDeviceDescriptionTrait;
}

pub struct RradCApiDeviceDescription<'client> {
    c_api: *mut RradApi,
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
    c_api: *mut RradApi,
    memory: &'client RradMemorySpace<'client>
}

pub trait RradMemorySpaceTrait {
    fn client(&self) -> *mut RradClientInternal;
    fn device(&self) -> Vec<RradDeviceInternal>;
    fn id(&self) -> i64;
    fn kind(&self) -> &str;
    fn debug_string(&self) -> &str;
    fn string(&self) -> String;
    
}

pub struct RradCApiExecutable<'client> {
    c_api: *mut RradApi,
    _phantom: std::marker::PhantomData<&'client ()>,
}