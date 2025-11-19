//! Mock OpenAI API library
//!
//! This library exposes the internal modules for use in benchmarks and tests.

pub mod args;
pub mod endpoints;
pub mod tls;
pub mod types;
pub mod utils;

pub use types::AppState;
pub use endpoints::*;
