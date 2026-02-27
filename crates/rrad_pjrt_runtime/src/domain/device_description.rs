use std::collections::HashMap;
use crate::sys::pjrt::PJRT_NamedValue;
use crate::domain::utils::PjAttributeValue;

#[derive(Debug, Clone)]
pub struct RradDeviceDescriptionInternal {
    id: i64,
    kind: String,
    debug_string: String,
    attribute: HashMap<String, PjAttributeValue>
}

pub trait PjrtDeviceDescriptionTrait {
    fn id(&self) -> i64;
    fn kind(&self) -> &str;
    fn debug_string(&self) -> &str;
    fn string(&self) -> String {
        format!("{}:{}", self.kind(), self.id())
    }
}

impl RradDeviceDescriptionInternal {
    pub fn new(id: i64, kind: impl Into<String>, debug_string: impl Into<String>) -> Self {
        Self {
            id,
            kind: kind.into(),
            debug_string: debug_string.into(),
            attribute: Default::default(),
        }
    }
}

impl PjrtDeviceDescriptionTrait for RradDeviceDescriptionInternal {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> &str {
        &self.kind
    }

    fn debug_string(&self) -> &str {
        &self.debug_string
    }
}
