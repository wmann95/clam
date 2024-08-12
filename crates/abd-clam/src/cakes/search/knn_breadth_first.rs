//! K-Nearest Neighbors search using a Breadth First sieve.

use core::cmp::Reverse;

use distances::Number;
use rayon::prelude::*;

use crate::{cluster::ParCluster, dataset::ParDataset, linear_search::SizedHeap, Cluster, Dataset};

/// K-Nearest Neighbors search using a Breadth First sieve.
pub fn search<I, U, D, C>(data: &D, root: &C, query: &I, k: usize) -> Vec<(usize, U)>
where
    U: Number,
    D: Dataset<I, U>,
    C: Cluster<I, U, D>,
{
    let mut candidates = SizedHeap::<(Reverse<U>, &C)>::new(None);
    let mut hits = SizedHeap::<(U, usize)>::new(Some(k));

    let d = root.distance_to_center(data, query);
    candidates.push((Reverse(d_max(root, d)), root));

    while !candidates.is_empty() {
        let [needed, maybe_needed, _] = split_candidates(&hits, candidates);

        let (leaves, parents) = needed
            .into_iter()
            .chain(maybe_needed)
            .partition::<Vec<_>, _>(|(_, c)| c.is_leaf());

        for (d, c) in leaves {
            if c.is_singleton() {
                c.indices().for_each(|i| hits.push((d, i)));
            } else {
                c.distances_to_query(data, query)
                    .into_iter()
                    .for_each(|(i, d)| hits.push((d, i)));
            }
        }

        candidates = SizedHeap::new(None);
        for (_, p) in parents {
            p.child_clusters()
                .map(|c| (c, c.distance_to_center(data, query)))
                .for_each(|(c, d)| candidates.push((Reverse(d_max(c, d)), c)));
        }
    }

    hits.items().map(|(d, i)| (i, d)).collect()
}

/// Parallel K-Nearest Neighbors search using a Breadth First sieve.
pub fn par_search<I, U, D, C>(data: &D, root: &C, query: &I, k: usize) -> Vec<(usize, U)>
where
    I: Send + Sync,
    U: Number,
    D: ParDataset<I, U>,
    C: ParCluster<I, U, D>,
{
    let mut candidates = SizedHeap::<(Reverse<U>, &C)>::new(None);
    let mut hits = SizedHeap::<(U, usize)>::new(Some(k));

    let d = root.distance_to_center(data, query);
    candidates.push((Reverse(d_max(root, d)), root));

    while !candidates.is_empty() {
        let [needed, maybe_needed, _] = split_candidates(&hits, candidates);

        let (leaves, parents) = needed
            .into_iter()
            .chain(maybe_needed)
            .partition::<Vec<_>, _>(|(_, c)| c.is_leaf());

        for (d, c) in leaves {
            if c.is_singleton() {
                c.indices().for_each(|i| hits.push((d, i)));
            } else {
                c.par_distances_to_query(data, query)
                    .into_iter()
                    .for_each(|(i, d)| hits.push((d, i)));
            }
        }

        candidates = SizedHeap::new(None);
        let distances = parents
            .into_par_iter()
            .flat_map(|(_, p)| p.child_clusters().collect::<Vec<_>>())
            .map(|c| (c, c.distance_to_center(data, query)))
            .collect::<Vec<_>>();
        distances
            .into_iter()
            .for_each(|(c, d)| candidates.push((Reverse(d_max(c, d)), c)));
    }

    hits.items().map(|(d, i)| (i, d)).collect()
}

/// Returns the theoretical maximum distance from the query to a point in the cluster.
fn d_max<I, U: Number, D: Dataset<I, U>, C: Cluster<I, U, D>>(c: &C, d: U) -> U {
    c.radius() + d
}

/// Splits the candidates three ways: those needed to get to k hits, those that
/// might be needed to get to k hits, and those that are not needed to get to k
/// hits.
fn split_candidates<'a, I, U, D, C>(
    hits: &SizedHeap<(U, usize)>,
    candidates: SizedHeap<(Reverse<U>, &'a C)>,
) -> [Vec<(U, &'a C)>; 3]
where
    U: Number,
    D: Dataset<I, U>,
    C: Cluster<I, U, D>,
{
    let k = hits
        .k()
        .unwrap_or_else(|| unreachable!("The `hits` heap should have a maximum size."));
    let items = candidates.items().map(|(Reverse(d), c)| (d, c)).collect::<Vec<_>>();
    let threshold_index = items
        .iter()
        .scan(hits.len(), |num_hits_so_far, (_, c)| {
            *num_hits_so_far += c.cardinality();
            Some(*num_hits_so_far)
        })
        .position(|num_hits| num_hits > k)
        .unwrap_or_else(|| items.len() - 1);
    let threshold = {
        let (d, _) = items[threshold_index];
        let kth_distance = hits.peek().map_or(U::ZERO, |(d, _)| *d);
        if d < kth_distance {
            kth_distance
        } else {
            d
        }
    };

    let (needed, items) = items.into_iter().partition::<Vec<_>, _>(|(d, _)| *d < threshold);

    let (not_needed, maybe_needed) = items
        .into_iter()
        .map(|(d, c)| {
            let diam = c.radius().double();
            if d <= diam {
                (d, U::ZERO, c)
            } else {
                (d, d - diam, c)
            }
        })
        .partition::<Vec<_>, _>(|(_, d, _)| *d > threshold);

    let not_needed = not_needed.into_iter().map(|(d, _, c)| (d, c)).collect();
    let maybe_needed = maybe_needed.into_iter().map(|(d, _, c)| (d, c)).collect();

    [needed, maybe_needed, not_needed]
}

#[cfg(test)]
mod tests {
    use crate::{
        adapter::BallAdapter,
        cakes::OffBall,
        cluster::{Ball, Partition},
        Cluster,
    };

    use super::super::knn_depth_first::tests::check_knn;
    use crate::cakes::tests::{gen_grid_data, gen_line_data};

    #[test]
    fn line() -> Result<(), String> {
        let data = gen_line_data(10)?;
        let query = &0;

        let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
        let seed = Some(42);

        let ball = Ball::new_tree(&data, &criteria, seed);
        for k in [1, 4, 8] {
            assert!(check_knn(&ball, &data, query, k));
        }

        let (off_ball, perm_data) = OffBall::from_ball_tree(ball, data);
        for k in [1, 4, 8] {
            assert!(check_knn(&off_ball, &perm_data, query, k));
        }

        Ok(())
    }

    #[test]
    fn grid() -> Result<(), String> {
        let data = gen_grid_data(10)?;
        let query = &(0.0, 0.0);

        let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
        let seed = Some(42);

        let ball = Ball::new_tree(&data, &criteria, seed);
        for k in [1, 4, 8] {
            assert!(check_knn(&ball, &data, query, k));
        }

        let (off_ball, perm_data) = OffBall::from_ball_tree(ball, data);
        for k in [1, 4, 8] {
            assert!(check_knn(&off_ball, &perm_data, query, k));
        }

        Ok(())
    }
}
