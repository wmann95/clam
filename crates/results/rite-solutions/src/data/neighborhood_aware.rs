//! A `Dataset` in which every point stores the distances to its `k` nearest neighbors.

use abd_clam::{
    cluster::ParCluster, dataset::{metric_space::ParMetricSpace, ParDataset}, utils::mean, Cluster, Dataset, FlatVec, Metric, MetricSpace, Permutable
};
use rayon::prelude::*;

use crate::data::wasserstein;

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
    
    /// Check if a point is an outlier.
    pub fn is_outlier<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>) -> bool {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(self.k);
        
        let hits = alg.search(self, root, query);
        let neighbors_distances = hits
            .iter()
            .map(|&(i, _)| {
                self.neighbor_distances(i)
            })
            .collect::<Vec<_>>();
        
        let dist_mat = neighbors_distances.iter().map(|v| {
            neighbors_distances.iter().map(|q| wasserstein(v, q)).collect::<Vec<f32>>()
        }).collect::<Vec<Vec<f32>>>();
        
        for a in &dist_mat{
            println!("{:?}", *a);
        }
        
        let query_distances = hits.iter().map(|&(_, d)| d).collect::<Vec<_>>();
        
        let wasserstein_distances = neighbors_distances.iter().map(|v|{
            wasserstein(&query_distances, v)
        }).collect::<Vec<f32>>();
        
        println!();
        println!("{:?}", wasserstein_distances);
        
        // TODO: What am I using the dist_mat for? Am I comparing wasserstein_distances to the distances there?
        //       Am I to find the max of each of the inner arrays, then comparing that to wasserstein_distances?
        //       What is the intended means to collapse this into a single result? Is it just that if the
        //       difference between 
        
        // guessing here
        
        let max_dist = dist_mat.iter().flatten().fold(f32::NEG_INFINITY, |out, f|{
            let f = f.clone();
            if out < f{
                f
            }
            else{
                out
            }
        });
        
        println!("{}", max_dist);
        println!();
        
        wasserstein_distances.iter().filter(|f| **f > max_dist).collect::<Vec<_>>().len() > 0
    }

    /// Get the distances to the `k` nearest neighbors of a point.
    // fn neighbor_distances(&self, i: usize) -> Vec<f32> {
    //     self.data.metadata()[i].1.iter().map(|&(_, d)| d).collect()
    // }
    
    fn neighbor_distances(&self, i: usize) -> Vec<f32> {
        self.data.metadata()[i].1.iter().map(|&(_, d)| d).collect()
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
