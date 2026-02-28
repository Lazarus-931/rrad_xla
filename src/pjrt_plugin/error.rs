//! Proxy-owned PJRT error objects.
//!
//! When the Rust PJRT plugin proxy needs to surface an error *before* the
//! backend plugin is loaded (e.g. during `PJRT_Plugin_Initialize` if the
//! backend .so cannot be found), it creates a `ProxyError` on the heap and
//! casts it to `*mut PJRT_Error`.  A magic sentinel at the start of the struct
//! lets the proxy's `PJRT_Error_*` dispatch functions distinguish these from
//! backend-owned errors.

use crate::pjrt_sys::{PJRT_Error, PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_UNKNOWN};

/// Magic value written at offset 0 of every `ProxyError` so we can recognise
/// our own error pointers without external bookkeeping.
const PROXY_ERROR_MAGIC: u64 = 0x5252_4144_5F45_5252; // "RRAD_ERR"

/// Heap-allocated error used by the proxy before the backend is initialised.
#[repr(C)]
pub(super) struct ProxyError {
    /// Must stay at offset 0; used for `is_proxy_error` detection.
    magic: u64,
    pub(super) code: PJRT_Error_Code,
    /// Owned message bytes (not null-terminated).
    message: *mut u8,
    message_len: usize,
}

// Safety: `ProxyError` objects are only accessed through the single-threaded
// PJRT C-API call chain (initialize → error-message → error-destroy) and we
// never alias the interior pointers.
unsafe impl Send for ProxyError {}
unsafe impl Sync for ProxyError {}

/// Allocate a `ProxyError` on the heap and return it cast to `*mut PJRT_Error`.
///
/// The caller is responsible for ensuring the returned pointer is eventually
/// passed to `destroy_proxy_error`, or to the proxy's `PJRT_Error_Destroy`
/// dispatch function.
pub(super) fn make_proxy_error(code: PJRT_Error_Code, message: &str) -> *mut PJRT_Error {
    let msg_bytes: Box<[u8]> = message.as_bytes().into();
    let message_len = msg_bytes.len();
    let message = Box::into_raw(msg_bytes) as *mut u8;

    let err = Box::new(ProxyError {
        magic: PROXY_ERROR_MAGIC,
        code,
        message,
        message_len,
    });
    Box::into_raw(err) as *mut PJRT_Error
}

/// Returns `true` if `err` was created by `make_proxy_error`.
///
/// # Safety
/// `err` must be a valid non-null pointer to at least `size_of::<u64>()`
/// bytes.  This is always the case for any non-null `PJRT_Error*` pointer
/// because both backend errors and proxy errors are heap objects larger than
/// 8 bytes.
#[inline]
pub(super) unsafe fn is_proxy_error(err: *const PJRT_Error) -> bool {
    if err.is_null() {
        return false;
    }
    let magic = (err as *const u64).read_unaligned();
    magic == PROXY_ERROR_MAGIC
}

/// Returns the message bytes stored in a proxy error.
///
/// # Safety
/// `err` must be a valid `ProxyError` pointer (check with `is_proxy_error`
/// first).
#[inline]
pub(super) unsafe fn proxy_error_message(err: *const PJRT_Error) -> &'static [u8] {
    let pe = err as *const ProxyError;
    std::slice::from_raw_parts((*pe).message, (*pe).message_len)
}

/// Returns the error code stored in a proxy error.
///
/// # Safety
/// `err` must be a valid `ProxyError` pointer.
#[inline]
pub(super) unsafe fn proxy_error_code(err: *const PJRT_Error) -> PJRT_Error_Code {
    (*(err as *const ProxyError)).code
}

/// Free a proxy error created by `make_proxy_error`.
///
/// # Safety
/// `err` must be a valid, non-null `ProxyError` pointer that has not been
/// freed before.
pub(super) unsafe fn destroy_proxy_error(err: *mut PJRT_Error) {
    let pe = err as *mut ProxyError;
    // Reconstruct the boxed message slice and drop it.
    let _msg = Box::from_raw(std::slice::from_raw_parts_mut((*pe).message, (*pe).message_len));
    // Now drop the ProxyError itself.
    let _ = Box::from_raw(pe);
}

/// Create an `UNKNOWN` proxy error from a Rust `String`.
pub(super) fn proxy_error_from_string(msg: String) -> *mut PJRT_Error {
    make_proxy_error(PJRT_Error_Code_PJRT_Error_Code_UNKNOWN, &msg)
}
