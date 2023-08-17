//! Linear search for the k nearest neighbors of a query.

use distances::Number;

use crate::Dataset;

use super::Hits;

/// Linear search for the nearest neighbors of a query.
///
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
pub fn search<T, U, D>(data: &D, query: T, k: usize, indices: &[usize]) -> Vec<(usize, U)>
where
    T: Send + Sync + Copy,
    U: Number,
    D: Dataset<T, U>,
{
    let distances = data.query_to_many(query, indices);

    let mut hits = Hits::new(k);
    indices
        .iter()
        .zip(distances.iter())
        .for_each(|(&i, &d)| hits.push(i, d));
    hits.extract()
}

#[cfg(test)]
mod tests {

    use distances::vectors::euclidean;
    use symagen::random_data;

    use crate::{Cakes, PartitionCriteria, VecDataset};

    #[test]
    fn tiny() {
        let data = (1..=10).map(|i| vec![i as f32]).collect::<Vec<_>>();
        let data = data.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let data = VecDataset::new("tiny".to_string(), data, euclidean::<_, f32>, false);

        let query = vec![0.0];

        let criteria = PartitionCriteria::default();
        let model = Cakes::new(data, None, criteria);
        let tree = model.tree();

        let linear_nn = super::search(tree.data(), &query, 3, tree.indices());
        assert_eq!(linear_nn.len(), 3);

        let distances = {
            let mut distances = linear_nn.iter().map(|(_, d)| *d).collect::<Vec<_>>();
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
            distances
        };
        let true_distances = vec![1.0, 2.0, 3.0];

        assert_eq!(distances, true_distances);
    }

    #[test]
    fn linear() {
        let (cardinality, dimensionality) = (1_000, 10);
        let (min_val, max_val) = (-1.0, 1.0);
        let seed = 42;

        let data = random_data::random_f32(cardinality, dimensionality, min_val, max_val, seed);
        let data = data.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let data = VecDataset::new("knn-test".to_string(), data, euclidean::<_, f32>, false);

        let query = random_data::random_f32(1, dimensionality, min_val, max_val, seed * 2);
        let query = query[0].as_slice();

        let criteria = PartitionCriteria::default();
        let model = Cakes::new(data, Some(seed), criteria);
        let tree = model.tree();

        for k in [100, 10, 1] {
            let linear_nn = super::search(tree.data(), query, k, tree.indices());

            assert_eq!(
                linear_nn.len(),
                k,
                "Linear search returned {} neighbors instead of {}",
                linear_nn.len(),
                k
            );
        }
    }
}