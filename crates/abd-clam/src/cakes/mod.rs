//! Entropy Scaling Search

pub mod cluster;
pub(crate) mod codec;
pub mod dataset;
mod search;

pub use cluster::OffBall;
pub use codec::{
    CodecData, Compressible, Decodable, Decompressible, Encodable, ParCompressible, ParDecompressible, SquishyBall,
};
pub use dataset::Shardable;
pub use search::Algorithm;

#[cfg(test)]
pub mod tests {
    use distances::{number::Float, Number};
    use rand::prelude::*;
    use test_case::test_case;

    pub type Algs<U> = Vec<(super::Algorithm<U>, fn(Vec<(usize, U)>, Vec<(usize, U)>, &str) -> bool)>;

    use crate::{
        adapter::{BallAdapter, ParBallAdapter},
        cakes::{OffBall, SquishyBall},
        cluster::ParCluster,
        dataset::ParDataset,
        linear_search::ParLinearSearch,
        Ball, Cluster, FlatVec, Metric, Partition,
    };

    pub fn gen_line_data(max: i32) -> Result<FlatVec<i32, u32, usize>, String> {
        let data = (-max..=max).collect::<Vec<_>>();
        let distance_fn = |a: &i32, b: &i32| a.abs_diff(*b);
        let metric = Metric::new(distance_fn, false);
        FlatVec::new(data, metric)
    }

    pub fn gen_grid_data(max: i32) -> Result<FlatVec<(f32, f32), f32, usize>, String> {
        let data = (-max..=max)
            .flat_map(|x| (-max..=max).map(move |y| (x.as_f32(), y.as_f32())))
            .collect::<Vec<_>>();
        let distance_fn = |(x1, y1): &(f32, f32), (x2, y2): &(f32, f32)| (x1 - x2).hypot(y1 - y2);
        let metric = Metric::new(distance_fn, false);
        FlatVec::new(data, metric)
    }

    pub fn check_search_by_index<U: Number>(
        mut true_hits: Vec<(usize, U)>,
        mut pred_hits: Vec<(usize, U)>,
        name: &str,
    ) -> bool {
        true_hits.sort_by_key(|(i, _)| *i);
        pred_hits.sort_by_key(|(i, _)| *i);

        assert_eq!(
            true_hits.len(),
            pred_hits.len(),
            "{name}: {true_hits:?} vs {pred_hits:?}"
        );

        for ((i, p), (j, q)) in true_hits.into_iter().zip(pred_hits) {
            let msg = format!("Failed {name} i: {i}, j: {j}, p: {p}, q: {q}");
            assert_eq!(i, j, "{msg}");
            assert!(p.abs_diff(q) <= U::EPSILON, "{msg}");
        }

        true
    }

    pub fn check_search_by_distance<U: Number>(
        mut true_hits: Vec<(usize, U)>,
        mut pred_hits: Vec<(usize, U)>,
        name: &str,
    ) -> bool {
        true_hits.sort_by(|(_, p), (_, q)| p.partial_cmp(q).unwrap_or(core::cmp::Ordering::Greater));
        pred_hits.sort_by(|(_, p), (_, q)| p.partial_cmp(q).unwrap_or(core::cmp::Ordering::Greater));

        assert_eq!(
            true_hits.len(),
            pred_hits.len(),
            "{name}: {true_hits:?} vs {pred_hits:?}"
        );

        for (i, (&(_, p), &(_, q))) in true_hits.iter().zip(pred_hits.iter()).enumerate() {
            assert!(
                p.abs_diff(q) <= U::EPSILON,
                "Failed {name} i-th: {i}, p: {p}, q: {q} in {true_hits:?} vs {pred_hits:?}"
            );
        }

        true
    }

