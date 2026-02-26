


#[repr(C)]
pub enum PrimitiveTypes {
    Invalid = 0,
    Pred    = 1,
    S8      = 2,
    S16     = 3,
    S32     = 4,
    S64     = 5,
    U8      = 6,
    F16     = 10,
    F32     = 11,
    F64     = 12,
}
#[derive(Debug, Clone)]
pub enum PjAttributeValue {
    String(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}