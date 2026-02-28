//! Global plugin state: the loaded backend library and its `PJRT_Api*`.
//!
//! `PJRT_Plugin_Initialize` must be called exactly once; it loads the backend
//! shared library whose path is taken from the `RRAD_PJRT_BACKEND_PLUGIN`
//! environment variable, obtains its `GetPjrtApi` symbol, and stores the
//! returned `*const PJRT_Api` here for all subsequent proxy dispatch
//! functions to use.

use crate::pjrt_sys::PJRT_Api;
use libloading::{Library, Symbol};
use std::sync::OnceLock;

type GetPjrtApiFn = unsafe extern "C" fn() -> *const PJRT_Api;

/// Wraps the raw `*const PJRT_Api` so we can put it in a `OnceLock<>`.
///
/// # Safety
/// `PJRT_Api` is a static dispatch-table owned by the backend library for
/// the lifetime of the process.  We never mutate it through this pointer.
struct BackendApi(*const PJRT_Api);
unsafe impl Send for BackendApi {}
unsafe impl Sync for BackendApi {}

/// Holds the loaded backend library *and* its API pointer.
///
/// The library handle must outlive every use of the API pointer, so both are
/// kept together here.  We deliberately leak the `Library` (via
/// `std::mem::forget`) because the backend must remain loaded for the entire
/// process lifetime once the plugin is initialised.
struct BackendState {
    _lib: Library,
    api: BackendApi,
}
unsafe impl Send for BackendState {}
unsafe impl Sync for BackendState {}

static BACKEND: OnceLock<BackendState> = OnceLock::new();

/// Load the backend plugin and initialise global state.
///
/// Returns `Ok(())` on success or an error string that should be surfaced via
/// a `ProxyError`.
///
/// # Contract
/// Must be called exactly once, from `PJRT_Plugin_Initialize`.
pub(super) fn load_backend() -> Result<(), String> {
    // Determine the backend path.
    let path = std::env::var("RRAD_PJRT_BACKEND_PLUGIN").map_err(|_| {
        "RRAD_PJRT_BACKEND_PLUGIN env var not set: \
         set it to the path of the PJRT backend plugin .so (e.g. pjrt_c_api_cpu_plugin.so)"
            .to_string()
    })?;

    if path.is_empty() {
        return Err("RRAD_PJRT_BACKEND_PLUGIN env var is set but empty".to_string());
    }

    // Load the library.
    let lib = unsafe { Library::new(&path) }
        .map_err(|e| format!("rrad_xla proxy: failed to load backend '{path}': {e}"))?;

    // Resolve GetPjrtApi.
    let get_api: Symbol<GetPjrtApiFn> = unsafe { lib.get(b"GetPjrtApi\0") }
        .map_err(|e| format!("rrad_xla proxy: GetPjrtApi not found in '{path}': {e}"))?;

    let api_ptr = unsafe { get_api() };
    if api_ptr.is_null() {
        return Err(format!(
            "rrad_xla proxy: GetPjrtApi returned null from '{path}'"
        ));
    }

    // Check API version compatibility.
    let ver = unsafe { (*api_ptr).pjrt_api_version };
    if ver.major_version != crate::pjrt_sys::PJRT_API_MAJOR as i32 {
        return Err(format!(
            "rrad_xla proxy: PJRT API major version mismatch \
             (proxy={}, backend={})",
            crate::pjrt_sys::PJRT_API_MAJOR,
            ver.major_version
        ));
    }

    BACKEND
        .set(BackendState {
            _lib: lib,
            api: BackendApi(api_ptr),
        })
        .map_err(|_| "rrad_xla proxy: backend already loaded (double initialize?)".to_string())
}

/// Return the backend `PJRT_Api*`, or `None` if the plugin has not been
/// initialised yet.
#[inline]
pub(super) fn backend_api() -> Option<*const PJRT_Api> {
    BACKEND.get().map(|s| s.api.0)
}
