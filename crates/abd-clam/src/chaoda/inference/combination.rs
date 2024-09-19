//! Utilities for handling a pair of `MetaMLModel` and `GraphAlgorithm`.

use distances::Number;
use serde::{Deserialize, Serialize};

use crate::{
    chaoda::{training::GraphEvaluator, Graph, GraphAlgorithm, Vertex},
    Cluster, Dataset,
};

use super::TrainedMetaMlModel;

/// A combination of `TrainedMetaMLModel` and `GraphAlgorithm`.
#[derive(Clone, Serialize, Deserialize)]
pub struct TrainedCombination {
    /// The `MetaMLModel` to use.
    meta_ml: TrainedMetaMlModel,
    /// The `GraphAlgorithm` to use.
    graph_algorithm: GraphAlgorithm,
}

impl TrainedCombination {
    /// Get the name of the combination.
    ///
    /// The name is in the format `{meta_ml.short_name()}-{graph_algorithm.name()}`.
    pub fn name(&self) -> String {
        format!("{}-{}", self.meta_ml.short_name(), self.graph_algorithm.name())
    }

    /// Create a new `TrainedCombination`.
    pub fn new(meta_ml: TrainedMetaMlModel, graph_algorithm: GraphAlgorithm) -> Self {
        Self {
            meta_ml,
            graph_algorithm,
        }
    }

    /// Get the meta-ML scorer function in a callable for any number of `Vertex`es.
    pub fn meta_ml_scorer<I, U, D, S>(&self) -> impl Fn(&[&Vertex<I, U, D, S>]) -> Vec<f32> + '_
    where
        U: Number,
        D: Dataset<I, U>,
        S: Cluster<I, U, D>,
    {
        move |clusters| {
            let props = clusters.iter().map(|c| c.ratios().to_vec()).collect::<Vec<_>>();
            self.meta_ml.predict(&props).unwrap()
        }
    }

    /// Create a `Graph` from the `root` with the given `data` and `min_depth`
    /// using the `TrainedMetaMLModel`.
    pub fn create_graph<'a, I, U, D, S>(
        &self,
        root: &'a Vertex<I, U, D, S>,
        data: &D,
        min_depth: usize,
    ) -> Graph<'a, I, U, D, S>
    where
        U: Number,
        D: Dataset<I, U>,
        S: Cluster<I, U, D>,
    {
        let cluster_scorer = self.meta_ml_scorer();
        Graph::from_root(root, data, cluster_scorer, min_depth)
    }

    /// Predict the anomaly scores of the points in the `data`.
    ///
    /// # Arguments
    ///
    /// * `root`: A root `Vertex` of the tree.
    /// * `data`: The `Dataset` to predict on.
    /// * `min_depth`: The minimum depth at which to consider a `Cluster` for `Graph` construction.
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// * The `Graph` constructed from the `root`.
    /// * The anomaly scores of the points in the `data`.
    pub fn predict<'a, I, U, D, S>(
        &self,
        root: &'a Vertex<I, U, D, S>,
        data: &D,
        min_depth: usize,
    ) -> (Graph<'a, I, U, D, S>, Vec<f32>)
    where
        U: Number,
        D: Dataset<I, U>,
        S: Cluster<I, U, D>,
    {
        let mut graph = self.create_graph(root, data, min_depth);
        let scores = self.graph_algorithm.evaluate_points(&mut graph);
        (graph, scores)
    }
}
