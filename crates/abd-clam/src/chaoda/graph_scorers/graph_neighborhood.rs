use std::hash::Hash;

use distances::Number;

use crate::graph::{Graph, Vertex};

use super::{ClusterScores, GraphScorer};

/// A graph scorer that calculates scores based on the neighborhood of clusters in a graph.
pub struct GraphNeighborhood {
    /// Fraction used to determine neighborhood size.
    #[allow(dead_code)]
    eccentricity_fraction: f64,
}

impl GraphNeighborhood {
    /// Creates a new instance of the `GraphNeighborhood` scorer.
    ///
    /// The `eccentricity_fraction` parameter specifies a factor that influences the number of steps taken
    /// in the neighborhood computation.
    ///
    #[must_use]
    pub const fn new(eccentricity_fraction: f64) -> Self {
        Self { eccentricity_fraction }
    }

    /// Calculates the number of steps for neighborhood computation in the graph.
    ///
    /// This function computes the number of steps based on the eccentricity of the given cluster
    /// and the eccentricity fraction specified during initialization.
    ///
    /// # Arguments
    ///
    /// * `_graph`: The input graph for which the number of steps is calculated.
    /// * `_c`: The cluster for which the number of steps is determined.
    ///
    /// # Returns
    ///
    /// The number of steps for neighborhood computation as a `usize` value.
    #[allow(dead_code)]
    fn num_steps<'a, U: Number>(&self, _graph: &'a Graph<'a, U>, _c: &'a Vertex<U>) -> usize {
        todo!()

        // let steps = graph.unchecked_eccentricity(c) as f64 * self.eccentricity_fraction;
        // 1 + steps as usize
    }
}

impl Hash for GraphNeighborhood {
    /// Generates a hash for the `GraphNeighborhood` instance.
    ///
    /// This function hashes the string "`graph_neighborhood`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "graph_neighborhood".hash(state);
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for GraphNeighborhood {
    /// Returns the name of the `GraphNeighborhood` graph scorer.
    ///
    /// The name is "`graph_neighborhood`."
    fn name(&self) -> &str {
        "graph_neighborhood"
    }

    /// Returns the short name of the `GraphNeighborhood` graph scorer.
    ///
    /// The short name is "gn."
    fn short_name(&self) -> &str {
        "gn"
    }

    /// Indicates whether normalization should be performed based on clusters for `GraphNeighborhood`.
    ///
    /// TODO!
    fn normalize_on_clusters(&self) -> bool {
        todo!()
        //true
    }

    /// Computes and returns cluster scores based on the neighborhood of clusters in the graph.
    ///
    /// This function calculates the scores for clusters based on their neighborhood in the graph,
    /// considering the number of steps and the size of the clusters within the neighborhood.
    ///
    /// # Arguments
    ///
    /// * `_graph`: The input graph for which cluster scores are to be computed.
    ///
    /// # Returns
    ///
    /// A map of cluster indices to their respective scores as floating-point values.
    fn score_graph(&self, _graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String> {
        todo!()

        // graph
        //     .ordered_clusters()
        //     .iter()
        //     .map(|&c| {
        //         let steps = self.num_steps(graph, c);
        //         // TODO: Do we need +1?
        //         let score = (0..steps + 1)
        //             .zip(graph.unchecked_frontier_sizes(c).iter())
        //             .fold(0, |score, (_, &size)| score + size);
        //         (c, -(score as f64))
        //     })
        //     .collect()
    }
}
