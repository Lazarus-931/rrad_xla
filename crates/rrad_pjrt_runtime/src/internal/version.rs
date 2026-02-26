pub trait PjrtRuntimeVersion {
    fn is_compatible_with(&self, other: &Self) -> bool;
    fn is_compatible_executable_with(&self, other: &PjRtExecutableAbiVersion) -> bool;

}


pub trait PjRtExecutableAbiVersion {

}
