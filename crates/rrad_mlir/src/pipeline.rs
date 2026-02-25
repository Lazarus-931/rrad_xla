use crate::error::RradMlirError;
use crate::module::ModuleText;
use rrad_pjrt::client::PJRTClient;
use rrad_pjrt::executable::PJRTLoadedExecutable;

/// Thin pipeline layer to treat PJRT as backend for MLIR/HLO module text.
pub struct PjrtMlirPipeline<'rt> {
    client: &'rt PJRTClient<'rt>,
}

impl<'rt> PjrtMlirPipeline<'rt> {
    pub fn new(client: &'rt PJRTClient<'rt>) -> Self {
        Self { client }
    }

    pub fn compile(
        &self,
        module: &ModuleText,
        compile_options: &[u8],
    ) -> Result<PJRTLoadedExecutable<'rt, '_>, RradMlirError> {
        self.client
            .compile(module.source(), module.format_name(), compile_options)
            .map_err(Into::into)
    }

    pub fn compile_on_topology(
        &self,
        module: &ModuleText,
        compile_options: &[u8],
        overridden_compile_options: Option<&[u8]>,
    ) -> Result<PJRTLoadedExecutable<'rt, '_>, RradMlirError> {
        self.client
            .compile_on_topology_code(
                module.source(),
                module.format_name(),
                compile_options,
                overridden_compile_options,
            )
            .map_err(Into::into)
    }
}
