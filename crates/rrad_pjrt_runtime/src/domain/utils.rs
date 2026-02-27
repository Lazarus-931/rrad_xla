


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


// rrad/src/internal/domain/element_type.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]  // matches PJRT_Buffer_Type underlying integer type
pub enum ElementType {
    Invalid       = 0,
    // Boolean
    Pred          = 1,
    // Signed integers
    S2            = 24,
    S4            = 21,
    S8            = 2,
    S16           = 3,
    S32           = 4,
    S64           = 5,
    // Unsigned integers
    U2            = 25,
    U4            = 22,
    U8            = 6,
    U16           = 7,
    U32           = 8,
    U64           = 9,
    // Floats
    F16           = 10,
    F32           = 11,
    F64           = 12,
    BF16          = 13,
    // Complex
    C64           = 14,
    C128          = 15,
    // FP8 variants
    F8E5M2        = 16,
    F8E4M3FN      = 17,
    F8E4M3B11FNUZ = 18,
    F8E5M2FNUZ    = 19,
    F8E4M3FNUZ    = 20,
    F8E4M3        = 26,
    F8E3M4        = 27,
    F8E8M0FNU     = 28,
    // FP4
    F4E2M1FN      = 29,
    // Special
    Token         = 23,
}

impl ElementType {
    // convert from raw PJRT_Buffer_Type integer — used at the bridge layer
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0  => Some(Self::Invalid),
            1  => Some(Self::Pred),
            2  => Some(Self::S8),
            3  => Some(Self::S16),
            4  => Some(Self::S32),
            5  => Some(Self::S64),
            6  => Some(Self::U8),
            7  => Some(Self::U16),
            8  => Some(Self::U32),
            9  => Some(Self::U64),
            10 => Some(Self::F16),
            11 => Some(Self::F32),
            12 => Some(Self::F64),
            13 => Some(Self::BF16),
            14 => Some(Self::C64),
            15 => Some(Self::C128),
            16 => Some(Self::F8E5M2),
            17 => Some(Self::F8E4M3FN),
            18 => Some(Self::F8E4M3B11FNUZ),
            19 => Some(Self::F8E5M2FNUZ),
            20 => Some(Self::F8E4M3FNUZ),
            21 => Some(Self::S4),
            22 => Some(Self::U4),
            23 => Some(Self::Token),
            24 => Some(Self::S2),
            25 => Some(Self::U2),
            26 => Some(Self::F8E4M3),
            27 => Some(Self::F8E3M4),
            28 => Some(Self::F8E8M0FNU),
            29 => Some(Self::F4E2M1FN),
            _  => None,
        }
    }

    // size in bytes — useful for buffer allocation validation
    pub fn byte_size(self) -> Option<usize> {
        match self {
            Self::Pred              => Some(1),
            Self::S8  | Self::U8   => Some(1),
            Self::S16 | Self::U16  => Some(2),
            Self::S32 | Self::U32  => Some(4),
            Self::S64 | Self::U64  => Some(8),
            Self::F16 | Self::BF16 => Some(2),
            Self::F32              => Some(4),
            Self::F64              => Some(8),
            Self::C64              => Some(8),
            Self::C128             => Some(16),
            Self::F8E5M2
            | Self::F8E4M3FN
            | Self::F8E4M3B11FNUZ
            | Self::F8E5M2FNUZ
            | Self::F8E4M3FNUZ
            | Self::F8E4M3
            | Self::F8E3M4
            | Self::F8E8M0FNU      => Some(1),
            // sub-byte types — no clean byte size
            Self::S4  | Self::U4
            | Self::S2 | Self::U2
            | Self::F4E2M1FN       => None,
            Self::Token
            | Self::Invalid        => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PjAttributeValue {
    String(String),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    Float32(f32),
    Float64(f64),
    Bool(bool),
}