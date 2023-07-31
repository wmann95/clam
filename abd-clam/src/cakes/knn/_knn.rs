//! K-Nearest Neighbor search algorithms.
//!
//! The stable algorithms are `Linear` and `RepeatedRnn`, with the default being
//! `RepeatedRnn`. We will experiment with other algorithms in the future, and they
//! will be added to this module as they are being implemented. They should not be
//! considered stable until they are documented as such.

use core::{cmp::Ordering, f64::EPSILON};

use crate::cakes::knn::knn_thresholds_no_sep_centers;
use distances::Number;

use crate::{utils, Dataset, RnnAlgorithm, Tree};

/// The multiplier to use for increasing the radius in the repeated RNN algorithm.
const MULTIPLIER: f64 = 2.0;

/// The algorithm to use for K-Nearest Neighbor search.
///
/// The default is `RepeatedRnn`, as determined by the benchmarks in the crate.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug)]
pub enum KnnAlgorithm {
    /// Use linear search on the entire dataset. This is a stable algorithm.
    Linear,
    /// Use a repeated RNN search, increasing the radius until enough neighbors
    /// are found. This is a stable algorithm.
    ///
    /// Search starts with a radius equal to the radius of the tree divided by
    /// the cardinality of the dataset. If no neighbors are found, the radius is
    /// increased by a factor of 2 until at least one neighbor is found. Then,
    /// the radius is increased by a factor determined by the local fractal
    /// dimension of the neighbors found until enough neighbors are found. This
    /// factor is capped at 2. Once enough neighbors are found, the neighbors
    /// are sorted by distance and the first `k` neighbors are returned. Ties
    /// are broken arbitrarily.
    RepeatedRnn,
    /// Use a thresholds approach to search. For each iteration of the search,
    /// a threshold is calculated based on the distance from the query to the closest cluster
    /// such that no cluster further away than the threshold can contain one
    /// of the `k` nearest neighbors.
    ///
    /// This approach does not treat the center of a cluster separately from the rest
    /// of the points in the cluster
    Thresholds,
}

impl Default for KnnAlgorithm {
    fn default() -> Self {
        Self::RepeatedRnn
    }
}

impl KnnAlgorithm {
    /// Searches for the nearest neighbors of a query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to search around.
    /// * `k` - The number of neighbors to search for.
    /// * `tree` - The tree to search.
    ///
    /// # Returns
    ///
    /// A vector of 2-tuples, where the first element is the index of the instance
    /// and the second element is the distance from the query to the instance.
    pub(crate) fn search<T, U, D>(self, query: T, k: usize, tree: &Tree<T, U, D>) -> Vec<(usize, U)>
    where
        T: Send + Sync + Copy,
        U: Number,
        D: Dataset<T, U>,
    {
        match self {
            Self::Linear => Self::linear_search(tree.data(), query, k, tree.indices()),
            Self::RepeatedRnn => Self::knn_by_rnn(tree, query, k),
            Self::Thresholds => Self::knn_by_thresholds_no_separate_centers(tree, query, k),
        }
    }

