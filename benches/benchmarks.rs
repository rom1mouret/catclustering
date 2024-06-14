use criterion::{criterion_group, criterion_main, Criterion};

use std::vec::Vec;
use std::collections::HashSet;
use std::any::Any;
use rand::Rng;
use catclustering::CategoryMatrix;
use catclustering::IndexableCategoryData;
use catclustering::create_dendrogram;

struct SimpleMatrix {
    sets: Vec<HashSet<u16>>,
}

struct MyData {
    vecs: Vec<Vec<i32>>
}


impl catclustering::CategoryMatrix for SimpleMatrix {
    fn num_categories(&self) -> u16 {
        let mut n = 0;
        for h in &self.sets {
            n += h.len();
        }
        n as u16
    }
    fn distance(&self, other: &dyn catclustering::CategoryMatrix) -> i16 {
        // TODO: panic here if not right type
        other.as_any().downcast_ref::<SimpleMatrix>().map_or(0, |other_matrix| {
            let mut d = 0;
            for i in 0..self.sets.len() {
                d += self.sets[i].symmetric_difference(&other_matrix.sets[i]).count();
            }
            d as i16
        })
    }
    fn extend(&mut self, other: &dyn catclustering::CategoryMatrix) {
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
    fn clear(&mut self) {
        self.sets.clear();
        self.sets.shrink_to_fit();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}   

impl catclustering::IndexableCategoryData for MyData {
    fn get_category_value(&self, row_index: usize, column_index: usize) -> u16 {
        self.vecs[row_index][column_index] as u16
    }

    fn get_num_columns(&self) -> usize {
        // Relying on the assumption all rows are of the same length.
        self.vecs[0].len()
    }

    fn get_num_rows(&self) -> usize {
        self.vecs.len()
    }

    fn create_category_matrix(&self, row_index: usize) -> Box<dyn catclustering::CategoryMatrix> {
        Box::new(SimpleMatrix{
            sets: (0..self.vecs[0].len()).map(|i| {
                HashSet::from_iter(vec![self.vecs[row_index][i] as u16])
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

fn any_size(c: &mut Criterion, n_rows: usize) {
    let mut group = c.benchmark_group("custom-sample-count");
    group.sample_size(10);

    let mut rng = rand::thread_rng();
    let matrix = MyData{vecs: create_random_matrix(n_rows, 5)};

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

fn size4(c: &mut Criterion) {
    any_size(c, 10_000_000);
}


criterion_group!(
    benches,
    size1,
    size2,
    size3,
    size4
);
criterion_main!(benches);