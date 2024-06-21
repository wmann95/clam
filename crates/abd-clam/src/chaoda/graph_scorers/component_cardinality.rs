use std::hash::Hash;

use distances::Number;

use crate::graph::Graph;

use super::{ClusterScores, GraphScorer};

/// A graph scorer that calculates scores based on component cardinality.
pub struct ComponentCardinality;

impl Hash for ComponentCardinality {
    /// Generates a hash for the `ComponentCardinality` instance.
    ///
    /// This function hashes the string "`component_cardinality`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "component_cardinality".hash(state);
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for ComponentCardinality {
    /// Returns the name of the `ComponentCardinality` graph scorer.
    ///
    /// The name is "`component_cardinality`."
    fn name(&self) -> &str {
        "component_cardinality"
    }

    /// Returns the short name of the `ComponentCardinality` graph scorer.
    ///
    /// The short name is "sc."
    fn short_name(&self) -> &str {
        "sc"
    }

    /// Indicates whether normalization should be performed based on clusters for `ComponentCardinality`.
    fn normalize_on_clusters(&self) -> bool {
        true
    }

    /// Computes and returns cluster scores based on component cardinality.
    ///
    /// This function calculates the scores for clusters based on the cardinality of components they belong to
    /// and returns them as a map of cluster references to their respective scores as floating-point values.
    /// The scores are calculated based on the cardinality of the components that each cluster belongs to.
    ///
    /// # Arguments
    ///
    /// * `_graph`: A reference to the input graph from which cluster scores are calculated.
    ///
    /// # Returns
    ///
    /// A `ClusterScores` mapping clusters to their calculated scores based on the cardinality of their components.
    fn score_graph(&self, graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String> {
        let scores = graph
            .find_component_clusters()
            .iter()
            .flat_map(|clusters| {
                let score = -clusters.len().as_f64();
                clusters.iter().map(move |&c| (c, score))
            })
            .collect();
        Ok(scores)
    }
}
