use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use rrad_pjrt::pjrt_sys::{
    PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_ExecuteContext_Destroy_Args,
    PJRT_ExecuteContext_Destroy_Args_STRUCT_SIZE,
};
use rrad_pjrt::rrad_pjrt::execute_context::PJRTExecuteContext;
use rrad_pjrt::rrad_pjrt::loader::PjrtRuntime;
use super::tools::TestResult;

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
fn compile_and_execute_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;
    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let (outputs, done) = executable
        .execute_with_context(&[&input_buffer], Some(&execute_context))
        .map_err(|e| e.to_string())?;
    done.ok()?;

    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        )
        .into());
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}").into());
    }

    Ok(())
}

#[test]
fn execute_with_context_fails_on_missing_args() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;
    let result = executable.execute_with_context(&[], Some(&execute_context));
    if result.is_ok() {
        return Err("expected execute_with_context to fail with missing arguments".to_string().into());
    }

    Ok(())
}

#[test]
fn non_empty_compile_options_topology_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip().map_err(|e| e.to_string())? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let topology = client.topology_description()?;
    let descs = topology.device_descriptions()?;
    assert!(
        !descs.is_empty(),
        "expected topology to include at least one device description"
    );

    let baseline = client
        .compile(MODULE_ADD_ONE, "mlir", &[])
        .map_err(|e| e.to_string())?;
    let compile_options = baseline.get_compile_options().map_err(|e| e.to_string())?;
    if compile_options.is_empty() {
        eprintln!("Skipping non_empty_compile_options_topology_smoke: runtime returned empty compile options");
        return Ok(());
    }

    let executable =
        client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &compile_options, None)?;
    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let (outputs, done) = executable
        .execute_with_context(&[&input_buffer], Some(&execute_context))
        .map_err(|e| e.to_string())?;
    done.ok()?;
    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        )
        .into());
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}").into());
    }

    Ok(())
}

#[test]
fn multi_output_execute_with_context_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let _topology = client.topology_description()?;
    let executable = client.compile_on_topology_code(MODULE_TWO_OUTPUTS, "mlir", &[], None)?;

    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;
    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let (outputs, done) = executable
        .execute_with_context(&[&input_buffer], Some(&execute_context))
        .map_err(|e| e.to_string())?;
    done.ok()?;

    if outputs.len() != 2 {
        return Err(format!(
            "expected exactly two output buffers, got {}",
            outputs.len()
        )
        .into());
    }

    let mut first_bytes = [0u8; size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut first_bytes)?;
    let first = f32::from_le_bytes(first_bytes);
    if (first - 42.0).abs() > 1e-6 {
        return Err(format!("expected first output 42.0, got {first}").into());
    }

    let mut second_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[1].to_host_buffer_blocking(&mut second_bytes)?;
    let second = f32::from_le_bytes(second_bytes);
    if (second - 41.0).abs() > 1e-6 {
        return Err(format!("expected second output 41.0, got {second}").into());
    }

    Ok(())
}

#[test]
fn execute_with_options_launch_id_and_device_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;
    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let (outputs, done) = executable
        .execute_with_options(
            &[&input_buffer],
            Some(&execute_context),
            0,
            0,
            1234,
            &[],
            device.raw(),
        )
        .map_err(|e| e.to_string())?;
    done.ok()?;

    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        )
        .into());
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 42.0).abs() > 1e-6 {
        return Err(format!("expected 42.0, got {out}").into());
    }

    Ok(())
}

#[test]
fn execute_with_options_rejects_callback_counts() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;
    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let result = executable.execute_with_options(
        &[&input_buffer],
        Some(&execute_context),
        1,
        0,
        0,
        &[],
        null_mut(),
    );
    if result.is_ok() {
        return Err("expected execute_with_options to reject nonzero callback counts".to_string().into());
    }

    Ok(())
}

#[test]
fn execute_with_options_rejects_negative_non_donatable_indices() -> TestResult {
    let Some(rt) = runtime_or_skip().map_err(|e| e.to_string())? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;
    let execute_context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;

    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [41.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let result = executable.execute_with_options(
        &[&input_buffer],
        Some(&execute_context),
        0,
        0,
        0,
        &[-1],
        device.raw(),
    );
    if result.is_ok() {
        return Err(
            "expected execute_with_options to reject negative non_donatable_input_indices"
                .to_string()
                .into(),
        );
    }

    Ok(())
}

#[test]
fn execute_without_context_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip().map_err(|e| e.to_string())? else {
        return Ok(());
    };

    let client = rt.create_client()?;
    let executable = client.compile_on_topology_code(MODULE_ADD_ONE, "mlir", &[], None)?;
    let raw_devices = client.devices().map_err(|e| e.to_string())?;
    assert!(!raw_devices.is_empty(), "client has no devices");
    let device = &raw_devices[0];

    let input = [5.0f32];
    let input_buffer = client.buffer_from_host_slice_copy(
        &input,
        PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
        &[],
        Some(device.raw()),
    )?;

    let (outputs, done) = executable
        .execute_with_context(&[&input_buffer], None)
        .map_err(|e| e.to_string())?;
    done.ok()?;
    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one output buffer, got {}",
            outputs.len()
        )
        .into());
    }

    let mut out_bytes = [0u8; std::mem::size_of::<f32>()];
    outputs[0].to_host_buffer_blocking(&mut out_bytes)?;
    let out = f32::from_le_bytes(out_bytes);
    if (out - 6.0).abs() > 1e-6 {
        return Err(format!("expected 6.0, got {out}").into());
    }
    Ok(())
}

#[test]
fn execute_context_into_raw_manual_destroy_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(());
    };

    let context = PJRTExecuteContext::create(&rt).map_err(|e| e.to_string())?;
    let raw = context.into_raw();
    assert!(!raw.is_null(), "raw execute context should not be null");

    let destroy = rt
        .api()
        .PJRT_ExecuteContext_Destroy
        .ok_or("PJRT_ExecuteContext_Destroy symbol not found")?;
    let mut args = PJRT_ExecuteContext_Destroy_Args {
        struct_size: PJRT_ExecuteContext_Destroy_Args_STRUCT_SIZE as usize,
        extension_start: null_mut(),
        context: raw,
    };
    let err = unsafe { destroy(&mut args) };
    if !err.is_null() {
        return Err("PJRT_ExecuteContext_Destroy failed for raw execute context".to_string().into());
    }
    Ok(())
}
