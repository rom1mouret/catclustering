use std::mem::{replace, ManuallyDrop};

pub enum Dendrogram {
    Leaf(usize),                                        // row index
    Node(Box<Dendrogram>, Box<Dendrogram>, f32, usize), // cluster1, cluster2, distance, size
}

impl Dendrogram {
    pub fn size(&self) -> usize {
        match self {
            Dendrogram::Leaf(_) => 1,
            Dendrogram::Node(_, _, _, s) => *s,
        }
    }
}

/// Rust's default's implementation of drop() is recursive, so we write a custom
/// non-recursive drop() to avoid stack overflows.
impl Drop for Dendrogram {
    fn drop(&mut self) {
        let mut stack = vec![ManuallyDrop::new(replace(self, Dendrogram::Leaf(0)))];

        while let Some(mut current) = stack.pop() {
            match &mut *current {
                Dendrogram::Leaf(_) => {}
                Dendrogram::Node(ref mut cluster1, ref mut cluster2, _, _) => {
                    // Move the boxes out to avoid double-free issues
                    let left = replace(cluster1, Box::new(Dendrogram::Leaf(0)));
                    let right = replace(cluster2, Box::new(Dendrogram::Leaf(0)));

                    // Extract the inner Dendrogram from the Box and push it onto the stack
                    stack.push(ManuallyDrop::new(*left));
                    stack.push(ManuallyDrop::new(*right));
                }
            }
        }
    }
}

fn assign_rows_to<'a>(
    cluster_n: usize,
    dendrogram: &'a Dendrogram,
    assigments: &mut Vec<usize>,
    stack: &mut Vec<&'a Dendrogram>,
) {
    stack.clear();
    stack.push(dendrogram);

    while let Some(current) = stack.pop() {
        match current {
            Dendrogram::Leaf(row_index) => {
                assigments[*row_index] = cluster_n;
            }
            Dendrogram::Node(cluster1, cluster2, _, _) => {
                stack.push(cluster1);
                stack.push(cluster2);
            }
        }
    }
}

/// This assigns a cluster number to each row under the constraint that all clusters must have the same size or be smaller than
/// the provided size.
///
/// # Arguments
///
/// * `dendrogram` - The dendogram returned by create_dendrogram.
/// * `assignments` - cluster number for each row. This is passed as an argument for you to be able to reuse the vector across multiple calls.
///    The vector will be resized if too small.
/// * `max_cluster_size` - The maximum size of the returned clusters.
///
/// # Returns
///
/// The number of clusters found.
///
pub fn assign_rows_to_clusters(
    dendrogram: &Dendrogram,
    assignments: &mut Vec<usize>,
    max_cluster_size: usize,
) -> usize {
    if dendrogram.size() > assignments.len() {
        assignments.resize(dendrogram.size(), usize::MAX);
    }

    let mut cluster_n = 0;
    let mut stack = Vec::new();
    let mut same_cluster_stack = Vec::new();
    stack.push(dendrogram);

    while let Some(current) = stack.pop() {
        match current {
            Dendrogram::Leaf(row_index) => {
                assignments[*row_index] = cluster_n;
                cluster_n += 1;
            }
            Dendrogram::Node(cluster1, cluster2, _, size) => {
                if *size > max_cluster_size {
                    // dive deeper
                    stack.push(cluster1);
                    stack.push(cluster2);
                } else {
                    assign_rows_to(cluster_n, &current, assignments, &mut same_cluster_stack);
                    cluster_n += 1;
                }
            }
        }
    }
    cluster_n
}

/// This traverses the dendrogram until it finds clusters of the same size or smaller than the given size,
/// and returns these clusters.
///
/// # Arguments
///
/// * `dendrogram` - The dendogram returned by create_dendrogram.
/// * `max_cluster_size` - The maximum size of the clusters returned.
///
/// # Returns
///
/// The indices of the rows that belong to each returned cluster.
///
pub fn find_clusters(dendrogram: &Dendrogram, max_cluster_size: usize) -> Vec<Vec<usize>> {
    let n_rows = dendrogram.size();
    let mut assignments = vec![usize::MAX as usize; n_rows];
    let num_clusters = assign_rows_to_clusters(&dendrogram, &mut assignments, max_cluster_size);
    let mut clusters: Vec<Vec<usize>> = (0..num_clusters).map(|_| Vec::new()).collect();

    for (row_idx, assignment) in assignments.iter().enumerate() {
        clusters[*assignment].push(row_idx)
    }
    return clusters;
}
