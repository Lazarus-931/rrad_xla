use std::path::{Path, PathBuf};

use rrad_xla::pjrt::loader::PjrtRuntime;

fn resolve_plugin_path() -> Option<PathBuf> {
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

fn runtime_or_skip() -> Result<Option<PjrtRuntime>, String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping wrapper::memory tests: PJRT plugin not found");
        return Ok(None);
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    Ok(Some(rt))
}

#[test]
fn memory_id_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let memories = client.addressable_memory_refs()?;
    assert!(
        !memories.is_empty(),
        "expected at least one addressable memory"
    );

    for memory in &memories {
        let id = memory.id()?;
        let kind = memory.kind()?;
        assert!(id > 0, "memory id should be positive, got {id}");
        assert!(!kind.is_empty(), "memory kind should be non-empty");
    }

    Ok(())
}