    pub fn gen_random_data<F: Float>(
        car: usize,
        dim: usize,
        max: F,
        seed: u64,
    ) -> Result<FlatVec<Vec<F>, F, usize>, String> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let data = symagen::random_data::random_tabular(car, dim, -max, max, &mut rng);
        let distance_fn = |a: &Vec<F>, b: &Vec<F>| distances::vectors::euclidean(a, b);
        let metric = Metric::new(distance_fn, false);
        FlatVec::new(data, metric)
    }

    pub fn check_search<I, U, D, C>(algs: &Algs<U>, data: &D, root: &C, query: &I, name: &str) -> bool
    where
        I: Send + Sync,
        U: Number,
        D: ParDataset<I, U> + ParLinearSearch<I, U>,
        C: ParCluster<I, U, D>,
    {
        for (alg, checker) in algs {
            let true_hits = alg.par_linear_search(data, query);
            let pred_hits = alg.par_search(data, root, query);
            let alg_name = format!("{name}-{}", alg.name());
            checker(true_hits.clone(), pred_hits, &alg_name);
        }

        true
    }

    #[test_case(1_000, 10)]
    #[test_case(10_000, 10)]
    #[test_case(1_000, 100)]
    #[test_case(10_000, 100)]
    fn vectors(car: usize, dim: usize) -> Result<(), String> {
        let mut algs: Algs<f32> = vec![];
        for radius in [0.1, 1.0] {
            algs.push((super::Algorithm::RnnClustered(radius), check_search_by_index));
        }
        for k in [1, 10, 100] {
            algs.push((super::Algorithm::KnnRepeatedRnn(k, 2.0), check_search_by_distance));
            algs.push((super::Algorithm::KnnBreadthFirst(k), check_search_by_distance));
            algs.push((super::Algorithm::KnnDepthFirst(k), check_search_by_distance));
        }

        let seed = 42;
        let data = gen_random_data(car, dim, 10.0, seed)?;
        let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
        let seed = Some(seed);
        let query = &vec![0.0; dim];

        let ball = Ball::new_tree(&data, &criteria, seed);
        check_search(&algs, &data, &ball, query, "ball");

        let (off_ball, perm_data) = OffBall::from_ball_tree(ball.clone(), data.clone());
        check_search(&algs, &perm_data, &off_ball, query, "off_ball");

        let (par_off_ball, per_perm_data) = OffBall::par_from_ball_tree(ball, data);
        check_search(&algs, &per_perm_data, &par_off_ball, query, "par_off_ball");

        Ok(())
    }

    #[test_case::test_case(16, 16, 3)]
    fn strings(num_clumps: usize, clump_size: usize, clump_radius: u16) -> Result<(), String> {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();

        pool.install(|| {
            let mut algs: Algs<u16> = vec![];
            for radius in [4, 8, 16] {
                algs.push((super::Algorithm::RnnClustered(radius), check_search_by_index));
            }
            for k in [1, 10, 20] {
                algs.push((super::Algorithm::KnnRepeatedRnn(k, 2), check_search_by_distance));
                algs.push((super::Algorithm::KnnBreadthFirst(k), check_search_by_distance));
                algs.push((super::Algorithm::KnnDepthFirst(k), check_search_by_distance));
            }

            let seed_length = 100;
            let alphabet = "ACTGN".chars().collect::<Vec<_>>();
            let seed_string = symagen::random_edits::generate_random_string(seed_length, &alphabet);
            let penalties = distances::strings::Penalties::default();
            let inter_clump_distance_range = (clump_radius * 5, clump_radius * 7);
            let len_delta = seed_length / 10;
            let (metadata, data) = symagen::random_edits::generate_clumped_data(
                &seed_string,
                penalties,
                &alphabet,
                num_clumps,
                clump_size,
                clump_radius,
                inter_clump_distance_range,
                len_delta,
            )
            .into_iter()
            .unzip::<_, _, Vec<_>, Vec<_>>();
            let query = &seed_string;

            let distance_fn = |a: &String, b: &String| distances::strings::levenshtein::<u16>(a, b);
            let metric = Metric::new(distance_fn, true);
            let data = FlatVec::new(data, metric)?.with_metadata(metadata)?;

            let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
            let seed = Some(42);

            let ball = Ball::new_tree(&data, &criteria, seed);
            check_search(&algs, &data, &ball, query, "ball");

            let (off_ball, perm_data) = OffBall::from_ball_tree(ball.clone(), data.clone());
            check_search(&algs, &perm_data, &off_ball, query, "off_ball");

            let (par_off_ball, par_perm_data) = OffBall::par_from_ball_tree(ball.clone(), data.clone());
            check_search(&algs, &par_perm_data, &par_off_ball, query, "par_off_ball");

            let (squishy_ball, co_data) = SquishyBall::from_ball_tree(ball.clone(), data.clone());
            check_search(&algs, &co_data, &squishy_ball, query, "squishy_ball");

            let (par_squishy_ball, par_co_data) = SquishyBall::par_from_ball_tree(ball, data);
            check_search(&algs, &par_co_data, &par_squishy_ball, query, "par_squishy_ball");

            Ok::<_, String>(())
        })?;

        Ok(())
    }
}
