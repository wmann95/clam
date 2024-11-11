//! 1D Wasserstein distance

use distances::{number::Float, Number};

/// Compute the Wasserstein distance between two 1D distributions.
///
/// Uses Euclidean distance as the ground metric.
///
/// See the [SciPy documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.wasserstein_distance.html) for more information.
pub fn wasserstein<T: Number, U: Float>(u_values: &Vec<T>, v_values: &Vec<T>) -> U {
    // We assume that both vecs are sorted.
    
    let u_values = u_values.iter().map(|f| U::from(*f)).collect::<Vec<U>>();
    let v_values = v_values.iter().map(|f| U::from(*f)).collect::<Vec<U>>();
    
    let vec_length = u_values.len();
    
    // In the original implementation, the vectors are not expected to be sorted.
    // Instead, the values are taken in as they are, sorted, and their indices of
    // where they would be in the concatenated, sorted list of all values.
    // These are already sorted, so their indices will always just be a vec of 0 to the len,
    // each divided by the len of the array to get the cdf of the index. Each will have the
    // same result, as they are both sorted, and are both of the same length.
    let cdf_indices = (0..vec_length)
        .map(|i| U::from(i) / U::from(vec_length))
        .collect::<Vec<U>>();
    
    
    u_values.iter()
        .zip(v_values.iter())
        // find the absolute difference
        .map(|(&l, &r)| l.abs_diff(r))
        .zip(cdf_indices.iter())
        // multiply against the cdf
        .map(|(diff, &cdf)| diff * cdf)
        // find the sum
        .sum::<U>()
}

pub fn direct_flow<T: Number, U: Float>(x: &Vec<T>, y: &Vec<T>) -> U {
    let mut work = U::ZERO;
    
    let mut zipped = x.iter().zip(y.iter()).collect::<Vec<(&T, &T)>>();
    
    while let Some((&l_val, &r_val)) = zipped.pop(){
        
        let flow = U::from(l_val - r_val).abs();
        
        work += U::from(flow);
    }
    
    work
}

#[cfg(test)]
mod wasserstein_tests{
    use rand::{thread_rng, Rng};

    use crate::data::wasserstein::wasserstein;

    const K: usize = 100000;

    #[test]
    fn wasserstein_test(){
        let mut dirt: Vec<f32> = vec![0.; K];
        let mut holes: Vec<f32> = vec![0.; K];

        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        holes = holes.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let t = std::time::Instant::now();

        dirt.sort_by(|a, b| a.partial_cmp(b).unwrap());
        holes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let res: f32 = wasserstein(&dirt, &holes);

        let time = t.elapsed().as_secs_f64();

        println!("Time: {}", time);

        println!("{}", res);
        }

        #[test]
        fn deep_tests(){
        for _ in 0..100{
            identity_test();
            symmetry_test();
            triangle_inequality_test();
        }
    }

    #[test]
    fn identity_test(){
        let mut dirt = vec![0.; K];
        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let res: f32 = wasserstein(&dirt, &dirt);

        assert_eq!(res, 0.);
    }

    #[test]
    fn symmetry_test(){
        let mut dirt = vec![0.; K];
        let mut holes = vec![0.; K];

        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        holes = holes.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let res1: f32 = wasserstein(&dirt, &holes);
        let res2: f32 = wasserstein(&holes, &dirt);

        assert_eq!(res1, res2);
    }

    #[test]
    fn triangle_inequality_test(){
            
        let mut v1 = vec![0.; K];
        let mut v2 = vec![0.; K];
        let mut v3 = vec![0.; K];

        v1 = v1.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        v2 = v2.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        v3 = v3.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let d_v1_v3: f32 = wasserstein(&v1, &v3);
        let d_v1_v2: f32 = wasserstein(&v1, &v2);
        let d_v2_v3: f32 = wasserstein(&v2, &v3);

        assert!(d_v1_v3 <= d_v1_v2 + d_v2_v3);
    }
}

#[cfg(test)]
mod direct_flow_tests{
    use rand::{thread_rng, Rng};

    use crate::data::wasserstein::direct_flow;

    const K: usize = 10;

    #[test]
    fn direct_flow_test(){
        let mut dirt: Vec<f32> = vec![0.; K];
        let mut holes: Vec<f32> = vec![0.; K];

        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        holes = holes.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let t = std::time::Instant::now();

        let res: f32 = direct_flow(&dirt, &holes);

        let time = t.elapsed().as_secs_f64();

        println!("Time: {}", time);

        println!("{}", res);
    }

    #[test]
    fn deep_tests(){
        for _ in 0..100{
            identity_test();
            symmetry_test();
            triangle_inequality_test();
        }
    }

    #[test]
    fn identity_test(){
        let mut dirt = vec![0.; K];
        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let res: f32 = direct_flow(&dirt, &dirt);

        assert_eq!(res, 0.);
    }

    #[test]
    fn symmetry_test(){
        let mut dirt = vec![0.; K];
        let mut holes = vec![0.; K];

        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        holes = holes.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let res1: f32 = direct_flow(&dirt, &holes);
        let res2: f32 = direct_flow(&holes, &dirt);

        assert_eq!(res1, res2);
    }

    #[test]
    fn triangle_inequality_test(){
            
        let mut v1 = vec![0.; K];
        let mut v2 = vec![0.; K];
        let mut v3 = vec![0.; K];

        v1 = v1.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        v2 = v2.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        v3 = v3.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let d_v1_v3: f32 = direct_flow(&v1, &v3);
        let d_v1_v2: f32 = direct_flow(&v1, &v2);
        let d_v2_v3: f32 = direct_flow(&v2, &v3);

        assert!(d_v1_v3 <= d_v1_v2 + d_v2_v3);
    }
}


#[cfg(test)]
mod direct_flow_vs_wasserstein_tests{
    use rand::{thread_rng, Rng};

    use crate::data::wasserstein::{direct_flow, wasserstein};

    const K: usize = 1000;

    #[test]
    fn direct_flow_vs_wasserstein_test(){
        let mut dirt: Vec<f32> = vec![0.; K];
        let mut holes: Vec<f32> = vec![0.; K];

        dirt = dirt.iter().map(|_| thread_rng().r#gen::<f32>()).collect();
        holes = holes.iter().map(|_| thread_rng().r#gen::<f32>()).collect();

        let t = std::time::Instant::now();

        let direct_flow_result: f32 = direct_flow(&dirt, &holes);

        let df_time = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        
        let wasserstein_result: f32 = wasserstein(&dirt, &holes);
        
        let w_time = t.elapsed().as_secs_f64();

        println!("Direct-Flow Time: {}", df_time);
        println!("Wasserstein Time: {}", w_time);

        println!("Direct-Flow distance: {}", direct_flow_result);
        println!("Wasserstein distance: {}", wasserstein_result);
    }
    
    // #[test]
    // fn deep_tests(){
    //     for _ in 0..100{
    //         identity_test();
    //         symmetry_test();
    //         triangle_inequality_test();
    //     }
    // }
}

