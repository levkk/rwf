//! Analytics around aplication usage.
//!
//! Work in progress, but currently handles HTTP request tracking. On the roadmap:
//!
//! * Experiments (A/B testing)

pub mod requests;

pub use requests::Request;
