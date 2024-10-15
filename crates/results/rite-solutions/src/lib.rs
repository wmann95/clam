mod data;
mod utils;

#[test]
fn test_neighborhood_aware() {
    use abd_clam::{Ball, Cluster, Partition};
    use data::NeighborhoodAware;
    
    let k = 10;
    
    let n = 10000;
    let dim = 10;
    let inlier_mean = 1000.;
    let outlier_mean = 1200.;
    let inlier_std = 200.;
    let outlier_std = 300.;
    
    let data = data::read_or_generate(
        None,
        &data::VecMetric::DirectFlow,
        Some(n),
        Some(dim),
        Some(inlier_mean),
        Some(inlier_std),
        None,
    ).unwrap();
    
    let criteria = |c: &Ball<_, _, _>| c.cardinality() > 1;
    let root = Ball::new_tree(&data, &criteria, None);

    let data = NeighborhoodAware::new(&data, &root, k);
    
    let root = root.with_dataset_type::<NeighborhoodAware>();

    let test_cardinality = 10;
    let outliers = data::gen_random(outlier_mean, outlier_std, test_cardinality, dim, None);
    let inliers = data::gen_random(inlier_mean, inlier_std, test_cardinality, dim, None);

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
