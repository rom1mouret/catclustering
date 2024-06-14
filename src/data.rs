use std::any::Any;

pub trait CategoryMatrix {
    fn num_categories(&self) -> u16;

    // Doesn't have to be a proper distance. It can:
    // - negative
    // - not satisfy the triangle inequality
    // However, it has to be symmetric.
    fn distance(&self, other: &dyn CategoryMatrix) -> i16;
    fn extend(&mut self, other: &dyn CategoryMatrix);
    fn clear(&mut self);
    fn as_any(&self) -> &dyn Any;
}   

pub trait IndexableCategoryData {
    fn get_category_value(&self, row_index: usize, column_index: usize) -> u16;
    fn get_num_columns(&self) -> usize;
    fn get_num_rows(&self) -> usize;
    fn create_category_matrix(&self, row_index: usize) -> Box<dyn CategoryMatrix>;
}
