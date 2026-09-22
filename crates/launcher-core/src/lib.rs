//! launcher-core: provider-neutral models + provider abstraction.
//! Read docs/architecture.md before changing this system.

pub mod models;
pub mod providers;

pub use models::*;
pub use providers::{ContentProvider, ProviderId, ProviderRegistry};
