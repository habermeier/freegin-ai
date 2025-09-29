#![warn(missing_docs)]
#![allow(unused_crate_dependencies)]

//! Library entry point exposing the project's modules for reuse in the binary
//! and integration tests.

pub mod config;
pub mod credentials;
pub mod database;
pub mod error;
pub mod models;
pub mod providers;
pub mod routes;
pub mod usage;

pub use routes::AppState;
