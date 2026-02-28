//! PyO3-based Python extension module — compiled only with `--features python`.
//!
//! Exposes the Rust PJRT wrappers directly to Python, allowing scripts (and
//! TF/JAX integration layers) to drive PJRT execution without going through a
//! separate C-ABI shared library.
//!
//! ## Build
//! ```bash
//! maturin develop --features python   # editable install
//! maturin build   --features python   # wheel
//! ```
//!
//! ## Python usage
//! ```python
//! import rrad_xla as rx
//!
//! rt     = rx.PjrtRuntime("/path/to/pjrt_c_api_cpu_plugin.so")
//! client = rt.create_client()
//! print(client.platform_name())          # "cpu"
//! print(client.device_count())           # e.g. 1
//!
//! buf = client.buffer_from_host_f32([1.0, 2.0, 3.0], [3])
//! print(buf.on_device_size_in_bytes())   # 12
//! print(buf.to_host_f32())              # [1.0, 2.0, 3.0]
//! ```

#![cfg(feature = "python")]

use crate::pjrt::buffer::PJRTBuffer;
use crate::pjrt::client::PJRTClient;
use crate::pjrt::executable::PJRTLoadedExecutable;
use crate::pjrt::loader::PjrtRuntime;
use crate::pjrt_sys::{PJRT_Buffer, PJRT_Client, PJRT_LoadedExecutable};
use crate::pjrt_sys::PJRT_Buffer_Type_PJRT_Buffer_Type_F32;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::sync::Arc;

fn to_py(e: String) -> PyErr {
    PyRuntimeError::new_err(e)
}

// ---------------------------------------------------------------------------
// Lifetime helper
// ---------------------------------------------------------------------------

/// Produce a `&'static PjrtRuntime` from an `Arc`.
///
/// # Safety
/// The caller must ensure the `Arc` is kept alive for at least as long as the
/// returned reference is used.  In practice every call site pairs this with an
/// immediate `std::mem::forget` on any borrowed value before the Arc can drop.
unsafe fn arc_as_static(rt: &Arc<PjrtRuntime>) -> &'static PjrtRuntime {
    &*(Arc::as_ptr(rt))
}

// ---------------------------------------------------------------------------
// PyPjrtRuntime
// ---------------------------------------------------------------------------

/// Loaded PJRT plugin runtime.
///
/// Call `PjrtRuntime(library_path)` to load a PJRT plugin `.so`, then
/// `create_client()` to get a client for device management and computation.
#[pyclass(name = "PjrtRuntime")]
struct PyPjrtRuntime {
    inner: Arc<PjrtRuntime>,
}

#[pymethods]
impl PyPjrtRuntime {
    /// Load the PJRT plugin at *library_path* and return a runtime handle.
    #[new]
    fn new(library_path: &str) -> PyResult<Self> {
        let rt = PjrtRuntime::load(Path::new(library_path)).map_err(to_py)?;
        Ok(PyPjrtRuntime {
            inner: Arc::new(rt),
        })
    }

    /// Call `PJRT_Plugin_Initialize` on the backend plugin.
    fn initialize(&self) -> PyResult<()> {
        self.inner.initialize_plugin().map_err(to_py)
    }

    /// Create a `PjrtClient` backed by this runtime.
    fn create_client(&self) -> PyResult<PyPjrtClient> {
        let raw = self.inner.create_client().map_err(to_py)?;
        Ok(PyPjrtClient {
            raw,
            rt: Arc::clone(&self.inner),
        })
    }
}

// ---------------------------------------------------------------------------
// PyPjrtClient
// ---------------------------------------------------------------------------

/// PJRT client: device enumeration, buffer upload, and compilation.
#[pyclass(name = "PjrtClient")]
struct PyPjrtClient {
    raw: *mut PJRT_Client,
    rt: Arc<PjrtRuntime>,
}

unsafe impl Send for PyPjrtClient {}
unsafe impl Sync for PyPjrtClient {}

impl Drop for PyPjrtClient {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            let _ = self.rt.destroy_client(self.raw);
            self.raw = std::ptr::null_mut();
        }
    }
}

#[pymethods]
impl PyPjrtClient {
    /// Platform name (e.g. `"cpu"`).
    fn platform_name(&self) -> PyResult<String> {
        // Safety: rt Arc is alive for the duration of this method.
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let c = ManuallyDrop::new(PJRTClient::new(rt, self.raw));
        c.platform_name().map_err(to_py)
    }

    /// Platform version string.
    fn platform_version(&self) -> PyResult<String> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let c = ManuallyDrop::new(PJRTClient::new(rt, self.raw));
        c.platform_version().map_err(to_py)
    }

    /// Number of addressable devices visible to this client.
    fn device_count(&self) -> PyResult<usize> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let c = ManuallyDrop::new(PJRTClient::new(rt, self.raw));
        c.devices().map(|d| d.len()).map_err(to_py)
    }

    /// Upload a flat list of `f32` values to device and return a buffer handle.
    fn buffer_from_host_f32(&self, data: Vec<f32>, shape: Vec<i64>) -> PyResult<PyPjrtBuffer> {
        // Safety: self.rt Arc is alive; we forget the returned wrappers before
        //         they can outlive the Arc.
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let c = ManuallyDrop::new(PJRTClient::new(rt, self.raw));

        let devices = c.devices().map_err(to_py)?;
        let device = devices.into_iter().next().ok_or_else(|| {
            PyRuntimeError::new_err("no addressable devices on this client")
        })?;

        let buf: PJRTBuffer<'static> = c
            .buffer_from_host_slice_copy(
                &data,
                PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
                &shape,
                Some(device),
            )
            .map_err(to_py)?;

        // Steal the raw pointer — PyPjrtBuffer takes ownership.
        let raw = buf.raw;
        std::mem::forget(buf);
        Ok(PyPjrtBuffer {
            raw,
            rt: Arc::clone(&self.rt),
        })
    }

    /// Compile an MLIR / StableHLO program and return a loaded executable.
    fn compile(&self, program_code: &str, format: &str) -> PyResult<PyPjrtExecutable> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let c = ManuallyDrop::new(PJRTClient::new(rt, self.raw));

        let exe: PJRTLoadedExecutable<'static> =
            c.compile(program_code, format, &[]).map_err(to_py)?;

        let raw = exe.raw;
        std::mem::forget(exe);
        Ok(PyPjrtExecutable {
            raw,
            rt: Arc::clone(&self.rt),
        })
    }
}

