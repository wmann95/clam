//! 1D Wasserstein distance

use distances::{number::{Addition, Float}, Number};

/// Compute the Wasserstein distance between two 1D distributions.
///
/// Uses Euclidean distance as the ground metric.
///
/// See the [SciPy documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.wasserstein_distance.html) for more information.
pub fn wasserstein<T: Number, U: Float>(x: &Vec<T>, y: &Vec<T>) -> U {
    let mut work = U::ZERO;
    
    //IDEA: iterate through one side, subtract value from first POSITIVE value of other
    
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
              
              //println!("l<=r {{{}}}", flow);
              
              flow
            }
            else{
              let flow = r_val;
              l_val -= flow;
              
              //println!("l>r {{{}}}", flow);
              
              flow
            };
            
            work += U::from(flow) * (U::from(l_index) - U::from(r_index)).abs();
            // println!("Current work: {}", work);
        }
    }
    
    // x.iter().zip(y.iter()).fold(0f32, |acc, (x, y)| {
    //     let flow = max(x, y);
    // });
                                
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



/*
using System;
namespace Wasserstein
{
  class Program
  {
    static void Main(string[] args)
    {
      Console.WriteLine("\nBegin demo \n");

      double[] P = new double[]
        { 0.6, 0.1, 0.1, 0.1, 0.1 };
      double[] Q1 = new double[]
        { 0.1, 0.1, 0.6, 0.1, 0.1 };
      double[] Q2 = new double[]
        { 0.1, 0.1, 0.1, 0.1, 0.6 };

      double wass_p_q1 = MyWasserstein(P, Q1);
      double wass_p_q2 = MyWasserstein(P, Q2);

      Console.WriteLine("Wasserstein(P, Q1) = " +
        wass_p_q1.ToString("F4"));
      Console.WriteLine("Wasserstein(P, Q2) = " +
        wass_p_q2.ToString("F4"));

      Console.WriteLine("\nEnd demo ");
      Console.ReadLine();
    }  // Main

    static int FirstNonZero(double[] vec)
    {
      int dim = vec.Length;
      for (int i = 0; i < dim; ++i)
        if (vec[i] > 0.0)
          return i;
      return -1;
    }

    static double MoveDirt(double[] dirt, int di,
      double[] holes, int hi)
    {
      double flow = 0.0;
      int dist = 0;
      if (dirt[di] <= holes[hi])
      {
        flow = dirt[di];
        dirt[di] = 0.0;
        holes[hi] -= flow;
      }
      else if (dirt[di] > holes[hi])
      {
        flow = holes[hi];
        dirt[di] -= flow;
        holes[hi] = 0.0;
      }
      dist = Math.Abs(di - hi);
      return flow * dist;
    }

    static double MyWasserstein(double[] p, double[] q)
    {
      double[] dirt = (double[])p.Clone();
      double[] holes = (double[])q.Clone();
      double totalWork = 0.0;
      while (true)
      {
        int fromIdx = FirstNonZero(dirt);
        int toIdx = FirstNonZero(holes);
        if (fromIdx == -1 || toIdx == -1)
          break;
        double work = MoveDirt(dirt, fromIdx,
          holes, toIdx);
        totalWork += work;
      }
      return totalWork;
    }
  }  // Program
}  // ns
*/