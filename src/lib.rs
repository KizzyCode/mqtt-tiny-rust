#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
// Clippy lints
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unreachable)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::cognitive_complexity)]

pub mod anyvec;
pub mod coding;
pub mod packets;

// Re-export `arrayvec` if enabled
#[cfg(feature = "arrayvec")]
pub extern crate arrayvec;

// Re-export default type aliases
#[cfg(any(feature = "std", feature = "arrayvec"))]
include!("_defaults.rs");
