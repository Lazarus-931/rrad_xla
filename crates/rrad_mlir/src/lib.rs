//! rrad_mlir: minimal MLIR/StableHLO-facing API on top of `rrad_pjrt`.

pub mod error;
pub mod module;
pub mod pipeline;

pub use error::RradMlirError;
pub use module::{ModuleText, ProgramFormat};
pub use pipeline::PjrtMlirPipeline;
