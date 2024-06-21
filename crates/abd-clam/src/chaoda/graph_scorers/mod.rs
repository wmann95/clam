//! Different ways to score clustered graphs based on their properties.
mod parent_cardinality;
mod cluster_cardinality;
mod component_cardinality;
mod graph_neighborhood;
mod graph_scorer;
mod stationary_probabilities;
mod vertex_degree;

pub use parent_cardinality::ParentCardinality;
pub use cluster_cardinality::ClusterCardinality;
pub use component_cardinality::ComponentCardinality;
pub use graph_neighborhood::GraphNeighborhood;
pub use graph_scorer::GraphScorer;
pub use stationary_probabilities::StationaryProbabilities;
pub use vertex_degree::VertexDegree;

use std::collections::HashMap;
use super::Vertex;

/// Type alias for cluster scores associated with clusters in a graph.
pub type ClusterScores<'a, U> = HashMap<&'a Vertex<U>, f64>;
/// Type alias for scores associated with individual instances or elements.
pub type InstanceScores = HashMap<usize, f64>;

