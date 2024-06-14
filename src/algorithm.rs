use rand::RngCore;
use rand::seq::SliceRandom;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use crate::data;
use crate::cluster;
use crate::dendrogram;

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

fn find_umerged_cluster(clusters: &Vec<cluster::Cluster>, mut index: usize) -> usize {
    while let Some(other) = clusters[index].merged_into {
        index = other;
    }
    index
}

pub fn create_dendrogram<D, R>(data: &D, init_iterations: Option<i32>, rng: &mut R) -> dendrogram::Dendrogram
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
            distance: clusters[*row_idx1].distance(&clusters[*row_idx2]),
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
                                distance: c1.distance(c2),
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
                            /* the stack overflow happens because of one of the instructions below: */
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
                            /* the stack overflow happens because of one of the instructions above */
                            dest.categories.extend(&*src.categories);
                            src.categories.clear();

                            src.merged_into = Some(dest_idx);
                            last_cluster_idx = dest_idx;
                        }
                    },
                    Some(c2_parent_index) => {
                        let unmerged_idx = find_umerged_cluster(&clusters, c2_parent_index);
                        if unmerged_idx != link.cluster1_index {
                            let unmerged = &clusters[unmerged_idx];
                            heap.push(cluster::Link{
                                distance: c1.distance(unmerged),
                                cluster1_index: link.cluster1_index,
                                cluster2_index: unmerged_idx,
                                cluster1_num_categories: c1.num_categories(),
                                cluster2_num_categories: unmerged.num_categories()
                            })
                        } 
                    }
                }
            },
            Some(c1_parent_index) => {
                match c2.merged_into {
                    None => {
                        let unmerged_idx = find_umerged_cluster(&clusters, c1_parent_index);
                        if unmerged_idx != link.cluster2_index {
                            let unmerged = &clusters[unmerged_idx];
                            heap.push(cluster::Link{
                                distance: c2.distance(unmerged),
                                cluster1_index: unmerged_idx,
                                cluster2_index: link.cluster2_index,
                                cluster1_num_categories: unmerged.num_categories(),
                                cluster2_num_categories: c2.num_categories()
                            })
                        }
                    },
                    Some(c2_parent_index) => {
                        let unmerged1_idx = find_umerged_cluster(&clusters, c1_parent_index);
                        let unmerged2_idx = find_umerged_cluster(&clusters, c2_parent_index);
                        if unmerged1_idx != unmerged2_idx {
                            let unmerged1 = &clusters[unmerged1_idx];
                            let unmerged2 = &clusters[unmerged1_idx];
                            heap.push(cluster::Link{
                                distance: unmerged1.distance(unmerged2),
                                cluster1_index: unmerged1_idx,
                                cluster2_index: unmerged2_idx,
                                cluster1_num_categories: unmerged1.num_categories(),
                                cluster2_num_categories: unmerged2.num_categories()
                            })
                        }
                    }
                }
            },
        }
    }

    let top_cluster = clusters.remove(last_cluster_idx);
    top_cluster.dendrogram.unwrap()
}