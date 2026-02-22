use crate::wrapper::tools::{runtime_or_skip, TestResult};

#[test]
fn chunk_smoke() -> TestResult{
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let stream = client.
}
