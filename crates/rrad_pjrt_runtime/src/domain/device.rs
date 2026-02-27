use crate::sys::device_description::RradDeviceDescription;
use crate::domain::device_description::{RradDeviceDescriptionInternal, PjrtDeviceDescriptionTrait};

pub trait PjrtDevice {
    fn id(&self) -> i64;
    fn is_addressable(&self) -> bool;
    fn description(&self) -> &dyn PjrtDeviceDescriptionTrait;
}

pub struct PjRtCApiDevice {
    id: i64,
    addressable: bool,
    description: Box<dyn PjrtDeviceDescriptionTrait + Send + Sync>,
}

impl PjRtCApiDevice {
    pub fn new(
        id: i64,
        addressable: bool,
        description: Box<dyn PjrtDeviceDescriptionTrait + Send + Sync>,
    ) -> Self {
        Self {
            id,
            addressable,
            description,
        }
    }
}

impl PjrtDevice for PjRtCApiDevice {
    fn id(&self) -> i64 {
        self.id
    }

    fn is_addressable(&self) -> bool {
        self.addressable
    }

    fn description(&self) -> &dyn PjrtDeviceDescriptionTrait {
        self.description.as_ref()
    }
}

pub struct RradDeviceInternal {
    id: i64,
    addressable: bool,
    description: Box<dyn PjrtDeviceDescriptionTrait + Send + Sync>,
}
