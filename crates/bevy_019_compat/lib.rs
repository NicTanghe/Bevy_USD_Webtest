//! Compatibility facade for crates that still require the Bevy 0.19 package.
//!
//! All public items come from the exact Bevy revision used by `usd_bevy`, so
//! components, resources, plugins, entities, and render assets share one type
//! identity throughout the application.

pub use bevy_pinned::*;
