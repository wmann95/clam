//! A `Dataset` in which every point stores the distances to its `k` nearest neighbors.

use core::f32;

use abd_clam::{
    cluster::ParCluster, dataset::{metric_space::ParMetricSpace, ParDataset}, utils::{mean, standard_deviation}, Cluster, Dataset, FlatVec, Metric, MetricSpace, Permutable
};
use distances::number::Addition;
use rayon::prelude::*;

use crate::{data::wasserstein, utils::{normalize_distances, normalize_distribution}};

use super::wasserstein::wasserstein;

type Fv = FlatVec<Vec<f32>, f32, usize>;

/// A `Dataset` in which every point stores the distances to its `k` nearest neighbors.
#[allow(clippy::type_complexity)]
pub struct NeighborhoodAware {
    data: FlatVec<Vec<f32>, f32, (usize, Vec<(usize, f32)>)>,
    k: usize,
}

#[allow(dead_code)]
impl NeighborhoodAware {
    /// Create a new `NeighborhoodAware` `Dataset`.
    ///
    /// This will run knn-search on every point in the dataset and store the
    /// results in the dataset.
    pub fn new<C: Cluster<Vec<f32>, f32, Fv>>(data: &Fv, root: &C, k: usize) -> Self {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(k);

        let results: Vec<(usize, Vec<(usize, f32)>)> = data
            .instances()
            .iter()
            .enumerate()
            .map(|(_, query)| alg.search(data, root, query))
            .zip(data.metadata().iter())
            .map(|(h, &i)| (i, h))
            .collect();
        
        let data = data
            .clone()
            .with_metadata(results)
            .unwrap_or_else(|e| unreachable!("We created the correct size for neighborhood aware data: {e}"));
        Self { data, k }
    }

    /// Parallel version of `new`.
    pub fn par_new<C: ParCluster<Vec<f32>, f32, Fv>>(data: &Fv, root: &C, k: usize) -> Self {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(k);

        let results: Vec<(usize, Vec<(usize, f32)>)> = data
            .instances()
            .par_iter()
            .map(|query| alg.par_search(data, root, query))
            .zip(data.metadata().par_iter())
            .map(|(h, &i)| (i, h))
            .collect();
        
        let data = data
            .clone()
            .with_metadata(results)
            .unwrap_or_else(|e| unreachable!("We created the correct size for neighborhood aware data: {e}"));
        Self { data, k }
    }
    
    
    
