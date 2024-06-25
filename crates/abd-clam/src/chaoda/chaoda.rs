
use distances::Number;

use crate::{core::space::Space, Cluster, PartitionCriteria, PartitionCriterion};

use super::graph_scorers::GraphScorer;

pub enum NormalizationMode{
    LINEAR,
    GAUSSIAN,
    SIGMOID
}

pub enum VotingMode{
    Mean,
    Product,
    Median,
    Min,
    Max,
    P25,
    P75
}

/// The main struct representing CHAODA.
pub struct CHAODA<'a, U: Number> {
    metric_spaces: Vec<Space>,
    partition_criteria: Option<PartitionCriteria<U>>,
    selector_scorers: Option<Vec<(Box<dyn PartitionCriterion<U>>, Vec<Box<dyn GraphScorer<'a, U>>>)>>,
    normalization_mode: Option<NormalizationMode>,
    use_speed_threshold: Option<bool>,
    voting_mode: Option<VotingMode>
}

impl<'a, U: Number> CHAODA<'a, U>{
    /// Creates and initializes a CHAODA object.
    ///
    ///     Args:
    ///         metric_spaces: A list of metric spaces to use for anomaly detection.
    ///             All metric spaces should have the same `Dataset` object.
    ///         partition_criteria: list of criteria for partitioning clusters when
    ///             building trees.
    ///         selector_scorers: list of 2-tuples whose items are
    ///             - a trained meta-ml model for selecting a graph.
    ///             - a list of individual algorithms to run on that graph.
    ///         normalization_mode: What normalization mode to use. Must be one of
    ///             - 'linear',
    ///             - 'gaussian', or
    ///             - 'sigmoid'.
    ///         use_speed_threshold: Whether to skip slow graph scorers.
    ///         voting_mode: to use to aggregate scores for the ensemble.
    pub fn new(
        metric_spaces: Vec<Space>,
        partition_criteria: Option<PartitionCriteria<U>>,
        selector_scorers: Option<Vec<(Box<dyn PartitionCriterion<U>>, Vec<Box<dyn GraphScorer<'a, U>>>)>>,
        normalization_mode: Option<NormalizationMode>,
        use_speed_threshold: Option<bool>,
        voting_mode: Option<VotingMode>
    ) -> Self {
        Self{
            metric_spaces,
            partition_criteria,
            selector_scorers,
            normalization_mode,
            use_speed_threshold,
            voting_mode
        }
    }
    
    pub fn build(&self){
        todo!();
    }
    
    fn vote(&self){
        todo!();
    }
    
    pub fn scores(&self){
        todo!();
    }
    
    pub fn fit_predict(&self){
        todo!();
    }
    
    pub fn predict_single(&self){
        todo!();
    }
    
    pub fn predict(&self){
        todo!();
    }
}

struct SingleSpaceChaoda{
    
}

impl SingleSpaceChaoda {
    pub fn new() -> Self {
        todo!();
    }
    
    pub fn root(&self) {
        todo!();
    }
    
    pub fn scores(&self){
        todo!();
    }
    
    pub fn cluster_scores_list(&self){
        todo!();
    }
    
    pub fn searcher(&self){
        todo!();
    }
    
    pub fn build(&self){
        todo!();
    }
}

