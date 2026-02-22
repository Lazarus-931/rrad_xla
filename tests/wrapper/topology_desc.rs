use rrad_pjrt::rrad_pjrt::topology_desc::PJRTTopologyDescription;
use crate::wrapper::tools::{runtime_or_skip, TestResult};

#[test]
fn serialize_deseralize_smoke() -> TestResult {

    let Some(rt) = runtime_or_skip()? else {
        return Ok(())
    };

    let client = rt.create_client()?;

    let topology = client.topology_description()?;
    let topology_description = topology.device_descriptions()?;
    let topology_name = topology.platform_name()?;
    let topology_version = topology.platform_version()?;
    let serialized_top = topology.serialize()?;
    let attr_count = topology.attributes()?.len();

    assert!(!topology_description.is_empty());
    assert!(!topology_name.is_empty());
    assert!(!serialized_top.is_empty(), "serialized topology should not be empty");


    let deserialized = PJRTTopologyDescription::deserialize(&rt, &serialized_top)?;
    let deserialized_description = deserialized.device_descriptions()?;
    let deserialized_name = deserialized.platform_name()?;
    let deserialized_version = deserialized.platform_version()?;
    let deserialized_attr_count = deserialized.attributes()?.len();

    assert_eq!(topology_name, deserialized_name);
    assert_eq!(topology_version, deserialized_version);
    assert_eq!(attr_count, deserialized_attr_count);

    Ok(())
}

#[test]
fn compile_smoke() -> TestResult {
    let Some(rt) = runtime_or_skip()? else {
        return Ok(())
    };

    let client = rt.create_client()?;
    let topology = client.topology_description()?;


}