
//! rrad_pjrt - a functional, simple and memory-safe binding for XLA's rrad_pjrt_runtime
//!
//! PJRT is the device api_wrapper to XLA's (Accelerated Linear Algebra) which centralizes execution
//! across frameworks and pieces of hardware, with an independent interface in mind.
//!


#[cfg(docsrs)]
mod documentation;

pub mod buffer;
pub mod client;
pub mod compile;
pub mod copy_to_device_stream;
pub mod device;
pub mod error;
pub mod event;
pub mod executable;
pub mod execute_context;
pub mod host_to_device_manager;
pub mod loader;
pub mod memory;
pub mod topology_desc;
pub mod utils;
pub mod ffi;

