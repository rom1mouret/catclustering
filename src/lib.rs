//! # CatClustering
//!
//! `catclustering` implements hierarchical agglomerative clustering optimized for categorical features and (approximated) complete-linkage.
//! Explanation and examples [here](https://github.com/rom1mouret/catclustering).

mod algorithm;
mod cluster;
mod data;
mod dendrogram;

pub use algorithm::create_dendrogram;
pub use data::ClusterSummary;
pub use data::IndexableData;
pub use dendrogram::assign_rows_to_clusters;
pub use dendrogram::find_clusters;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::any::Any;
    use std::collections::HashSet;

    struct SimpleMatrix {
        sets: Vec<HashSet<u16>>,
    }

    impl data::ClusterSummary for SimpleMatrix {
        fn summary_size(&self) -> usize {
            let mut n = 0;
            for h in &self.sets {
                n += h.len();
            }
            n
        }
        fn distance(&self, other: &dyn data::ClusterSummary) -> f32 {
            let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();
            let mut d = self.summary_size() + o.summary_size();
            for i in 0..self.sets.len() {
                d -= self.sets[i].intersection(&o.sets[i]).count();
            }
            d as f32
        }
        fn extend(&mut self, other: &dyn data::ClusterSummary) {
            let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();
            for i in 0..self.sets.len() {
                self.sets[i].extend(&o.sets[i]);
            }
        }
        fn clear(&mut self) {
            self.sets.clear();
            self.sets.shrink_to_fit();
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl data::IndexableData for Vec<Vec<i32>> {
        fn get_value(&self, row_index: usize, column_index: usize) -> f32 {
            self[row_index][column_index] as f32
        }

        fn get_num_columns(&self) -> usize {
            self[0].len()
        }

        fn get_num_rows(&self) -> usize {
            self.len()
        }

        fn create_cluster_summary(&self, row_index: usize) -> Box<dyn data::ClusterSummary> {
            Box::new(SimpleMatrix {
                sets: (0..self[0].len())
                    .map(|i| HashSet::from_iter(vec![self[row_index][i] as u16]))
                    .collect(),
            })
        }
    }

    fn create_random_matrix(
        rows: usize,
        cols: usize,
        range: std::ops::Range<i32>,
    ) -> Vec<Vec<i32>> {
        let mut rng = rand::thread_rng();
        let mut matrix = Vec::with_capacity(rows);

        for _ in 0..rows {
            let row: Vec<i32> = (0..cols).map(|_| rng.gen_range(range.clone())).collect();
            matrix.push(row);
        }

        matrix
    }

    #[test]
    fn test_clear_boundaries() {
        let mut rng = rand::thread_rng();
        for iteration in 1..10 {
            let n_clusters = 10 * iteration;
            let n_rows = n_clusters * 12;

            let mut matrix = create_random_matrix(n_rows, 3, 0..1);

            // initialize matrix with only n_clusters different values that should be clustered together
            for i in 0..matrix.len() {
                let v = i % n_clusters;
                for c in 0..matrix[0].len() {
                    matrix[i][c] = v as i32;
                }
            }

            let dendro = create_dendrogram(&matrix, None, &mut rng);
            let clusters = dendrogram::find_clusters(&dendro, n_rows / n_clusters);

            let clustered_rows = clusters.iter().map(|v| v.len()).sum();
            assert!(n_rows == clustered_rows);

            assert!(clusters.len() == n_clusters);
            for row_indices in &clusters {
                assert!(row_indices.len() == n_rows / n_clusters);
                let t = row_indices[0] % n_clusters;
                for i in row_indices {
                    assert!(i % n_clusters == t);
                }
            }
        }
    }

    #[test]
    fn test_two_clusters() {
        let cluster_size = 100;
        let matrix1 = create_random_matrix(cluster_size, 3, 0..4);
        let matrix2 = create_random_matrix(cluster_size, 3, 5..10);

        let mut matrix = Vec::new();
        matrix.extend(matrix1);
        matrix.extend(matrix2);

        let mut rng = rand::thread_rng();
        let dendro = create_dendrogram(&matrix, None, &mut rng);
        let clusters = dendrogram::find_clusters(&dendro, cluster_size);

        assert!(clusters.len() == 2);
        assert!(clusters[0].len() == cluster_size);
        assert!(clusters[1].len() == cluster_size);

        let (m1, m2) = if clusters[0] < clusters[1] {
            (&clusters[0], &clusters[1])
        } else {
            (&clusters[1], &clusters[0])
        };

        assert!(*m1.iter().min().unwrap() == 0);
        assert!(*m1.iter().max().unwrap() == cluster_size - 1);
        assert!(*m2.iter().min().unwrap() == cluster_size);
        assert!(*m2.iter().max().unwrap() == 2 * cluster_size - 1);
    }
}
