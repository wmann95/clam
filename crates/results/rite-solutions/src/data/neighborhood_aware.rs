//! A `Dataset` in which every point stores the distances to its `k` nearest neighbors.

use abd_clam::{
    cluster::ParCluster, dataset::{metric_space::ParMetricSpace, ParDataset}, utils::mean, Cluster, Dataset, FlatVec, Metric, MetricSpace, Permutable
};
use rayon::prelude::*;

use super::wasserstein::wasserstein;

type Fv = FlatVec<Vec<f32>, f32, usize>;

/// A `Dataset` in which every point stores the distances to its `k` nearest neighbors.
#[allow(clippy::type_complexity)]
pub struct NeighborhoodAware {
    data: FlatVec<Vec<f32>, f32, (usize, Vec<(usize, f32)>)>,
    std: f32,
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
        
        let variance = results.iter().map(|(_, v)|{
            let r = v.iter().map(|&(_, d)| d).collect::<Vec<f32>>();
            let out: f32 = mean(&r);
            out
        }).collect::<Vec<f32>>();
        
        let variance: f32 = mean(&variance);
        
        let std = variance.sqrt() as f32;

        let data = data
            .clone()
            .with_metadata(results)
            .unwrap_or_else(|e| unreachable!("We created the correct size for neighborhood aware data: {e}"));
        Self { data, std, k }
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

        let variance = results.par_iter().map(|(_, v)|{
            let r = v.par_iter().map(|&(_, d)| d as f64).collect::<Vec<f64>>();
            let out: f64 = mean(&r);
            out
        }).collect::<Vec<f64>>();
        
        let variance: f64 = mean(&variance);
        
        let std = variance.sqrt() as f32;
        
        let data = data
            .clone()
            .with_metadata(results)
            .unwrap_or_else(|e| unreachable!("We created the correct size for neighborhood aware data: {e}"));
        Self { data, std, k }
    }

    pub fn outlier_score<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>) -> f32 {
        let alg = abd_clam::cakes::Algorithm::KnnLinear(self.k);
        
        let hits = alg.search(self, root, query);
        let neighbors_distances = hits
            .iter()
            .map(|&(i, _)| {
                let dists = self.neighbor_distances(i);
                let out: f32 = abd_clam::utils::mean(&dists);
                out
            })
            .collect::<Vec<_>>();
        
        
        let neighbors_variance: f32 = abd_clam::utils::mean(&neighbors_distances);
        
        let wasserstein_distances = hits.iter().map(|(i, _)|{
            let t = self.data.get(*i);
            let out: f32 = wasserstein(query, t);
            out
        }).collect::<Vec<_>>();
        
        let mean_wasserstein: f32 = abd_clam::utils::mean(&wasserstein_distances);
        
        let neighbors_deviation = neighbors_variance.sqrt();
        let deviation = mean_wasserstein.sqrt();
        
        deviation / neighbors_deviation
    }
    
    /// Check if a point is an outlier.
    pub fn is_outlier<C: Cluster<Vec<f32>, f32, Self>>(&self, root: &C, query: &Vec<f32>) -> bool {
        let res = self.outlier_score(root, query);

        res > 2.0
    }

    /// Get the distances to the `k` nearest neighbors of a point.
    // fn neighbor_distances(&self, i: usize) -> Vec<f32> {
    //     self.data.metadata()[i].1.iter().map(|&(_, d)| d).collect()
    // }
    
    fn neighbor_distances(&self, i: usize) -> Vec<f32> {
        self.data.metadata()[i].1.iter().map(|&(j, _)| {
            wasserstein(self.get(i), self.get(j))
        }).collect()
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
            std: self.std,
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
