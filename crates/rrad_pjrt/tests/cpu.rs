use std::path::{Path, PathBuf};

use rrad_pjrt::ffi::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use rrad_pjrt::loader::PjrtRuntime;

const MODULE_ADD_ONE: &str = r#"module {
func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2 : tensor<f32>
}}"#;

fn resolve_plugin_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PJRT_PLUGIN") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_root.clone());

    let rel_candidates = [
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.so",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.dylib",
        "xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin",
    ];

    for base in [&crate_root, &workspace_root] {
        for rel in rel_candidates {
            let p = base.join(rel);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    None
}

#[test]
fn cpu_end_to_end_compile_execute_download() -> Result<(), String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping cpu_end_to_end_compile_execute_download: PJRT plugin not found");
        return Ok(());
    };

    let rt = PjrtRuntime::load(&plugin_path).map_err(|e| e.to_string())?;
    rt.initialize_plugin().map_err(|e| e.to_string())?;
    let client = rt.create_client().map_err(|e| e.to_string())?;

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    if raw_devices.is_empty() {
        return Err("client has no devices".to_string());
    }

    let executable = client
        .compile(MODULE_ADD_ONE, "mlir", &[])
        .map_err(|e| e.to_string())?;

    let input = [41.0f32];
    let input_buffer = client
        .buffer_from_host_slice_copy(&input, PJRT_Buffer_Type_PJRT_Buffer_Type_F32, &[], None)
        .map_err(|e| e.to_string())?;

    let (outputs, done) = executable.execute(&[&input_buffer]).map_err(|e| e.to_string())?;
    done.ok().map_err(|e| e.to_string())?;
    if outputs.len() != 1 {
        return Err(format!("expected exactly 1 output, got {}", outputs.len()));
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0]
        .to_host_buffer_blocking(&mut out_bytes)
        .map_err(|e| e.to_string())?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}"));
    }

    Ok(())
}
