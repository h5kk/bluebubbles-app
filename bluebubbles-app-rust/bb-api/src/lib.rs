//! BlueBubbles API - HTTP client for the BlueBubbles server REST API.
//!
//! This crate provides a typed HTTP client covering all 12 REST API endpoint
//! categories exposed by the BlueBubbles macOS server. It handles authentication,
//! custom headers (ngrok/zrok), SSL certificate handling, file upload/download
//! with progress tracking, and automatic retry with exponential backoff.

pub mod client;
pub mod endpoints;
pub mod response;

// Re-export key types
pub use client::{ApiClient, RetryConfig};
pub use response::{ServerResponse, ServerPayload, PaginationMetadata};
