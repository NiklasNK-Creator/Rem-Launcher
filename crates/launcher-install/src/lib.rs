//! launcher-install: version manifests, verified downloads, instances.
//! Read docs/architecture.md before changing this system.
pub mod forge;
pub mod instance;
pub mod manifest;
pub use forge::*;
pub use instance::*;
pub use manifest::*;
