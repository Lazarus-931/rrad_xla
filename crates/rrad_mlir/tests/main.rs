use rrad_mlir::{ModuleText, ProgramFormat};

#[test]
fn module_constructor_rejects_empty_source() {
    let err = ModuleText::new("   ", ProgramFormat::Mlir).unwrap_err();
    assert!(err.to_string().contains("source"));
}

#[test]
fn module_format_name_mapping_smoke() {
    let mlir = ModuleText::mlir("module { func.func @main() -> () { return } }").unwrap();
    let stablehlo = ModuleText::stablehlo("module { func.func @main() -> () { return } }").unwrap();
    let hlo = ModuleText::hlo("HloModule test\nENTRY main() -> f32[] { ROOT sys = f32[] constant(1) }").unwrap();

    assert_eq!(mlir.format_name(), "mlir");
    assert_eq!(stablehlo.format_name(), "stablehlo");
    assert_eq!(hlo.format_name(), "hlo");
}

#[test]
fn custom_format_is_preserved() {
    let module = ModuleText::new(
        "module { func.func @main() -> () { return } }",
        ProgramFormat::Custom("my-format".to_string()),
    )
    .unwrap();

    assert_eq!(module.format_name(), "my-format");
}
