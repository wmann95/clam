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
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_lossless
)]
#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use abd_clam::{partition::ParPartition, Ball, Cluster, Dataset, Partition};
use clap::Parser;
use data::NeighborhoodAware;
use rayon::prelude::*;

mod data;
mod utils;

#[test]
fn test_neighborhood_aware() {
    let k = 10;
    
    let n = 1000;
    let dim = 100;
    let inlier_mean = 1200.;
    let outlier_mean = 1100.;
    let inlier_std = 300.;
    let outlier_std = 200.;
    
    let data = data::read_or_generate(
        None,
        &data::VecMetric::Wasserstein,
        Some(n),
        Some(dim),
        Some(inlier_mean),
        Some(inlier_std),
        None,
    ).unwrap();
    
    let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
    let root = Ball::new_tree(&data, &criteria, None);

    let data = data::NeighborhoodAware::new(&data, &root, k);
    
    let root = root.with_dataset_type::<NeighborhoodAware>();

    let dim = data.dimensionality_hint().0;
    let outliers = data::gen_random(outlier_mean, outlier_std, 10, dim, None);
    let inliers = data::gen_random(inlier_mean, inlier_std, 10, dim, None);

    let outlier_results: Vec<_> = outliers
            .iter()
            .map(|outlier| data.is_outlier(&root, outlier))
            .collect();
    
    let outlier_results = outlier_results.into_iter().enumerate().collect::<Vec<_>>();
    
    let inlier_results: Vec<_> = inliers
            .iter()
            .map(|inlier| data.is_outlier(&root, inlier))
            .collect();

    let inlier_results = inlier_results.into_iter().enumerate().collect::<Vec<_>>();
    
    print!("Outlier Results:\n {outlier_results:?}\n");
    print!("Inlier Results:\n {inlier_results:?}");
}
