//! Different ways to score clustered graphs based on their properties.
mod parent_cardinality;
mod cluster_cardinality;
mod component_cardinality;
mod graph_neighborhood;
mod graph_scorer;
mod stationary_probabilities;
mod vertex_degree;

pub use graph_scorer::GraphScorer;
pub use parent_cardinality::ParentCardinality;
pub use cluster_cardinality::ClusterCardinality;

use std::collections::HashMap;
use super::Vertex;

/// Type alias for cluster scores associated with clusters in a graph.
pub type ClusterScores<'a, U> = HashMap<&'a Vertex<U>, f64>;
/// Type alias for scores associated with individual instances or elements.
pub type InstanceScores = HashMap<usize, f64>;

