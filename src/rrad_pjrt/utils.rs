use crate::pjrt_sys::{PJRT_Buffer_MemoryLayout, PJRT_Buffer_Type, PJRT_NamedValue, PJRT_ShapeSpec, PJRT_ShapeSpec_STRUCT_SIZE};
use crate::rrad_pjrt::device::PJRTDevice;
use crate::rrad_pjrt::memory::PJRTMemory;
use std::ptr;

pub struct Shape<'a> {
    pub dims: &'a [i64],
    pub element_type: PJRT_Buffer_Type,
}

#[derive(Debug, Clone)]
pub struct PJRTShapeSpec {
    dims: Vec<i64>,
    element_type: PJRT_Buffer_Type,
}

impl PJRTShapeSpec {
    pub fn new(dims: impl Into<Vec<i64>>, element_type: PJRT_Buffer_Type) -> Self {
        Self {
            dims: dims.into(),
            element_type,
        }
    }

    pub fn dims(&self) -> &[i64] {
        &self.dims
    }

    pub fn element_type(&self) -> PJRT_Buffer_Type {
        self.element_type
    }

    pub(crate) fn to_raw(&self) -> PJRT_ShapeSpec {
        PJRT_ShapeSpec {
            struct_size: PJRT_ShapeSpec_STRUCT_SIZE as usize,
            extension_start: std::ptr::null_mut(),
            dims: if self.dims.is_empty() {
                ptr::null()
            } else {
                self.dims.as_ptr()
            },
            num_dims: self.dims.len(),
            element_type: self.element_type,
        }
    }
}

pub enum HostBufferSemantics {
    ImmutableOnlyDuringCalls,
    ImmutableUntilTransferCompletes,
    ImmutableZeroCopy,
    MutableZeroCopy,
}

pub struct BufferFromHostOptions<'a> {
    pub device: Option<PJRTDevice<'a>>,
    pub memory: Option<PJRTMemory<'a>>,
    pub layout: Option<&'a PJRT_Buffer_MemoryLayout>,
    pub semantics: HostBufferSemantics,
}
