//! Wall-clock and monotonic time sources that work on every target.
//!
//! `std::time::Instant` and `std::time::SystemTime` panic on
//! `wasm32-unknown-unknown`; the `web-time` crate provides drop-in
//! replacements backed by `performance.now()` / `Date`. Importing from this
//! module instead of `std::time` gives every call site the right type per
//! target with no other change. `Duration` intentionally stays `std` (it is
//! pure arithmetic and identical on both targets).
//!
//! ```
//! use common::clock::Instant;
//!
//! let start = Instant::now();
//! let elapsed = start.elapsed();
//! assert!(elapsed.as_secs_f32() >= 0.0);
//! ```

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
pub use web_time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH};
