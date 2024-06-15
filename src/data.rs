use std::any::Any;

/// The trait you need to implement for the algorithm to peform merge and distance calculation between clusters.
pub trait CategoryMatrix {
    fn num_categories(&self) -> u16;

    /// Doesn't have to be a proper distance. It has to be symmetric, but it can:
    ///
    /// * be negative
    /// * not satisfy the triangle inequality
    fn distance(&self, other: &dyn CategoryMatrix) -> i16;

    /// Merge another CategoryMatrix into the one at hand.
    fn extend(&mut self, other: &dyn CategoryMatrix);

    /// Clear the memory used by your structure
    fn clear(&mut self);

    /// Return itself. Used for dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

/// The trait you need to implement for the clustering algorithm to access your data.
pub trait IndexableCategoryData {
    fn get_category_value(&self, row_index: usize, column_index: usize) -> u16;
    fn get_num_columns(&self) -> usize;
    fn get_num_rows(&self) -> usize;
    fn create_category_matrix(&self, row_index: usize) -> Box<dyn CategoryMatrix>;
}
