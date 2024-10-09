//! A `Dataset` in which every point stores the distances to its `k` nearest neighbors.

use abd_clam::{
    cluster::ParCluster,
    dataset::{metric_space::ParMetricSpace, ParDataset},
    Cluster, Dataset, FlatVec, Metric, MetricSpace, Permutable,
};
use rayon::prelude::*;

use super::wasserstein::{self, wasserstein};

type Fv = FlatVec<Vec<f32>, f32, usize>;

/// A `Dataset` in which every point stores the distances to its `k` nearest neighbors.
#[allow(clippy::type_complexity)]
pub struct NeighborhoodAware {
    data: FlatVec<Vec<f32>, f32, (usize, Vec<(usize, f32)>)>,
    k: usize,
}

impl NeighborhoodAware {
    /// Create a new `NeighborhoodAware` `Dataset`.
    ///
    /// This will run knn-search on every point in the dataset and store the
    /// results in the dataset.
    pub fn new<C: Cluster<Vec<f32>, f32, Fv>>(data: &Fv, root: &C, k: usize) -> Self {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(k);

        let results = data
            .instances()
            .iter()
            .map(|query| alg.search(data, root, query))
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

        let results = data
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
    pub fn is_outlier<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>, threshold: Option<f32>) -> bool {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(self.k);
        
        let threshold = match threshold{
            Some(v) => v,
            None => 0.5f32
        };

        let hits = alg.search(self, root, query);
        let mut neighbors_distances = hits
            .iter()
            .map(|&(i, _)| (i, self.neighbor_distances(i)))
            .collect::<Vec<_>>();
        
        let avg_wass_dists = neighbors_distances.iter().enumerate().map(|(i, (_, v))|{
            
            let neighbors = neighbors_distances.clone();
            let dists = neighbors.iter().map(|(_, nv)|{
                wasserstein(v, nv)
            }).collect::<Vec<f32>>();
            
            let out: f32 = abd_clam::utils::mean(&dists);
            
            out
        }).collect::<Vec<_>>();
        
        let avg_wass_dist: f32 = abd_clam::utils::mean(&avg_wass_dists);
        
        // TODO: Compute all-pairs matrix of wasserstein distances among the neighbors' distance distributions.

        // TODO: Compute the wasserstein distances for the query

        // TODO: Optionally use the threshold to determine if the query is an outlier.
        println!();
        println!("Query:\n{:?}", query);
        println!();
        
        let wasserstein_distances = hits.iter().map(|(i, f)|{
            let t = self.data.get(*i);
            let out = wasserstein(query, t);
            println!("Algorithmic Dist: {}", f);
            println!("Wasserstein dist: {}", out);
            out
        }).collect::<Vec<_>>();
        
        let mean_wasserstein: f32 = abd_clam::utils::mean(&wasserstein_distances);
        
        println!();
        println!("{avg_wass_dist}");
        println!("{mean_wasserstein}");

        mean_wasserstein > threshold
    }

    /// Get the distances to the `k` nearest neighbors of a point.
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