// ---------------------------------------------------------------------------
// PyPjrtBuffer
// ---------------------------------------------------------------------------

/// An on-device PJRT buffer.
#[pyclass(name = "PjrtBuffer")]
struct PyPjrtBuffer {
    raw: *mut PJRT_Buffer,
    rt: Arc<PjrtRuntime>,
}

unsafe impl Send for PyPjrtBuffer {}
unsafe impl Sync for PyPjrtBuffer {}

impl Drop for PyPjrtBuffer {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
            let b = PJRTBuffer { rt, raw: self.raw };
            drop(b); // calls PJRT_Buffer_Destroy
            self.raw = std::ptr::null_mut();
        }
    }
}

#[pymethods]
impl PyPjrtBuffer {
    /// Number of bytes this buffer occupies on device.
    fn on_device_size_in_bytes(&self) -> PyResult<usize> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let b = ManuallyDrop::new(PJRTBuffer { rt, raw: self.raw });
        b.on_device_size_in_bytes().map_err(to_py)
    }

    /// Logical shape as a list of dimension sizes.
    fn dimensions(&self) -> PyResult<Vec<i64>> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let b = ManuallyDrop::new(PJRTBuffer { rt, raw: self.raw });
        b.dimensions().map_err(to_py)
    }

    /// Download the buffer and return it as a flat list of `f32` values.
    fn to_host_f32(&self) -> PyResult<Vec<f32>> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let b = ManuallyDrop::new(PJRTBuffer { rt, raw: self.raw });
        let size = b.on_device_size_in_bytes().map_err(to_py)?;
        let mut bytes = vec![0u8; size];
        b.to_host_buffer_blocking(&mut bytes).map_err(to_py)?;
        let floats = bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        Ok(floats)
    }
}

// ---------------------------------------------------------------------------
// PyPjrtExecutable
// ---------------------------------------------------------------------------

/// A compiled, loaded PJRT executable.
#[pyclass(name = "PjrtExecutable")]
struct PyPjrtExecutable {
    raw: *mut PJRT_LoadedExecutable,
    rt: Arc<PjrtRuntime>,
}

unsafe impl Send for PyPjrtExecutable {}
unsafe impl Sync for PyPjrtExecutable {}

impl Drop for PyPjrtExecutable {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
            let e = PJRTLoadedExecutable { rt, raw: self.raw };
            drop(e); // calls PJRT_LoadedExecutable_Destroy
            self.raw = std::ptr::null_mut();
        }
    }
}

#[pymethods]
impl PyPjrtExecutable {
    /// Name of the compiled program.
    fn name(&self) -> PyResult<String> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let e = ManuallyDrop::new(PJRTLoadedExecutable { rt, raw: self.raw });
        e.name().map_err(to_py)
    }

    /// Number of output buffers produced per execution.
    fn num_outputs(&self) -> PyResult<usize> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };
        let e = ManuallyDrop::new(PJRTLoadedExecutable { rt, raw: self.raw });
        e.output_element_types().map(|t| t.len()).map_err(to_py)
    }

    /// Execute with the provided `PjrtBuffer` inputs; blocks until done.
    fn execute(&self, inputs: Vec<PyRef<'_, PyPjrtBuffer>>) -> PyResult<Vec<PyPjrtBuffer>> {
        let rt: &'static PjrtRuntime = unsafe { arc_as_static(&self.rt) };

        // Build ManuallyDrop buffer views so inputs aren't freed by their Drop.
        let tmp_bufs: Vec<ManuallyDrop<PJRTBuffer<'static>>> = inputs
            .iter()
            .map(|b| ManuallyDrop::new(PJRTBuffer { rt, raw: b.raw }))
            .collect();
        let buf_refs: Vec<&PJRTBuffer<'static>> = tmp_bufs.iter().map(|b| &**b).collect();

        let e = ManuallyDrop::new(PJRTLoadedExecutable { rt, raw: self.raw });
        let (out_bufs, event) = e.execute(&buf_refs).map_err(to_py)?;

        // Block until computation completes.
        event.await_ready().map_err(to_py)?;

        // Transfer ownership of output buffers to Python.
        let py_bufs = out_bufs
            .into_iter()
            .map(|b| {
                let raw = b.raw;
                std::mem::forget(b);
                PyPjrtBuffer {
                    raw,
                    rt: Arc::clone(&self.rt),
                }
            })
            .collect();

        Ok(py_bufs)
    }
}

// ---------------------------------------------------------------------------
// Module definition
// ---------------------------------------------------------------------------

/// `rrad_xla` — direct Python access to the Rust PJRT wrappers.
#[pymodule]
fn rrad_xla(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPjrtRuntime>()?;
    m.add_class::<PyPjrtClient>()?;
    m.add_class::<PyPjrtBuffer>()?;
    m.add_class::<PyPjrtExecutable>()?;
    Ok(())
}
