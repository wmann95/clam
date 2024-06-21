use std::hash::Hash;

use distances::Number;

use crate::graph::Graph;

use super::{ClusterScores, GraphScorer};

/// A graph scorer that calculates stationary probabilities of clusters after a specified number of steps.
pub struct StationaryProbabilities {
    /// Number of steps for stationary probability calculation.
    #[allow(dead_code)]
    num_steps: usize,
}

impl Hash for StationaryProbabilities {
    /// Generates a hash for the `StationaryProbabilities` instance.
    ///
    /// This function hashes the string "`stationary_probabilities`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "stationary_probabilities".hash(state);
    }
}

impl StationaryProbabilities {
    /// Creates a new instance of the `StationaryProbabilities` scorer.
    ///
    /// The `num_steps` parameter specifies the number of steps for which stationary probabilities will be computed.
    ///
    #[must_use]
    pub const fn new(num_steps: usize) -> Self {
        Self { num_steps }
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for StationaryProbabilities {
    /// Returns the name of the `StationaryProbabilities` graph scorer.
    ///
    /// The name is "`stationary_probabilities`."
    fn name(&self) -> &str {
        "stationary_probabilities"
    }

    /// Returns the short name of the `StationaryProbabilities` graph scorer.
    ///
    /// The short name is "sp."
    fn short_name(&self) -> &str {
        "sp"
    }

    /// Indicates whether normalization should be performed based on clusters for `StationaryProbabilities`.
    ///
    /// TODO!
    fn normalize_on_clusters(&self) -> bool {
        todo!()
        //true
    }

    #[allow(unused_variables)]

    /// Computes and returns cluster scores based on stationary probabilities after a specified number of steps.
    ///
    /// This function calculates the scores for clusters based on their stationary probabilities in the graph
    /// after a fixed number of steps. The `num_steps` parameter specified during initialization determines
    /// the number of steps.
    ///
    /// # Arguments
    ///
    /// * `graph`: The input graph for which cluster scores are to be computed.
    ///
    /// # Returns
    ///
    /// A map of cluster indices to their respective scores as floating-point values.
    fn score_graph(&self, graph: &'a Graph<U>) -> Result<ClusterScores<'a, U>, String> {
        todo!()
    }
}
