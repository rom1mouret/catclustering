use std::any::Any;

pub trait CategoryMatrix {
    fn num_categories(&self) -> u16;
    fn symmetric_distance(&self, other: &dyn CategoryMatrix) -> u16;
    fn extend(&mut self, other: &dyn CategoryMatrix);
    fn as_any(&self) -> &dyn Any;
}   

pub trait IndexableCategoryData {
    fn get_category_value(&self, row_index: usize, column_index: usize) -> u16;
    fn get_num_columns(&self) -> usize;
    fn get_num_rows(&self) -> usize;
    fn create_category_matrix(&self, row_index: usize) -> Box<dyn CategoryMatrix>;
}
