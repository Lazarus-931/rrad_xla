use std::path::{Path, PathBuf};

use rrad_pjrt::pjrt::loader::PjrtRuntime;
use rrad_pjrt::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;

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

#[test]
fn cpu_end_to_end_compile_execute_download() -> Result<(), String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping cpu_end_to_end_compile_execute_download: PJRT plugin not found");
        return Ok(());
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    let client = rt.create_client_raii()?;

    let raw_devices = client.devices()?;
    if raw_devices.is_empty() {
        return Err("client has no devices".to_string());
    }
    let device = raw_devices[0];

    let executable = client.compile(MODULE_ADD_ONE, "mlir", &[])?;

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device),
    )?;

    let (outputs, done) = executable.execute(&[&input_buffer])?;
    done.ok()?;
    if outputs.len() != 1 {
        return Err(format!("expected exactly 1 output, got {}", outputs.len()));
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}"));
    }

    Ok(())
}
