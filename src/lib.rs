pub mod pjrt;
pub mod pjrt_sys;

// PJRT proxy plugin — exports `GetPjrtApi` / `GetPjrtApiForTf`.
// Compiled whenever the `pjrt-plugin` feature is active (the default).
#[cfg(feature = "pjrt-plugin")]
pub mod pjrt_plugin;

// PyO3 Python extension module — compiled only when the `python` feature is
// explicitly requested (e.g. `cargo build --features python` or maturin).
#[cfg(feature = "python")]
pub mod python;
