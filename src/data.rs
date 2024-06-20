use std::any::Any;

/// The trait you need to implement to provide the algorithm a distance and merging strategy.
pub trait ClusterSummary {
    fn summary_size(&self) -> usize;

    /// Doesn't have to be a proper distance. It has to be symmetric, but it can:
    ///
    /// * be negative
    /// * not satisfy the triangle inequality
    fn distance(&self, other: &dyn ClusterSummary) -> f32;

    /// Merge another ClusterSummary into the one at hand.
    fn extend(&mut self, other: &dyn ClusterSummary);

    /// Clear the memory used by your structure
    fn clear(&mut self);

    /// Return itself. Used for dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

/// The trait you need to implement for the clustering algorithm to access your data.
pub trait IndexableData {
    fn get_value(&self, row_index: usize, column_index: usize) -> f32;
    fn get_num_columns(&self) -> usize;
    fn get_num_rows(&self) -> usize;
    fn create_cluster_summary(&self, row_index: usize) -> Box<dyn ClusterSummary>;
}
