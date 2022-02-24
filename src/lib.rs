//! AWS Nitro Secure Module API
//!
//! This is the library that provides the API for the Nitro Secure Module used in AWS Nitro
//! Enclaves for management, attestation and entropy generation.
//!
//! nsm_io provides the API and CBOR encoding functionality.
//! nsm_driver provides the ioctl interface for the Nitro Secure Module driver.

pub use nsm_driver;
pub use nsm_io;