    pub fn outlier_score<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>) -> f32 {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(self.k);
        
        // find the k-nearest neighbors of this query point.
        let hits = alg.search(self, root, query);
        
        // for each of the neighbors, retrieve THEIR k-nearest neighbors
        let neighbors_distances = hits
            .iter()
            .map(|&(i, _)| {
                self.neighbor_distances(i)
            })
            .collect::<Vec<_>>();
        
        // Get the normalized distances of each of the neighbors...
        // produces incoherent results.
        /*
        let neighbors_distances_normalized = neighbors_distances.iter()
            .map(|v| normalize_distances(v))
            .collect::<Vec<_>>();
         */
        
        // make a vector from the query neighbors consisting only
        // of floats, without the otherwise included neighbor index.
        let query_distances = hits.iter()
            .map(|&(_, f)| f)
            .collect::<Vec<_>>();
        
        // Same as previous normalized distances.
        //let query_distances_normalized = normalize_distances(&query_distances);
        
        // Normalize the query distances into a vector with range of
        // 0..1 which sums to 1.0
        let query_distribution_normalized = normalize_distribution(&query_distances);
        
        // Normalize each of the neighbor distance vectors the same way
        // as the query.
        let distance_distribution_mat = neighbors_distances.iter()
            .map(|v| normalize_distribution(v))
            .collect::<Vec<Vec<f32>>>();
        
        // Find the Wasserstein distance between each of the neighbors'
        // normalized neighbor distances.
        let dist_distrib_wass_mat = distance_distribution_mat.iter()
            .map(|v| {
                distance_distribution_mat.iter()
                .map(|q| wasserstein(&v, q))
                .collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>();
        
        // find the Wasserstein distance between the query and its
        // neighbors.
        let query_wass = distance_distribution_mat.iter()
            .map(|v| wasserstein(&query_distribution_normalized, v))
            .collect::<Vec<f32>>();
        
        let dist_distrib_means = dist_distrib_wass_mat.iter()
            .map(|v| mean(&v))
            .collect::<Vec<f32>>();
        
        let dist_distrib_stds = dist_distrib_wass_mat.iter()
            .map(|v| {
                standard_deviation(v)
            })
            .collect::<Vec<f32>>();
        
        let zipped_means_stds = dist_distrib_means.iter()
            .zip(dist_distrib_stds.iter())
            .collect::<Vec<(&f32, &f32)>>();
        
        let query_mean: f32 = mean(&query_wass);
        
        let out = zipped_means_stds.iter()
            .map(|(&mean, &std)|{
                let dist_between_means = mean.abs_diff(query_mean);
                let var = dist_between_means / std;
                println!("{var}");
                var
            })
            .collect::<Vec<f32>>();
        
        let out_mean: f32 = mean(&out);
        
        // println!("{}", out_mean);
        // println!();
        
        // let r1 = dist_mat[0].clone();
        
        // let variations = dist_mat[1..].iter()
        //     .fold(r1, |acc, v|{
        //         acc.iter().zip(v.iter()).map(|(a, b)| a + b).collect::<Vec<_>>()
        //     }).iter()
        //     .map(|&f| (f / (dist_mat.len() as f32)).sqrt())
        //     .collect::<Vec<_>>();
        
        // println!("{:?}", variations);
        // println!();
        
        
        // let wasserstein_distances = neighbors_distances_normalized.iter().map(|v|{
        //     wasserstein(&query_distances_normalized, v)
        // }).collect::<Vec<f32>>();
        
        // let out = wasserstein_distances.iter().zip(variations.iter()).map(|(&a, &b)|{
        //     a.abs_diff(b).sqrt()
        // }).collect::<Vec<_>>();
        
        // // println!("{:?}", out);
        
        out.iter().sum::<f32>().sqrt()
    }
    
    /// Check if a point is an outlier.
    // pub fn is_outlier<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>) -> bool {
        
        
    //     // TODO: What am I using the dist_mat for? Am I comparing wasserstein_distances to the distances there?
    //     //       Am I to find the max of each of the inner arrays, then comparing that to wasserstein_distances?
    //     //       What is the intended means to collapse this into a single result? Is it just that if the
    //     //       difference between 
        
    //     // guessing here
        
    //     // let max_dist = dist_mat.iter().flatten().fold(f32::NEG_INFINITY, |out, f|{
    //     //     let f = f.clone();
    //     //     if out < f{
    //     //         f
    //     //     }
    //     //     else{
    //     //         out
    //     //     }
    //     // });
        
    //     // println!("{}", max_dist);
    //     // println!();
        
    //     // wasserstein_distances.iter().filter(|f| **f > max_dist).collect::<Vec<_>>().len() > 0
    // }

    /// Get the distances to the `k` nearest neighbors of a point.
    // fn neighbor_distances(&self, i: usize) -> Vec<f32> {
    //     self.data.metadata()[i].1.iter().map(|&(_, d)| d).collect()
    // }
    
    fn neighbor_distances(&self, i: usize) -> Vec<f32> {
        self.data.metadata()[i].1.iter().filter(|(ind, _)| *ind != i).map(|&(_, d)| d).collect()
    }
}

impl MetricSpace<Vec<f32>, f32> for NeighborhoodAware {
    fn metric(&self) -> &Metric<Vec<f32>, f32> {
        self.data.metric()
    }

    fn set_metric(&mut self, metric: Metric<Vec<f32>, f32>) {
        self.data.set_metric(metric);
    }
}

impl Dataset<Vec<f32>, f32> for NeighborhoodAware {
    fn name(&self) -> &str {
        self.data.name()
    }

    fn with_name(self, name: &str) -> Self {
        Self {
            data: self.data.with_name(name),
            k: self.k,
        }
    }

    fn cardinality(&self) -> usize {
        self.data.cardinality()
    }

    fn dimensionality_hint(&self) -> (usize, Option<usize>) {
        self.data.dimensionality_hint()
    }

    fn get(&self, index: usize) -> &Vec<f32> {
        self.data.get(index)
    }
}

impl Permutable for NeighborhoodAware {
    fn permutation(&self) -> Vec<usize> {
        self.data.permutation()
    }

    fn set_permutation(&mut self, permutation: &[usize]) {
        self.data.set_permutation(permutation);
    }

    fn swap_two(&mut self, i: usize, j: usize) {
        self.data.swap_two(i, j);
    }
}

impl ParMetricSpace<Vec<f32>, f32> for NeighborhoodAware {}

impl ParDataset<Vec<f32>, f32> for NeighborhoodAware {}
