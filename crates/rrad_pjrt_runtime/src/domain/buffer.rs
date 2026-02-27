use crate::domain::utils::ElementType;

pub struct RradBufferInternal {
    pub dimensions: Vec<usize>,
    pub dtype: ElementType,
    pub data: Vec<u8>
}