#![deny(clippy::correctness)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_lossless
)]
#![doc = include_str!("../README.md")]

pub mod cakes;
mod core;
pub mod utils;

pub use crate::core::{
    adapter, cluster, dataset, partition, BalancedBall, Ball, Cluster, Dataset, FlatVec, Metric, MetricSpace, Partition,
    Permutable, LFD,
};

#[cfg(feature = "chaoda")]
mod chaoda;
#[cfg(feature = "chaoda")]
pub use crate::chaoda::{Algorithm, Chaoda, Member, MlModel, Vertex};

/// Used to bypass the recursion limit in Rust.
pub const MAX_RECURSION_DEPTH: usize = 128;

/// The current version of the crate.
pub const VERSION: &str = "0.31.0";
