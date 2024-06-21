
use distances::Number;

use crate::PartitionCriteria;

struct Space {}

/// TODO: Add documentation for this struct.
pub struct CHAODA<U: Number> {
    metric_spaces: Vec<Space>,
    partition_criteria: Option<PartitionCriteria<U>>,
    //selector_scorers: Option<HashMap<(Box<dyn PartitionCriterion<U>>, )>>
}

impl<U: Number> CHAODA<U>{
    pub fn new() -> Self {
        todo!();
    }
}