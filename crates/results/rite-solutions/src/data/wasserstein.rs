//! 1D Wasserstein distance

use distances::{number::Float, Number};

/// Compute the Wasserstein distance between two 1D distributions.
///
/// Uses Euclidean distance as the ground metric.
///
/// See the [SciPy documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.wasserstein_distance.html) for more information.
pub fn wasserstein<T: Number, U: Float>(x: &Vec<T>, y: &Vec<T>) -> U {
    let mut work = U::ZERO;
    
    let mut left = x.iter().map(|f| *f).enumerate().collect::<Vec<(usize, T)>>();
    let mut right = y.iter().map(|f| *f).enumerate().collect::<Vec<(usize, T)>>();
    
    while let Some((l_index, mut l_val)) = left.pop(){
        while l_val.as_f64().is_normal() {
            let (r_index, mut r_val) = match right.pop(){
              Some(v) => v,
              None => break
            };
            
            let flow = if l_val <= r_val{
              let flow = l_val;
              l_val = T::ZERO;
              r_val -= flow;
              
              if r_val.as_f64().is_normal(){
                  right.push((r_index, r_val));
              }
              
              flow
            }
            else{
              let flow = r_val;
              l_val -= flow;
              
              flow
            };
            
            work += U::from(flow) * (U::from(l_index) - U::from(r_index)).abs();
        }
    }
                       
    work
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

