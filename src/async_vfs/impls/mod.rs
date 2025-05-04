//! Async Virtual filesystem implementations

pub mod altroot;
pub mod memory;
pub mod overlay;

#[cfg(not(target_os = "unknown"))]
pub mod physical;
