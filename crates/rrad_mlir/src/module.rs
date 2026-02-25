use crate::error::RradMlirError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramFormat {
    Mlir,
    StableHlo,
    HloText,
    Custom(String),
}

impl ProgramFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mlir => "mlir",
            Self::StableHlo => "stablehlo",
            Self::HloText => "hlo",
            Self::Custom(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleText {
    source: String,
    format: ProgramFormat,
}

impl ModuleText {
    pub fn new(source: impl Into<String>, format: ProgramFormat) -> Result<Self, RradMlirError> {
        let source = source.into();
        if source.trim().is_empty() {
            return Err(RradMlirError::InvalidModule("source must not be empty".to_string()));
        }
        if format.as_str().trim().is_empty() {
            return Err(RradMlirError::InvalidModule("format must not be empty".to_string()));
        }

        Ok(Self { source, format })
    }

    pub fn mlir(source: impl Into<String>) -> Result<Self, RradMlirError> {
        Self::new(source, ProgramFormat::Mlir)
    }

    pub fn stablehlo(source: impl Into<String>) -> Result<Self, RradMlirError> {
        Self::new(source, ProgramFormat::StableHlo)
    }

    pub fn hlo(source: impl Into<String>) -> Result<Self, RradMlirError> {
        Self::new(source, ProgramFormat::HloText)
    }

    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    pub fn format(&self) -> &ProgramFormat {
        &self.format
    }

    pub fn format_name(&self) -> &str {
        self.format.as_str()
    }
}
