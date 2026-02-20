use rrad_pjrt::rrad_pjrt::error::PJRTError;
use rrad_pjrt::rrad_pjrt::loader::PjrtRuntime;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TestError(pub String);

pub type TestResult<T = ()> = Result<T, TestError>;

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TestError {}

impl From<String> for TestError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TestError {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl<'a> From<PJRTError<'a>> for TestError {
    fn from(value: PJRTError<'a>) -> Self {
        Self(value.to_string())
    }
}

pub fn resolve_plugin_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PJRT_PLUGIN") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }

    let candidates = [
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.so",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.dylib",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin",
    ];
    for candidate in candidates {
        let p = Path::new(candidate).to_path_buf();
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

pub fn runtime_or_skip() -> Result<Option<PjrtRuntime>, String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping wrapper tests: PJRT plugin not found");
        return Ok(None);
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    Ok(Some(rt))
}
