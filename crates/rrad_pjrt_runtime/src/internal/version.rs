pub trait PjrtRuntimeVersion {
    fn is_compatible_with(&self, other: &Self) -> bool;
    fn is_compatible_executable_with(&self, other: &dyn PjRtExecutableAbiVersion) -> bool;

}


pub trait PjRtExecutableAbiVersion {

}
