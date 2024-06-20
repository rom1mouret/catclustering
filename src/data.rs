use std::any::Any;

/// The trait you need to implement to provide the algorithm a distance and merging strategy.
pub trait ClusterSummary {
    /// The size of the summary structure (not of the cluster!).
    /// This increases when you add new categories to the cluster, and never decreases.
    fn summary_size(&self) -> usize;

    /// Distance between clusters.
    /// Not a proper distance. Check README to make sure your distance satisfies the algorithm's conditions.
    fn distance(&self, other: &dyn ClusterSummary) -> f32;

    /// Merge another ClusterSummary into the one at hand.
    fn extend(&mut self, other: &dyn ClusterSummary);

    /// Clear the memory used by your structure.
    fn clear(&mut self);

    /// Return itself. Used for dynamic dispatch.
    fn as_any(&self) -> &dyn Any;
}

/// The trait you need to implement for the clustering algorithm to access your data.
pub trait IndexableData {
    fn get_value(&self, row_index: usize, column_index: usize) -> f32;
    fn get_num_columns(&self) -> usize;
    fn get_num_rows(&self) -> usize;
    fn create_cluster_summary(&self, row_index: usize) -> Box<dyn ClusterSummary>;
}
