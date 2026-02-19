use crate::pjrt_sys::{PJRT_Buffer_MemoryLayout, PJRT_Buffer_Type};
use crate::rrad_pjrt::device::PJRTDevice;

pub struct Shape<'a> {
    pub dims: &'a [i64],
    pub element_type: PJRT_Buffer_Type,
}

pub enum HostBufferSemantics {
    ImmutableOnlyDuringCalls,
    ImmutableUntilTransferCompletes,
    ImmutableZeroCopy,
    MutableZeroCopy,
}

pub struct BufferFromHostOptions<'a> {
    pub device: Option<PJRTDevice<'a>>,
    pub memory: Option<PJRTDevice<'a>>,
    pub layout: Option<&'a PJRT_Buffer_MemoryLayout>,
    pub semantics: HostBufferSemantics,
}
