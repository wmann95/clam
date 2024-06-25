use std::hash::Hash;

use distances::Number;

use crate::{graph::Graph, utils::{mean, standard_deviation}, Cluster};

use super::{ClusterScores, InstanceScores};


/// A trait for scoring graphs.
pub trait GraphScorer<'a, U: Number> {
    /// Computes scores for the given graph and returns cluster scores and an array of scores.
    ///
    /// This function is responsible for calculating scores based on the input graph.
    ///
    /// # Arguments
    ///
    /// * `_graph`: A reference to the input graph from which scores are calculated.
    ///
    /// # Returns
    ///
    /// A tuple containing cluster scores and an array of scores.
    ///
    /// * `ClusterScores`: A structure that holds scores associated with individual clusters in the graph.
    /// * `Vec<f64>`: An array of scores, where each element corresponds to a specific aspect or metric.
    ///
    /// # Errors
    ///
    /// Throws an error if unable to compute scores for the given graph
    ///
    fn call(&self, graph: &'a Graph<'a, U>) -> Result<(ClusterScores<'a, U>, Vec<f64>), String> {
        let cluster_scores = {
            let scores = self.score_graph(graph)?;
            let mut cluster_scores: ClusterScores<'a, U> = scores;
            if self.normalize_on_clusters() {
                let (clusters, scores): (Vec<_>, Vec<_>) = cluster_scores.into_iter().unzip();
                cluster_scores = clusters
                    .into_iter()
                    .zip(crate::utils::normalize_1d(
                        &scores,
                        mean(&scores),
                        standard_deviation(&scores),
                    ))
                    .collect();
            }
            cluster_scores
        };

        let instance_scores = {
            let mut instance_scores = self.inherit_scores(&cluster_scores);
            if !self.normalize_on_clusters() {
                let (indices, scores): (Vec<_>, Vec<_>) = instance_scores.into_iter().unzip();
                instance_scores = indices
                    .into_iter()
                    .zip(crate::utils::normalize_1d(
                        &scores,
                        mean(&scores),
                        standard_deviation(&scores),
                    ))
                    .collect();
            }
            instance_scores
        };

        let scores_array = self.ordered_scores(&instance_scores);

        Ok((cluster_scores, scores_array))
    }

    /// Returns the name of the graph scorer.
    fn name(&self) -> &str;

    /// Returns the short name of the graph scorer.
    fn short_name(&self) -> &str;

    /// Indicates whether normalization should be performed based on clusters.
    fn normalize_on_clusters(&self) -> bool;

    /// Computes and returns cluster scores for clusters in the input graph.
    ///
    /// This function calculates cluster scores based on the characteristics of clusters in the input graph.
    /// Cluster scores are represented as a `ClusterScores` mapping clusters to their associated scores.
    ///
    /// # Arguments
    ///
    /// * `graph`: The input graph for which cluster scores are computed.
    ///
    /// # Returns
    ///
    /// A `ClusterScores` mapping clusters in the input graph to their associated scores.
    ///
    /// # Errors
    ///
    /// Throws an error if unable to compute a score for a cluster within a given graph.
    ///
    fn score_graph(&self, graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String>;

    /// Inherits cluster scores and computes scores for individual instances.
    ///
    /// This function inherits cluster scores and uses them to calculate scores for individual instances.
    /// It takes a `ClusterScores` as input, which maps clusters to their associated scores,
    /// and returns an `InstanceScores` mapping instances to their computed scores.
    ///
    /// # Arguments
    ///
    /// * `scores`: A `ClusterScores` mapping clusters to their associated scores.
    ///
    /// # Returns
    ///
    /// An `InstanceScores` mapping instances to their computed scores.
    fn inherit_scores(&self, scores: &ClusterScores<U>) -> InstanceScores {
        scores
            .iter()
            .flat_map(|(&c, &s)| c.indices().map(move |i| (i, s)))
            .collect()
    }

    /// Orders the scores for individual instances.
    ///
    /// This function takes an `InstanceScores` mapping instances to their associated scores
    /// and returns a sorted vector of scores for instances. The vector contains the scores
    /// in ascending order based on the instance indices.
    ///
    /// # Arguments
    ///
    /// * `scores`: An `InstanceScores` mapping instances to their associated scores.
    ///
    /// # Returns
    ///
    /// A sorted vector of scores for instances in ascending order.
    fn ordered_scores(&self, scores: &InstanceScores) -> Vec<f64> {
        let mut scores: Vec<_> = scores.iter().map(|(&i, &s)| (i, s)).collect();
        scores.sort_by_key(|(i, _)| *i);
        let (_, scores): (Vec<_>, Vec<f64>) = scores.into_iter().unzip();
        scores
    }
}