    /// Linear search for the nearest neighbors of a query.
    /// # Arguments
    ///
    /// * `data` - The dataset to search.
    /// * `query` - The query to search around.
    /// * `k` - The number of neighbors to search for.
    /// * `indices` - The indices to search.
    ///
    /// # Returns
    ///
    /// A vector of 2-tuples, where the first element is the index of the instance
    /// and the second element is the distance from the query to the instance.
    pub(crate) fn linear_search<T, U, D>(data: &D, query: T, k: usize, indices: &[usize]) -> Vec<(usize, U)>
    where
        T: Send + Sync + Copy,
        U: Number,
        D: Dataset<T, U>,
    {
        let distances = data.query_to_many(query, indices);
        let mut hits = indices.iter().copied().zip(distances.into_iter()).collect::<Vec<_>>();
        hits.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less));
        hits[..k].to_vec()
    }

    /// K-Nearest Neighbor search using a repeated RNN search.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree to search.
    /// * `query` - The query to search around.
    /// * `k` - The number of neighbors to search for.
    ///
    /// # Returns
    ///
    /// A vector of 2-tuples, where the first element is the index of the instance
    /// and the second element is the distance from the query to the instance.
    pub(crate) fn knn_by_rnn<T, U, D>(tree: &Tree<T, U, D>, query: T, k: usize) -> Vec<(usize, U)>
    where
        T: Send + Sync + Copy,
        U: Number,
        D: Dataset<T, U>,
    {
        let mut radius = EPSILON + tree.radius().as_f64() / tree.cardinality().as_f64();
        let [mut confirmed, mut straddlers] =
            RnnAlgorithm::tree_search(tree.data(), tree.root(), query, U::from(radius));

        let mut num_hits = confirmed
            .iter()
            .chain(straddlers.iter())
            .map(|&(c, _)| c.cardinality)
            .sum::<usize>();

        while num_hits == 0 {
            radius *= MULTIPLIER;
            [confirmed, straddlers] = RnnAlgorithm::tree_search(tree.data(), tree.root(), query, U::from(radius));
            num_hits = confirmed
                .iter()
                .chain(straddlers.iter())
                .map(|&(c, _)| c.cardinality)
                .sum::<usize>();
        }

        while num_hits < k {
            let lfd = utils::mean(
                &confirmed
                    .iter()
                    .chain(straddlers.iter())
                    .map(|&(c, _)| c.lfd)
                    .collect::<Vec<_>>(),
            );
            let factor = (k.as_f64() / num_hits.as_f64()).powf(1. / (lfd + EPSILON));
            assert!(factor > 1.);
            radius *= if factor < MULTIPLIER { factor } else { MULTIPLIER };
            [confirmed, straddlers] = RnnAlgorithm::tree_search(tree.data(), tree.root(), query, U::from(radius));
            num_hits = confirmed
                .iter()
                .chain(straddlers.iter())
                .map(|&(c, _)| c.cardinality)
                .sum::<usize>();
        }

        let mut hits = confirmed
            .into_iter()
            .chain(straddlers.into_iter())
            .flat_map(|(c, d)| {
                let indices = c.indices(tree.data());
                let distances = if c.is_singleton() {
                    vec![d; c.cardinality]
                } else {
                    tree.data().query_to_many(query, indices)
                };
                indices.iter().copied().zip(distances.into_iter())
            })
            .collect::<Vec<_>>();

        hits.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Greater));
        hits[..k].to_vec()
    }

    /// K-Nearest Neighbor search using a thresholds approach with no separate centers.

    pub(crate) fn knn_by_thresholds_no_separate_centers<T, U, D>(
        tree: &Tree<T, U, D>,
        query: T,
        k: usize,
    ) -> Vec<(usize, U)>
    where
        T: Send + Sync + Copy,
        U: Number,
        D: Dataset<T, U>,
    {
        let mut sieve = knn_thresholds_no_sep_centers::KnnSieve::new(tree, query, k);
        sieve.initialize_grains();
        while !sieve.is_refined() {
            sieve.refine_step();
        }
        sieve.extract()
    }
}

#[cfg(test)]

mod tests {

    use crate::cakes::Cakes;
    use crate::core::cluster::PartitionCriteria;
    use crate::core::dataset::VecDataset;
    use crate::KnnAlgorithm;
    use distances::vectors::euclidean;
    use symagen::random_data;

    #[test]
    fn test_knn_by_thresholds_no_separate_centers() {
        let f32_data = random_data::random_f32(5000, 30, 0., 10., 42);
        let f32_data = f32_data.iter().map(|v| v.as_slice()).collect::<Vec<_>>();
        let f32_data = VecDataset::new("f32_euclidean".to_string(), f32_data, euclidean::<_, f32>, false);
        let f32_query = random_data::random_f32(1, 30, 0., 1., 44);
        let f32_query = f32_query[0].as_slice();

        let criteria = PartitionCriteria::new(true).with_min_cardinality(1);
        let f32_cakes = Cakes::new(f32_data, Some(42), criteria);

        #[allow(clippy::single_element_loop)]
        for k in [10] {
            let mut jthresholds_nn = KnnAlgorithm::search(KnnAlgorithm::Thresholds, f32_query, k, f32_cakes.tree());
            let linear_nn = KnnAlgorithm::search(KnnAlgorithm::Linear, f32_query, k, f32_cakes.tree());

            thresholds_nn.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

            assert_eq!(linear_nn, thresholds_nn);
        }
    }
}