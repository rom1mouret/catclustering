mod algorithm;
mod dendrogram;
mod data;
mod cluster;

pub use algorithm::create_dendrogram;
pub use dendrogram::find_clusters;
pub use dendrogram::assign_rows_to_clusters;
pub use data::CategoryMatrix;
pub use data::IndexableCategoryData;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::any::Any;
    use rand::Rng;
    use super::*;
    

    struct SimpleMatrix {
        sets: Vec<HashSet<u16>>,
    }


    impl data::CategoryMatrix for SimpleMatrix {
        fn num_categories(&self) -> u16 {
            let mut n = 0;
            for h in &self.sets {
                n += h.len();
            }
            n as u16
        }
        fn symmetric_distance(&self, other: &dyn data::CategoryMatrix) -> u16 {
            // TODO: panic here if not right type
            other.as_any().downcast_ref::<SimpleMatrix>().map_or(0, |other_matrix| {
                let mut d = 0;
                for i in 0..self.sets.len() {
                    d += self.sets[i].symmetric_difference(&other_matrix.sets[i]).count();
                }
                d as u16
            })
        }
        fn extend(&mut self, other: &dyn data::CategoryMatrix) {
            // TODO: panic here
            match other.as_any().downcast_ref::<SimpleMatrix>() {
                Some(other_matrix) => {
                    for i in 0..self.sets.len() {
                        self.sets[i].extend(&other_matrix.sets[i]);
                    }
                }
                None => {},
            }
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }   

    impl data::IndexableCategoryData for Vec<Vec<i32>> {
        fn get_category_value(&self, row_index: usize, column_index: usize) -> u16 {
            self[row_index][column_index] as u16
        }

        fn get_num_columns(&self) -> usize {
            // Relying on the assumption all rows are of the same length.
            self[0].len()
        }

        fn get_num_rows(&self) -> usize {
            self.len()
        }

        fn create_category_matrix(&self, row_index: usize) -> Box<dyn data::CategoryMatrix> {
            Box::new(SimpleMatrix{
                sets: (0..self[0].len()).map(|i| {
                    HashSet::from_iter(vec![self[row_index][i] as u16])
                }).collect()
            })
        }
    }

    fn create_random_matrix(rows: usize, cols: usize) -> Vec<Vec<i32>> {
        let mut rng = rand::thread_rng();
        let mut matrix = Vec::with_capacity(rows);

        for _ in 0..rows {
            let row: Vec<i32> = (0..cols)
                .map(|_| rng.gen_range(0..5))
                .collect();
            matrix.push(row);
        }

        matrix
    }


    #[test]
    fn test_clear_clusters() {
        let n_clusters = 10;
        let n_rows = n_clusters * 12;

        let mut rng = rand::thread_rng();
        let mut matrix = create_random_matrix(n_rows, 3);

        
        // initialize matrix with only 10 different values that should be clustered together
        for i in 0..matrix.len() {
            let v = i % n_clusters;
            for c in 0..matrix[0].len() {
                matrix[i][c] = v as i32;
            }
        }

        let dendro = create_dendrogram(&matrix, None, &mut rng);
        let clusters = dendrogram::find_clusters(&dendro, n_rows, n_rows / n_clusters);
        
        let clustered_rows = clusters.iter().map(|v| v.len()).sum();
        assert!(n_rows == clustered_rows);

        println!("number of clusters: {}", clusters.len());

        assert!(clusters.len() == n_clusters);
        for row_indices in &clusters {
            assert!(row_indices.len() == n_rows / n_clusters);
            let t = row_indices[0] % n_clusters;
            for i in row_indices {
                assert!(i % n_clusters == t);
            }
        }
        
    }


    #[test]
    fn test_add() {
        let mut rng = rand::thread_rng();
        let matrix = create_random_matrix(10_000, 5);
        let dendro = create_dendrogram(&matrix, None, &mut rng);

        let clusters = dendrogram::find_clusters(&dendro, matrix.len(), 1000);
        
    }
}
