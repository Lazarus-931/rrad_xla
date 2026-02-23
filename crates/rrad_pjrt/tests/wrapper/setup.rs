#![cfg(test)]

use rrad_pjrt::client::PJRTClient;
use rrad_pjrt::loader::PjrtRuntime;

use super::tools::{runtime_or_skip, TestResult};

#[allow(dead_code)]
pub fn create_client<'a>(rt: &'a PjrtRuntime) -> TestResult<PJRTClient<'a>> {
    rt.create_client().map_err(Into::into)
}

#[allow(dead_code)]
pub fn with_runtime_and_client(
    test_body: impl for<'a> FnOnce(&'a PjrtRuntime, PJRTClient<'a>) -> TestResult,
) -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = create_client(&rt)?;
    test_body(&rt, client)
}
