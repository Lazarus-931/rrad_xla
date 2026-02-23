#[cfg(test)]
mod compile_wrapper_tests {
    // Remaining wrapper methods that still need dedicated tests:
    // - PJRTCompiler::addressable_devices

    use super::super::setup::with_runtime_and_client;
    use super::super::tools::TestResult;
    use rrad_pjrt::pjrt_sys::PJRT_Program;
    use std::ptr::null_mut;

    const MLIR_MODULE_ADD_ONE: &str = r#"module {
  func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
    %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
    %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
    %2 = mhlo.add %0, %1 : tensor<f32>
    return %2 : tensor<f32>
  }
}"#;

    const MLIR_FORMAT: &str = "mlir";

    const STABLEHLO_MODULE_FUNCTION: &str = r#"module {
  func.func @main(
    %image: tensor<28x28xf32>,
    %weights: tensor<784x10xf32>,
    %bias: tensor<1x10xf32>
  ) -> tensor<1x10xf32> {
    %0 = "stablehlo.reshape"(%image) : (tensor<28x28xf32>) -> tensor<1x784xf32>
    %1 = "stablehlo.dot"(%0, %weights) : (tensor<1x784xf32>, tensor<784x10xf32>) -> tensor<1x10xf32>
    %2 = "stablehlo.add"(%1, %bias) : (tensor<1x10xf32>, tensor<1x10xf32>) -> tensor<1x10xf32>
    %3 = "stablehlo.constant"() {value = dense<0.0> : tensor<1x10xf32>} : () -> tensor<1x10xf32>
    %4 = "stablehlo.maximum"(%2, %3) : (tensor<1x10xf32>, tensor<1x10xf32>) -> tensor<1x10xf32>
    "func.return"(%4): (tensor<1x10xf32>) -> ()
  }
}"#;

    const HLO_TEXT_ADD_ONE: &str = r#"HloModule add_one

ENTRY main {
  x = f32[] parameter(0)
  one = f32[] constant(1)
  ROOT out = f32[] add(x, one)
}"#;

    const HLO_FORMAT: &str = "hlo";
    const STABLEHLO_FORMAT: &str = "stablehlo";
    const COMPILE_OPTIONS: &[u8] = &[];

    fn program_with_format() -> PJRT_Program {
        PJRT_Program {
            struct_size: 0,
            extension_start: null_mut(),
            code: MLIR_MODULE_ADD_ONE.as_ptr() as *mut libc::c_char,
            code_size: MLIR_MODULE_ADD_ONE.len(),
            format: MLIR_FORMAT.as_ptr() as *const libc::c_char,
            format_size: MLIR_FORMAT.len(),
        }
    }

    fn program_without_format() -> PJRT_Program {
        PJRT_Program {
            struct_size: 0,
            extension_start: null_mut(),
            code: MLIR_MODULE_ADD_ONE.as_ptr() as *mut libc::c_char,
            code_size: MLIR_MODULE_ADD_ONE.len(),
            format: std::ptr::null(),
            format_size: 0,
        }
    }

    #[test]
    fn compile_rejects_invalid_arguments_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();

            match compiler.compile("", MLIR_FORMAT, COMPILE_OPTIONS) {
                Ok(_) => return Err("expected compile to fail for empty program_code".into()),
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        msg.contains("program_code must not be empty"),
                        "unexpected error for empty program_code: {msg}"
                    );
                }
            }

            match compiler.compile(MLIR_MODULE_ADD_ONE, "", COMPILE_OPTIONS) {
                Ok(_) => return Err("expected compile to fail for empty format".into()),
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        msg.contains("format must not be empty"),
                        "unexpected error for empty format: {msg}"
                    );
                }
            }

            Ok(())
        })
    }

    #[test]
    fn compile_program_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();
            let program = program_with_format();

            let _executable = compiler.compile_program(&program, COMPILE_OPTIONS)?;
            Ok(())
        })
    }

    #[test]
    fn compile_program_with_format_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();
            let mut program = program_without_format();

            let _executable =
                compiler.compile_program_with_format(&mut program, MLIR_FORMAT, COMPILE_OPTIONS)?;
            Ok(())
        })
    }

    #[test]
    fn compile_program_with_format_rejects_empty_format_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();
            let mut program = program_without_format();

            match compiler.compile_program_with_format(&mut program, "", COMPILE_OPTIONS) {
                Ok(_) => {
                    return Err(
                        "expected compile_program_with_format to fail for empty format".into(),
                    );
                }
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        msg.contains("format must not be empty"),
                        "unexpected error for empty format: {msg}"
                    );
                }
            }

            Ok(())
        })
    }

    #[test]
    fn compile_hlo_format_path_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();
            let result = compiler.compile(HLO_TEXT_ADD_ONE, HLO_FORMAT, COMPILE_OPTIONS);

            // This verifies the Rust wrapper forwards non-MLIR format strings through
            // PJRT_Client_Compile; backend acceptance is plugin/version specific.
            match result {
                Ok(_) => {}
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        !msg.contains("program_code must not be empty")
                            && !msg.contains("format must not be empty"),
                        "expected backend compile error or success for HLO path, got wrapper validation error: {msg}"
                    );
                }
            }

            Ok(())
        })
    }

    #[test]
    fn compile_stablehlo_format_path_smoke() -> TestResult {
        with_runtime_and_client(|_rt, client| {
            let compiler = client.compiler();
            let result =
                compiler.compile(STABLEHLO_MODULE_FUNCTION, STABLEHLO_FORMAT, COMPILE_OPTIONS);

            match result {
                Ok(_) => {}
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        !msg.contains("program_code must not be empty")
                            && !msg.contains("format must not be empty"),
                        "expected backend compile error or success for stablehlo path, got wrapper validation error: {msg}"
                    );
                }
            }

            Ok(())
        })
    }
}
