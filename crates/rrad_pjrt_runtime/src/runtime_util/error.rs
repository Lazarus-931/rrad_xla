use crate::c::pjrt::{
    PJRT_Error_Code,
    PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS,
    PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION,
    PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
    PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
    PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE,
    PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeErrorCode {
    InvalidArgument,
    NotFound,
    FailedPrecondition,
    AlreadyExists,
    Unimplemented,
    Internal,
    Unavailable,
}

impl RuntimeErrorCode {
    pub const fn as_pjrt_code(self) -> PJRT_Error_Code {
        match self {
            Self::InvalidArgument => PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            Self::NotFound => PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
            Self::FailedPrecondition => PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION,
            Self::AlreadyExists => PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS,
            Self::Unimplemented => PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED,
            Self::Internal => PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
            Self::Unavailable => PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE,
        }
    }
}

macro_rules! manage_runtime_error {
    ($self:expr, $( $variant:ident => $value:expr ),+ $(,)?) => {
        match $self {
            $(PJRTRuntimeError::$variant { .. } => $value,)+
        }
    };
}

macro_rules! create_runtime_error {
    ($( $variant:ident => ($ctor:ident, $code:ident, $literal:literal) ),+ $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum PJRTRuntimeError {
            $(
                $variant { sentence: String },
            )+
        }

        impl PJRTRuntimeError {
            $(
                pub fn $ctor<S: Into<String>>(sentence: S) -> Self {
                    Self::$variant {
                        sentence: sentence.into(),
                    }
                }
            )+

            pub fn from_code<S: Into<String>>(code: RuntimeErrorCode, sentence: S) -> Self {
                let sentence = sentence.into();
                match code {
                    RuntimeErrorCode::InvalidArgument => Self::invalid_argument(sentence),
                    RuntimeErrorCode::NotFound => Self::not_found(sentence),
                    RuntimeErrorCode::FailedPrecondition => Self::failed_precondition(sentence),
                    RuntimeErrorCode::AlreadyExists => Self::already_exists(sentence),
                    RuntimeErrorCode::Unimplemented => Self::unimplemented(sentence),
                    RuntimeErrorCode::Internal => Self::internal(sentence),
                    RuntimeErrorCode::Unavailable => Self::unavailable(sentence),
                }
            }

            pub fn not_found_for<S: AsRef<str>>(resource: S) -> Self {
                Self::not_found(format!("{} is not found", resource.as_ref()))
            }

            pub fn code_enum(&self) -> RuntimeErrorCode {
                manage_runtime_error!(self, $( $variant => RuntimeErrorCode::$code ),+)
            }

            pub fn code(&self) -> PJRT_Error_Code {
                self.code_enum().as_pjrt_code()
            }

            pub fn error_literal(&self) -> &'static str {
                manage_runtime_error!(self, $( $variant => $literal ),+)
            }

            pub fn sentence(&self) -> &str {
                match self {
                    $(Self::$variant { sentence } => sentence.as_str(),)+
                }
            }

            pub fn as_parts(&self) -> (RuntimeErrorCode, PJRT_Error_Code, &'static str, &str) {
                (self.code_enum(), self.code(), self.error_literal(), self.sentence())
            }
        }

        impl std::fmt::Display for PJRTRuntimeError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} ({:?}): {}",
                    self.error_literal(),
                    self.code_enum(),
                    self.sentence()
                )
            }
        }

        impl std::error::Error for PJRTRuntimeError {}
    };
}

create_runtime_error! {
    InvalidArgument => (invalid_argument, InvalidArgument, "INVALID_ARGUMENT"),
    NotFound => (not_found, NotFound, "NOT_FOUND"),
    FailedPrecondition => (failed_precondition, FailedPrecondition, "FAILED_PRECONDITION"),
    AlreadyExists => (already_exists, AlreadyExists, "ALREADY_EXISTS"),
    Unimplemented => (unimplemented, Unimplemented, "UNIMPLEMENTED"),
    Internal => (internal, Internal, "INTERNAL"),
    Unavailable => (unavailable, Unavailable, "UNAVAILABLE"),
}
