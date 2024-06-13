use rand::Rng;
use rand::RngCore;
use rand::seq::SliceRandom;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::any::Any;


mod data;
mod cluster;
mod dendrogram;

fn find_neighbors<D, R>(data: &D, init_iterations: Option<i32>, rng: &mut R) -> HashSet<(usize, usize)>
where
    D: data::IndexableCategoryData,
    R: rand::Rng,
{
    let init_ite =
    match init_iterations {
        Some(n) => std::cmp::max(n, 1),
        None => 1
    };

    let num_rows = data.get_num_rows();
    let num_columns = data.get_num_columns();    

    // find neighboring rows
    let mut neighbors: HashSet<(usize, usize)> = HashSet::new();
    let mut row_indices: Vec<usize> = (0..num_rows).collect();
    let mut col_indices: Vec<usize> = (0..num_columns).collect();

    for _ in 0..init_ite {
        for c in 0..num_columns {
            col_indices.shuffle(rng);
            for k in 0..num_columns {
                if col_indices[c] == k {
                    col_indices[c] = col_indices[num_columns-1];
                    col_indices[num_columns-1] = k;
                    break;
                }
            }
            row_indices.sort_by(|i, j|{
                for c in &col_indices {
                    let v1 = data.get_category_value(*i, *c);
                    let v2 = data.get_category_value(*j, *c);
                    if v1 < v2 {
                        return std::cmp::Ordering::Less;
                    } else if v2 > v1 {
                        return std::cmp::Ordering::Greater;
                    }
                }
                std::cmp::Ordering::Equal
            });
            for i in 0..num_rows-1 {
                let mut row1 = row_indices[i];
                let mut row2 = row_indices[i+1];
                if row1 > row2 {
                    std::mem::swap(&mut row1, &mut row2);
                }
                neighbors.insert((row1, row2));
            }
        }
    }

    neighbors
}

fn find_umerged_cluster(clusters: &Vec<cluster::Cluster>, index: usize) -> usize {
    match clusters[index].merged_into {
        None => index,
        Some(other) => find_umerged_cluster(clusters, other)
    }
}

fn create_dendrogram<D, R>(data: &D, init_iterations: Option<i32>, rng: &mut R) -> dendrogram::Dendrogram
where
    D: data::IndexableCategoryData,
    R: RngCore,
{
   let num_rows = data.get_num_rows();
   let num_cols = data.get_num_columns() as u16;
   let mut clusters: Vec<cluster::Cluster> = (0..num_rows).map({|r|
        cluster::Cluster {
            merged_into: None,
            categories: data.create_category_matrix(r),
            dendrogram: Some(dendrogram::Dendrogram::Leaf(r))
        }
    }).collect();    

    let neighbors = find_neighbors(data, init_iterations, rng);
    
    let mut heap: BinaryHeap<cluster::Link> = BinaryHeap::new();
    for (row_idx1, row_idx2) in &neighbors{
        heap.push(cluster::Link{
            distance: clusters[*row_idx1].symmetric_distance(&clusters[*row_idx2]),
            cluster1_index: *row_idx1,
            cluster2_index: *row_idx2,
            cluster1_num_categories: num_cols,
            cluster2_num_categories: num_cols
        });
    }

    let mut last_cluster_idx = 0;

    while let Some(link) = heap.pop() {
        let c1 = &clusters[link.cluster1_index];
        let c2 = &clusters[link.cluster2_index];

        match c1.merged_into {
            None => {
                match c2.merged_into {
                    None => {
                        let c1_len = c1.num_categories();
                        let c2_len = c2.num_categories();
                        if c1_len != link.cluster1_num_categories || c2_len != link.cluster2_num_categories {
                            // one of the two clusters have changed -> we need to update the distance
                            heap.push(cluster::Link{
                                distance: c1.symmetric_distance(c2),
                                cluster1_index: link.cluster1_index,
                                cluster2_index: link.cluster2_index,
                                cluster1_num_categories: c1_len,
                                cluster2_num_categories: c2_len
                            }) 
                        } else {
                            // we can merge the two clusters
                            assert!(link.cluster1_index != link.cluster2_index);
                            let (mut_c1, mut_c2) = 
                                if link.cluster1_index < link.cluster2_index {
                                    let (left, right) = clusters.split_at_mut(link.cluster2_index);
                                    (&mut left[link.cluster1_index], &mut right[0])
                                } else {
                                    let (left, right) = clusters.split_at_mut(link.cluster1_index);
                                    (&mut right[0], &mut left[link.cluster2_index])
                                };

                            let (src, dest, dest_idx) =
                                if c1_len > c2_len {
                                    // we will merge into the cluster that has more categories to make it more likely that it doesn't change
                                    (mut_c2, mut_c1, link.cluster1_index)
                                } else {
                                    (mut_c1, mut_c2, link.cluster2_index)
                                };

                            // update the cluster
                            let dendro1 = dest.dendrogram.take().unwrap();
                            let dendro2 = src.dendrogram.take().unwrap();
                            let new_size = dendro1.size() + dendro2.size();
                            dest.dendrogram = Some(
                                dendrogram::Dendrogram::Node(
                                    Box::new(dendro1),
                                    Box::new(dendro2),
                                    link.distance,
                                    new_size,
                                )
                            );
                            dest.categories.extend(&*src.categories);

                            // TODO: clear memory of src

                            src.merged_into = Some(dest_idx);
                            last_cluster_idx = dest_idx;
                        }
                    },
                    Some(other) => {
                        let unmerged_idx = find_umerged_cluster(&clusters, other);
                        if unmerged_idx != link.cluster1_index {
                            let unmerged = &clusters[unmerged_idx];
                            heap.push(cluster::Link{
                                distance: c1.symmetric_distance(unmerged),
                                cluster1_index: link.cluster1_index,
                                cluster2_index: unmerged_idx,
                                cluster1_num_categories: c1.num_categories(),
                                cluster2_num_categories: unmerged.num_categories()
                            })
                        } 
                    }
                }
            },
            Some(other) => {
                match c2.merged_into {
                    None => {
                        let unmerged_idx = find_umerged_cluster(&clusters, other);
                        if unmerged_idx != link.cluster2_index {
                            let unmerged = &clusters[unmerged_idx];
                            heap.push(cluster::Link{
                                distance: c2.symmetric_distance(unmerged),
                                cluster1_index: unmerged_idx,
                                cluster2_index: link.cluster2_index,
                                cluster1_num_categories: unmerged.num_categories(),
                                cluster2_num_categories: c2.num_categories()
                            })
                        }
                    },
                    Some(_) => {
                        // do nothing as both clusters have been merged already
                    }
                }
            },
        }
    }

    let top_cluster = clusters.remove(last_cluster_idx);
    top_cluster.dendrogram.unwrap()
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


fn main() -> () {
    let mut rng = rand::thread_rng();
    let matrix = create_random_matrix(100_000, 5);
    let dendro = create_dendrogram(&matrix, None, &mut rng);

    let clusters = dendrogram::find_clusters(&dendro, matrix.len(), 1000);
}