use std::path::{Path, PathBuf};

use rrad_xla::pjrt::execute_context::PJRTExecuteContext;
use rrad_xla::pjrt::loader::PjrtRuntime;
use rrad_xla::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;

const MODULE_ADD_ONE: &str = r#"module {
func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2 : tensor<f32>
}}"#;
const MODULE_TWO_OUTPUTS: &str = r#"module {
func.func @main(%arg0: tensor<f32>) -> (tensor<f32>, tensor<f32>) {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2, %0 : tensor<f32>, tensor<f32>
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

fn runtime_or_skip() -> Result<Option<PjrtRuntime>, String> {
    let Some(plugin_path) = resolve_plugin_path() else {
        eprintln!("Skipping wrapper::execute_context tests: PJRT plugin not found");
        return Ok(None);
    };

    let rt = PjrtRuntime::load(&plugin_path)?;
    rt.initialize_plugin()?;
    Ok(Some(rt))
}

#[test]
fn compile_and_execute_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt)?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device),
    )?;

    let (outputs, done) =
        executable.execute_with_context(&[&input_buffer], Some(&execute_context))?;
    done.ok()?;

    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        ));
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}"));
    }

    Ok(())
}

#[test]
fn execute_with_context_fails_on_missing_args() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt)?;
    let result = executable.execute_with_context(&[], Some(&execute_context));
    if result.is_ok() {
        return Err("expected execute_with_context to fail with missing arguments".to_string());
    }

    Ok(())
}

#[test]
fn non_empty_compile_options_topology_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let baseline = client.compile(MODULE_ADD_ONE, "mlir", &[])?;
    let compile_options = baseline.get_compile_options()?;
    if compile_options.is_empty() {
        eprintln!("Skipping non_empty_compile_options_topology_smoke: runtime returned empty compile options");
        return Ok(());
    }

    let executable =
        client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &compile_options, None)?;
    let execute_context = PJRTExecuteContext::create(&rt)?;

    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device),
    )?;

    let (outputs, done) =
        executable.execute_with_context(&[&input_buffer], Some(&execute_context))?;
    done.ok()?;
    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        ));
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}"));
    }

    Ok(())
}

#[test]
fn multi_output_execute_with_context_smoke() -> Result<(), String> {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client_raii()?;
    let _topology = client.topology_description()?;
    let executable = client.compile_on_topology_code(MODULE_TWO_OUTPUTS, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt)?;
    let raw_devices = client.devices()?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device),
    )?;

    let (outputs, done) =
        executable.execute_with_context(&[&input_buffer], Some(&execute_context))?;
    done.ok()?;

    if outputs.len() != 2 {
        return Err(format!(
            "expected exactly two output buffers, got {}",
            outputs.len()
        ));
    }

    let mut first_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut first_bytes)?;
    let first = f32::from_le_bytes(first_bytes);
    if (first - 42.0).abs() > 1e-6 {
        return Err(format!("expected first output 42.0, got {first}"));
    }

    let mut second_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[1].to_host_buffer_blocking(&mut second_bytes)?;
    let second = f32::from_le_bytes(second_bytes);
    if (second - 41.0).abs() > 1e-6 {
        return Err(format!("expected second output 41.0, got {second}"));
    }

    Ok(())
}
