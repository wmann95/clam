use std::hash::Hash;

use distances::Number;

use crate::graph::Graph;

use super::{ClusterScores, GraphScorer};

/// A graph scorer that calculates scores based on vertex degree.
pub struct VertexDegree;

impl Hash for VertexDegree {
    /// Generates a hash for the `VertexDegree` instance.
    ///
    /// This function hashes the string "`vertex_degree`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "vertex_degree".hash(state);
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for VertexDegree {
    /// Returns the name of the `VertexDegree` graph scorer.
    ///
    /// The name is "`vertex_degree`."
    fn name(&self) -> &str {
        "vertex_degree"
    }

    /// Returns the short name of the `VertexDegree` graph scorer.
    ///
    /// The short name is "vd."
    fn short_name(&self) -> &str {
        "vd"
    }

    /// Indicates whether normalization should be performed based on clusters for `VertexDegree`.
    fn normalize_on_clusters(&self) -> bool {
        true
    }

    /// Computes and returns cluster scores based on vertex degree.
    ///
    /// This function calculates the scores for clusters based on the vertex degrees of their vertices
    /// and returns them as a map of cluster references to their respective scores as floating-point values.
    /// The scores are calculated based on the vertex degrees of the clusters' vertices within the input graph.
    ///
    /// # Arguments
    ///
    /// * `_graph`: A reference to the input graph from which cluster scores are calculated.
    ///
    /// # Returns
    ///
    /// A `ClusterScores` mapping clusters to their calculated scores based on the vertex degrees of their vertices.
    #[allow(clippy::cast_precision_loss)]
    fn score_graph(&self, graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String> {
        let scores: Result<ClusterScores<'a, U>, String> = graph
            .ordered_clusters()
            .iter()
            .map(|&c| graph.vertex_degree(c).map(|degree| (c, -(degree as f64))))
            .collect();
        scores
    }
}
