use std::hash::Hash;

use distances::Number;

use crate::{graph::{Graph, Vertex}, Cluster};

use super::{ClusterScores, GraphScorer};


/// A graph scorer that calculates scores based on parent-child cluster relationships and cardinality.
///
/// This scorer assigns scores to clusters based on the cardinality of a cluster relative to its parent
/// cluster in a hierarchical structure. It uses a user-defined weight function to assign weights to
/// different levels of the hierarchy.
pub struct ParentCardinality<'a, U: Number> {
    /// The root cluster in the hierarchical structure.
    #[allow(dead_code)]
    root: &'a Vertex<U>,
    /// User-defined weight function for hierarchy levels.
    #[allow(dead_code)]
    weight: Box<dyn (Fn(usize) -> f64) + Send + Sync>,
}

impl<'a, U: Number> ParentCardinality<'a, U> {
    /// Creates a new instance of the `ParentCardinality` scorer.
    ///
    /// The `root` parameter specifies the root cluster of the hierarchy. The weight function is used
    /// to assign weights to different levels of the hierarchy.
    ///
    /// # Arguments
    ///
    /// * `_root`: The root cluster of the hierarchical structure.
    ///
    /// # Returns
    ///
    /// A new instance of the `ParentCardinality` scorer with the specified root cluster and weight function.
    pub fn new(root: &'a Vertex<U>) -> Self {
        let weight = Box::new(|d: usize| 1. / (d.as_f64()).sqrt());
        Self { root, weight }
    }

    /// Computes the ancestry of a given cluster.
    ///
    /// The ancestry of a cluster is a list of clusters starting from the root and ending at the given
    /// cluster. The list represents the hierarchical relationship between clusters in the structure.
    ///
    /// # Arguments
    ///
    /// * `_c`: The cluster for which the ancestry is to be computed.
    ///
    /// # Returns
    ///
    /// A vector of references to clusters representing the ancestry of the specified cluster.
    ///
    /// This method computes the ancestry of a cluster by traversing the hierarchical structure, starting from
    /// the root cluster and following parent-child relationships until the given cluster is reached.
    /// The resulting vector contains references to clusters that form the ancestry of the specified cluster.
    pub fn ancestry(&self, _c: &'a Vertex<U>) -> Vec<&'a Vertex<U>> {
        todo!()

        // c.history().into_iter().fold(vec![self.root], |mut ancestors, turn| {
        //     let last = ancestors.last().unwrap();
        //     let [left, right] = last.children().unwrap();
        //     let child = if *turn { right } else { left };
        //     ancestors.push(child);
        //     ancestors
        // })
    }
}

impl<'a, U: Number> Hash for ParentCardinality<'a, U> {
    /// Generates a hash for the `ParentCardinality` instance.
    ///
    /// This function hashes the string "`parent_cardinality`" to uniquely identify this scorer.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "parent_cardinality".hash(state);
    }
}

impl<'a, U: Number> GraphScorer<'a, U> for ParentCardinality<'a, U> {
    /// Returns the name of the `ParentCardinality` graph scorer.
    ///
    /// The name is "`parent_cardinality`."
    fn name(&self) -> &str {
        "parent_cardinality"
    }

    /// Returns the short name of the `ParentCardinality` graph scorer.
    ///
    /// The short name is "pc."
    fn short_name(&self) -> &str {
        "pc"
    }

    /// Indicates whether normalization should be performed based on clusters for `ParentCardinality`.
    ///
    /// TODO!
    fn normalize_on_clusters(&self) -> bool {
        todo!()
        //true
    }

    /// Computes and returns cluster scores based on parent-child cluster relationships and cardinality.
    ///
    /// This function calculates the scores for clusters based on their hierarchical relationships and
    /// cardinality, using the weight function to assign weights to different hierarchy levels.
    ///
    /// # Arguments
    ///
    /// * `_graph`: The input graph for which cluster scores are to be computed.
    ///
    /// # Returns
    ///
    /// A map of cluster indices to their respective scores as floating-point values.
    fn score_graph(&self, graph: &'a Graph<'a, U>) -> Result<ClusterScores<'a, U>, String> {
        let scores = graph
            .ordered_clusters()
            .iter()
            .map(|&c| {
                let ancestry = self.ancestry(c);
                let score: f64 = ancestry
                    .iter()
                    .skip(1)
                    .zip(ancestry.iter())
                    .enumerate()
                    .map(|(i, (child, parent))| {
                        (self.weight)(i + 1) * parent.cardinality().as_f64() / child.cardinality().as_f64()
                    })
                    .sum();
                (c, -score)
            })
            .collect();
        Ok(scores)
    }
}
