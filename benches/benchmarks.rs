use criterion::{criterion_group, criterion_main, Criterion};

use rand::Rng;
use std::any::Any;
use std::collections::HashSet;
use std::vec::Vec;

struct SimpleMatrix {
    col1to4: u32,
    col5: HashSet<u16>,
}

struct MyData {
    vecs: Vec<Vec<i32>>,
}

impl catclustering::ClusterSummary for SimpleMatrix {
    fn num_categories(&self) -> u16 {
        self.col1to4.count_ones() as u16 + self.col5.len() as u16
    }
    fn distance(&self, other: &dyn catclustering::ClusterSummary) -> i16 {
        let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();
        (self.col1to4 ^ o.col1to4).count_ones() as i16
            + self.col5.symmetric_difference(&o.col5).count() as i16
    }
    fn extend(&mut self, other: &dyn catclustering::ClusterSummary) {
        let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();

        self.col1to4 |= o.col1to4;
        self.col5.extend(&o.col5);
    }
    fn clear(&mut self) {
        self.col5.clear();
        self.col5.shrink_to_fit();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl catclustering::IndexableCategoryData for MyData {
    fn get_value(&self, row_index: usize, column_index: usize) -> u16 {
        self.vecs[row_index][column_index] as u16
    }

    fn get_num_columns(&self) -> usize {
        self.vecs[0].len()
    }

    fn get_num_rows(&self) -> usize {
        self.vecs.len()
    }

    fn create_cluster_summary(&self, row_index: usize) -> Box<dyn catclustering::ClusterSummary> {
        let row = &self.vecs[row_index];
        Box::new(SimpleMatrix {
            col1to4: (1 << row[0])
                | (1 << (row[1] + 8))
                | (2 << (row[2] + 16))
                | (1 << (row[3] + 24)),
            col5: HashSet::from_iter(vec![row[4] as u16]),
        })
    }
}

fn create_random_matrix(rows: usize, cols: usize) -> Vec<Vec<i32>> {
    let mut rng = rand::thread_rng();
    let mut matrix = Vec::with_capacity(rows);

    for _ in 0..rows {
        let row: Vec<i32> = (0..cols).map(|_| rng.gen_range(0..5)).collect();
        matrix.push(row);
    }

    matrix
}

fn any_size(c: &mut Criterion, n_rows: usize) {
    let mut group = c.benchmark_group("custom-sample-count");
    group.sample_size(10);

    let mut rng = rand::thread_rng();
    let matrix = MyData {
        vecs: create_random_matrix(n_rows, 5),
    };

    group.bench_function(format!("{n_rows} rows"), |b| {
        b.iter(|| {
            catclustering::create_dendrogram(&matrix, None, &mut rng);
        });
    });
    group.finish();
}

fn size1(c: &mut Criterion) {
    any_size(c, 10_000);
}

fn size2(c: &mut Criterion) {
    any_size(c, 100_000);
}

fn size3(c: &mut Criterion) {
    any_size(c, 1_000_000);
}

criterion_group!(benches, size1, size2, size3);
criterion_main!(benches);
