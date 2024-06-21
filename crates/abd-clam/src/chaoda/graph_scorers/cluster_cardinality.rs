use std::hash::Hash;

use distances::Number;

use crate::{graph::Graph, Cluster};

use super::{ClusterScores, GraphScorer};

/// A graph scorer that calculates scores based on cluster cardinality.
pub struct ClusterCardinality;

impl Hash for ClusterCardinality {
    /// Generates a hash for the `ClusterCardinality` instance.
    ///
    /// This function hashes the string "`cluster_cardinality`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "cluster_cardinality".hash(state);
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for ClusterCardinality {
    /// Returns the name of the `ClusterCardinality` graph scorer.
    ///
    /// The name is "`cluster_cardinality`."
    fn name(&self) -> &str {
        "cluster_cardinality"
    }

    /// Returns the short name of the `ClusterCardinality` graph scorer.
    ///
    /// The short name is "cc."
    fn short_name(&self) -> &str {
        "cc"
    }

    /// Indicates whether normalization should be performed based on clusters for `ClusterCardinality`.
    fn normalize_on_clusters(&self) -> bool {
        true
    }

    /// Computes and returns cluster scores based on cluster cardinality.
    ///
    /// This function calculates the scores for clusters based on their cardinality and returns them
    /// as a map of cluster references to their respective scores as floating-point values. The scores
    /// are calculated based on the cluster's cardinality, which represents the number of elements
    /// in the cluster.
    ///
    /// # Arguments
    ///
    /// * `_graph`: A reference to the input graph from which cluster scores are calculated.
    ///
    /// # Returns
    ///
    /// A `ClusterScores` mapping clusters to their calculated scores based on their cardinality.
    fn score_graph(&self, graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String> {
        let scores = graph
            .ordered_clusters()
            .iter()
            .map(|&c| (c, -c.cardinality().as_f64()))
            .collect();
        Ok(scores)
    }
}
