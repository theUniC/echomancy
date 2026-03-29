//! CLIPS I/O router stub for M1.
//!
//! CLIPS has a router system that intercepts all I/O (stdout, stderr, warnings).
//! In M1 we note the required FFI surface but defer actual router registration
//! to a later milestone. Error detection in M1 relies on return-value checking
//! from `LoadFromString`, `AssertString`, and `Run`.
//!
//! # Future work (M2+)
//!
//! Register a named router via CLIPS `AddRouter` that:
//! - Captures `"stderr"` → forwards to `tracing::error!()`
//! - Captures `"stdwrn"` → forwards to `tracing::warn!()`
//! - Captures `"stdout"` → forwards to `tracing::debug!()` (watch traces)
//!
//! This requires adding `AddRouter` to `clips-sys` (the C function exists in
//! CLIPS 6.4.2's `router.h`).
